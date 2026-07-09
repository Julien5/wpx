#![allow(non_snake_case)]

use std::collections::BTreeMap;

use crate::controls;
use crate::error::TrackError;
use crate::gpxexport;
use crate::inputpoint::*;
use crate::make_points;
use crate::math::IntegerSize2D;
use crate::parameters;
use crate::parameters::*;
use crate::point_collection::*;
use crate::segment::SegmentData;
use crate::speed::powergeometry::ConstantPowerGeometry;
use crate::speed::*;
use crate::split_ambiguity;
use crate::track::SharedTrack;
use crate::trackfile::*;
use crate::waypoint;
use crate::waypoint::ExportParameters;
use crate::waypoint::FlatWaypoints;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointInfo;
use crate::waypoint::Waypoints;
use crate::wheel;
use crate::zipexport;

pub type Segment = crate::segment::Segment;
pub type SegmentStatistics = crate::segment::SegmentStatistics;

pub struct BackendData {
    pub parameters: Parameters,
    pub trackfile: Option<TrackFile>,
    pub track: SharedTrack,
    pub packet_provider: PacketProvider,
}

use chrono::TimeDelta;

impl BackendData {
    pub fn make_segment_data(&self, segment: &Segment) -> SegmentData<'_> {
        SegmentData::new(
            segment,
            self.track.clone(),
            &self.packet_provider,
            self.parameters.clone(),
            self.time_parameters(),
        )
    }

    pub fn load_osm(&mut self, mut osmpoints: InputPointMap) {
        self.track.project_map(&mut osmpoints);
        self.packet_provider
            .collection
            .import_osm(&osmpoints.as_vector());
    }

    pub fn get_parameters(&self) -> Parameters {
        self.parameters.clone()
    }

    pub fn track_dataset(&self) -> TrackDataset {
        let waypoints = self
            .packet_provider
            .collection
            .get_vector(&Kind::GPXWaypoints);
        TrackDataset::from_track_and_waypoints(&self.track, &waypoints)
    }

    pub fn small_parameters(&self) -> JsonParameters {
        log::trace!("persist [1]");
        let controls = self.packet_provider.collection.get_vector(&Kind::Controls);
        JsonParameters {
            parameters: self.parameters.clone(),
            controls: controls.clone(),
            trackfile: self.trackfile.as_ref().unwrap().clone(),
        }
    }

    pub fn update_trackfile_name(&mut self, name: &str) -> JsonParameters {
        self.trackfile.as_mut().unwrap().name = format!("{}", name);
        self.small_parameters()
    }

    pub fn set_parameters(&mut self, parameters: &Parameters) {
        let old_time_parameters = self.time_parameters();
        let new_time_parameters = TimeParameters {
            controls: Vec::new(),
            start: parameters::parse_time(&parameters.start_time),
            speed: parse_speed(&parameters.speed),
            track_distance: self.track.total_distance(),
            power: None,
        };
        self.parameters = parameters.clone();

        // load_ordered calls set_parameters to update user steps (only for that?)
        // trackfile is not set in this case.
        if self.trackfile.is_some() {
            self.trackfile.as_mut().unwrap().start_time = parameters.start_time.clone();
        }

        // unsupported ?
        if self.parameters.segment_overlap > self.parameters.segment_length {
            debug_assert!(false);
        }

        // update user steps
        {
            let usersteps =
                make_points::user_points(&self.track, &self.parameters.user_steps_options);
            self.packet_provider
                .collection
                .import_other(&Kind::CutOff, usersteps);
        }

        let old_start = old_time_parameters.time(0f64);
        let new_start = new_time_parameters.time(0f64);
        let new_end = new_time_parameters.time(self.track.total_distance());

        // reset control time
        // we might have t(end) < t(CP) (if the speed gets higher).
        // a less drastic measure would be to only reset the time
        // on controls which time are after the time of the last control.
        {
            let mut controls = self.packet_provider.collection.get_vector(&Kind::Controls);

            // compute delta
            let mut delta_from_start: BTreeMap<usize, TimeDelta> = BTreeMap::new();
            for c in &mut controls {
                match c.data.as_control().unwrap().cutoff_time {
                    Some(t) => {
                        let index = c.track_projections.first().unwrap().track_index;
                        debug_assert!(t >= old_start);
                        delta_from_start.insert(index, t - old_start);
                    }
                    None => {}
                };
            }

            // apply delta
            for c in &mut controls {
                let index = c.track_projections.first().unwrap().track_index;
                let cdata = c.data.as_control().unwrap();
                let new_cutoff = match cdata.cutoff_time {
                    Some(_) => {
                        debug_assert!(delta_from_start.contains_key(&index));
                        // the END control time cannot be set.
                        debug_assert!(!cdata.is_end());
                        let delta = delta_from_start[&index];
                        let candidate = new_start + delta;
                        if candidate < new_end {
                            Some(candidate)
                        } else {
                            None
                        }
                    }
                    None => None,
                };
                c.data.as_control_mut().unwrap().cutoff_time = new_cutoff;
            }
            self.packet_provider
                .collection
                .import_other(&Kind::Controls, controls);
        }
    }

    pub fn get_points(&self, segment: &Segment, kinds: &Kinds) -> Vec<InputPoint> {
        let mut points = Vec::new();
        let range = self.track.subrange(segment.start, segment.end);
        if kinds.is_empty() {
            return Vec::new();
        }

        // take care of the GPXWaypoints/Control case first.
        if kinds.contains(&Kind::GPXWaypoints) {
            let controls = self.packet_provider.collection.get_vector(&Kind::Controls);
            let mut waypoints = self
                .packet_provider
                .collection
                .get_vector(&Kind::GPXWaypoints);
            if kinds.contains(&Kind::Controls) {
                waypoints = remove_control_waypoints(&waypoints, &controls);
            }
            points.extend_from_slice(&waypoints);
        }

        for kind in kinds {
            if *kind == Kind::GPXWaypoints {
                continue;
            }
            let kpoints = self.packet_provider.collection.get_vector(kind);
            let mut copy = kpoints.clone();
            copy.retain(|w| {
                w.is_close_to_track()
                    && range.contains(&w.track_projections.first().unwrap().track_index)
            });
            points.extend_from_slice(&copy);
        }
        log::info!(
            "segment: {} [{:.1}:{:.1}] export {} waypoints",
            segment.id,
            segment.start / 1000f64,
            segment.end / 1000f64,
            points.len()
        );
        points
    }

    fn controls(&self) -> Vec<InputPoint> {
        self.packet_provider.collection.get_vector(&Kind::Controls)
    }

    fn time_parameters(&self) -> TimeParameters {
        let start_time = parameters::parse_time(&self.parameters.start_time);
        let t0 = TimeParameters {
            controls: controls_speed_data(&start_time, &self.controls()),
            start: start_time,
            speed: parse_speed(&self.parameters.speed),
            track_distance: self.track.total_distance(),
            power: None,
        };

        let power_geometry = ConstantPowerGeometry::new(&self.track.simplified);
        TimeParameters {
            controls: controls_speed_data(&start_time, &self.controls()),
            start: parameters::parse_time(&self.parameters.start_time),
            speed: parse_speed(&self.parameters.speed),
            track_distance: self.track.total_distance(),
            power: Some(power_geometry.interpolation_points(&t0.control_interpolation_points())),
        }
    }

    pub fn export_points(&self, points: &Vec<InputPoint>) -> Waypoints {
        let projections = InputPoint::flatten_projections(&points);
        let mut list = FlatWaypoints::new();
        for (index, projection) in projections {
            let w = points[index].waypoint(&projection);
            list.push((projection.clone(), w));
            log::trace!(
                "export: {} => index:{}",
                points[index].name(),
                projection.track_floating_index
            );
        }
        debug_assert!(
            points.len() <= list.len(),
            "points:{} != map:{}",
            points.len(),
            list.len()
        );
        let export_parameters = ExportParameters {
            parameters: self.parameters.clone(),
            time_parameters: self.time_parameters(),
        };
        WaypointInfo::make_waypoint_infos(&mut list, &self.track, &export_parameters);
        list.iter().map(|(_proj, w)| w.clone()).collect()
    }

    pub fn get_waypoints(&self, segment: &Segment, kinds: &Kinds) -> Vec<Waypoint> {
        self.export_points(&self.get_points(&segment, kinds))
    }

    pub fn generatePdf(&self, kinds: &Kinds) -> Result<Vec<u8>, TrackError> {
        /*let typbytes = render::make_typst_document(self, kinds);
        let ret = pdf::compile(&typbytes, self.get_parameters().debug).await;*/
        let ret = crate::pdf::render::make_pdf_document(self, kinds)?;
        log::info!("generated {} pdf bytes", ret.len());
        Ok(ret)
    }
    pub fn generateGpx(&self) -> BTreeMap<String, Vec<u8>> {
        let collection = &self.packet_provider.collection;
        let usersteps = collection.get_vector(&Kind::CutOff);
        let waypoints = collection.get_vector(&Kind::GPXWaypoints);
        let controls = collection.get_vector(&Kind::Controls);
        let split_indices = split_ambiguity::user_steps_split(&usersteps, &controls, &self.track);
        let userssteps_w = self.export_points(&usersteps);
        let usersteps_groups = waypoint::group_waypoints(&userssteps_w, &split_indices);
        let mut check_sum = 0;
        for g in &usersteps_groups {
            check_sum += g.len();
        }
        debug_assert_eq!(check_sum, userssteps_w.len());
        debug_assert!(!usersteps_groups.is_empty());
        let waypoints_w = self.export_points(&waypoints);
        gpxexport::generate(&self.track, &controls, &usersteps_groups, &waypoints_w)
    }

    pub fn generateZip(&self, kinds: &Kinds) -> Result<Vec<u8>, TrackError> {
        let mut map = self.generateGpx();
        let pdf = self.generatePdf(kinds)?;
        map.insert("route.pdf".to_string(), pdf);
        Ok(zipexport::generate(map))
    }

    pub fn set_start_time(&mut self, rfc3339: String) {
        self.parameters.start_time = rfc3339;
    }

    pub fn set_segment_length(&mut self, length: f64) {
        self.parameters.segment_length = length;
    }

    pub fn segments(&self) -> Vec<Segment> {
        let mut ret = Vec::new();

        let mut start = 0f64;
        let mut k = 0usize;
        loop {
            let end = start + self.parameters.segment_length;
            ret.push(Segment {
                id: k as i32,
                start,
                end,
            });
            log::trace!("end:{} l={}", end, self.track.total_distance());
            if end > self.track.total_distance() {
                break;
            }
            start += self.parameters.segment_length - self.parameters.segment_overlap;
            k = k + 1;
        }
        ret
    }

    pub fn trackSegment(&self) -> Segment {
        let start = 0f64;
        let end = self.track.total_distance();
        Segment { id: -1, start, end }
    }

    pub fn render_segment_simple(
        &self,
        segment: &Segment,
        size: &IntegerSize2D,
        kinds: Kinds,
        function: RenderFunction,
    ) -> String {
        let input = RenderInput {
            kinds,
            function,
            size: (size.width, size.height),
        };
        self.render_segment(segment, &vec![input]).remove(0).svg
    }

    pub fn load_controls(&mut self) -> Result<usize, TrackError> {
        let waypoints = self
            .packet_provider
            .collection
            .get_vector(&Kind::GPXWaypoints);
        let mut controls = controls::infer_controls_from_gpx_segments(&self.track, &waypoints);
        for c in &mut controls {
            debug_assert!(!c.track_projections.is_empty());
            if c.track_projections.is_empty() {
                self.track.project_point(c);
            }
        }

        let len = controls.len();
        debug_assert!(len >= 2);

        self.packet_provider
            .collection
            .import_other(&Kind::Controls, controls);

        Ok(len)
    }

    pub fn make_control_at_waypoint(&mut self, waypoint: &Waypoint, on: bool) {
        let controls = self.packet_provider.collection.get_vector(&Kind::Controls);
        let new = match on {
            true => controls::add_control_at_waypoint(&self.track, controls, waypoint),
            false => controls::remove_control_at_waypoint(controls, waypoint),
        };
        {
            self.packet_provider
                .collection
                .import_other(&Kind::Controls, new);
        }
    }

    pub fn set_control_time(&mut self, waypoint: &Waypoint, time: &Option<String>) -> bool {
        match self.time_parameters().speed {
            Speed::ACP(_) => {
                return false;
            }
            Speed::LRM(_) => {
                return false;
            }
            Speed::KMH(_) => {}
        }
        let mut controls = self.packet_provider.collection.get_vector(&Kind::Controls);
        if let Some(control) = controls
            .iter_mut()
            .find(|c| c.index().is_some_and(|id| id == waypoint.index.unwrap()))
        {
            // do not allow changing time for start and end because
            // these are determined by self.parameters (start_time and speed).
            if control.data.as_control().unwrap().is_end()
                || control.data.as_control().unwrap().is_start()
            {
                return false;
            }
            if let Some(data) = time {
                let t = parameters::parse_time(&data);
                control.data.as_control_mut().unwrap().cutoff_time = Some(t);
            } else {
                control.data.as_control_mut().unwrap().cutoff_time = None;
            }
        } else {
            log::error!("no control found with id={:?}", waypoint.index);
        }
        self.packet_provider
            .collection
            .import_other(&Kind::Controls, controls);
        true
    }

    pub fn render_segment(
        &self,
        segment: &Segment,
        render_inputs: &Vec<RenderInput>,
    ) -> Vec<RenderOutput> {
        if render_inputs.len() == 2 {
            let sizes: BTreeMap<_, _> = render_inputs
                .iter()
                .map(|input| (input.function.clone(), input.size))
                .collect();
            let kinds = render_inputs.first().unwrap().kinds.clone();
            match (
                sizes.get(&RenderFunction::Map),
                sizes.get(&RenderFunction::Profile),
            ) {
                (Some(msize), Some(psize)) => {
                    let map_size = IntegerSize2D::new(msize.0, msize.1);
                    let profile_size = IntegerSize2D::new(psize.0, psize.1);
                    return self.render_segment_map_profile(
                        segment,
                        &map_size,
                        &profile_size,
                        kinds,
                    );
                }
                _ => {}
            }
        }

        let segment = self.make_segment_data(segment);
        let mut ret = Vec::new();
        let time_parameters = self.time_parameters();
        for render_input in render_inputs {
            let size = IntegerSize2D::new(render_input.size.0, render_input.size.1);

            let render_result = match render_input.function {
                RenderFunction::Profile => segment.render_profile(&size, &render_input.kinds),
                RenderFunction::Map => segment.render_map(&size, &render_input.kinds),
                RenderFunction::Wheel => {
                    let mut model = wheel::model::WheelModel::new(&time_parameters);
                    model.add_points(&segment, &render_input.kinds);
                    wheel::render(&size, &model)
                }
                RenderFunction::WheelPages => {
                    let mut model = wheel::model::WheelModel::new(&time_parameters);
                    model.add_points(&segment, &render_input.kinds);
                    model.add_pages(&self.segments());
                    wheel::render(&size, &model)
                }
                RenderFunction::Unknown => {
                    panic!("The render function is not set. Bye.");
                }
            };
            log::info!(
                "done - render_segment_what:{} {:?}",
                segment.id(),
                render_input.function
            );
            let points = render_result.rendered_input_points_for_table();
            ret.push(RenderOutput {
                svg: render_result.svg,
                render_input: render_input.clone(),
                error: None,
                waypoints: waypoint::table(&segment, &points),
            });
        }
        ret
    }

    pub fn render_segment_map_profile(
        &self,
        segment: &Segment,
        map_size: &IntegerSize2D,
        profile_size: &IntegerSize2D,
        kinds: Kinds,
    ) -> Vec<RenderOutput> {
        log::info!(
            "start - render_segment_profile_map:{} map_size:{}x{} profile_size:{}x{}",
            segment.id,
            map_size.width,
            map_size.height,
            profile_size.width,
            profile_size.height
        );
        let data = self.make_segment_data(segment);
        let (result_map, result_profile) = data.render_map_profile(map_size, profile_size, &kinds);
        let mut ret = Vec::new();
        ret.push((RenderFunction::Map, map_size, result_map));
        ret.push((RenderFunction::Profile, profile_size, result_profile));
        ret.iter()
            .map(|(function, size, result)| {
                debug_assert_eq!(result.parameters.function, function.clone());
                let points = result.rendered_input_points_for_table();
                RenderOutput {
                    svg: result.svg.clone(),
                    render_input: RenderInput {
                        kinds: kinds.clone(),
                        function: function.clone(),
                        size: (size.width, size.height),
                    },
                    error: None,
                    waypoints: waypoint::table(&data, &points),
                }
            })
            .collect()
    }

    pub fn segment_statistics(&self, segment: &Segment) -> SegmentStatistics {
        self.make_segment_data(segment).statistics()
    }

    pub fn statistics(&self) -> SegmentStatistics {
        self.segment_statistics(&self.trackSegment())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        math::IntegerSize2D,
        point_collection::{self, Kind, Kinds},
        testhelpers::load_backend_data,
        wheel,
    };
    static START_TIME: &'static str = "1985-04-12T06:05:00.00Z";
    static BLACK_FOREST: &'static str = "data/blackforest.gpx";

    #[tokio::test]
    async fn svg_segment_wheel() {
        let _ = env_logger::try_init();
        let mut backend = load_backend_data(BLACK_FOREST).await;
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((3_000) as f64);
        parameters.segment_length = 55000f64;
        parameters.segment_overlap = 5000f64;
        backend.set_parameters(&parameters);
        let reffilename = std::format!("data/ref/segment-wheel.svg");
        let data = if std::fs::exists(&reffilename).unwrap() {
            std::fs::read_to_string(&reffilename).unwrap()
        } else {
            String::new()
        };
        let track_segment = backend.trackSegment();
        let sgdata = backend.make_segment_data(&track_segment);
        let segments = backend.segments();
        let time_parameters = backend.time_parameters();
        let mut model = wheel::model::WheelModel::new(&time_parameters);
        model.add_pages(&segments);
        model.add_points(&sgdata, &point_collection::allkinds());
        let result = wheel::render(&IntegerSize2D::new(400, 400), &model);

        let tmpfilename = std::format!("/tmp/segment-wheel.svg");
        std::fs::write(&tmpfilename, result.svg.clone()).unwrap();
        if data != result.svg {
            println!("test failed: {} {}", tmpfilename, reffilename);
            assert!(false);
        }
    }

    #[tokio::test]
    async fn test_get_waypoints() {
        let _ = env_logger::try_init();
        let backend = load_backend_data(BLACK_FOREST).await;
        let fseg = backend.trackSegment();
        let seg = backend.make_segment_data(&fseg);
        let controls = seg.controls();
        let len = controls.len();
        assert!(len > 0);
        let kinds = Kinds::from([Kind::Controls]);
        let waypoints = backend.get_waypoints(&fseg, &kinds);
        assert!(!waypoints.is_empty());
        for waypoint in waypoints {
            log::info!("gpx name={}", waypoint.info.unwrap().gpx_name);
        }
    }

    #[tokio::test]
    async fn gpx() {
        let _ = env_logger::try_init();
        let mut backend = load_backend_data(&"data/synthetic.gpx").await;
        let mut parameters = backend.get_parameters();
        parameters.start_time = START_TIME.to_string();
        parameters.user_steps_options.step_distance = Some((10_000) as f64);
        backend.set_parameters(&parameters);
        let gpx = backend.generateGpx();
        let mut bad = Vec::new();
        for (filename, filecontent) in gpx {
            let tmpfilename = std::format!("/tmp/{}", filename);
            let reffilename = std::format!("data/ref/gpx/{}", filename);
            println!("test {}", reffilename);
            let data = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read(&reffilename).unwrap()
            } else {
                Vec::new()
            };
            std::fs::write(&tmpfilename, filecontent.clone()).unwrap();
            if data != filecontent {
                println!("test failed: {} {}", tmpfilename, reffilename);
                bad.push(tmpfilename);
            }
        }
        log::trace!("bad={:?}", bad);
        assert!(bad.is_empty());
    }
}
