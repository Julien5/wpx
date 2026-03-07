use std::{collections::BTreeMap, sync::Arc};

use crate::{
    backend::Segment,
    inputpoint::InputPoint,
    math,
    mercator::MercatorPoint,
    parameters::Parameters,
    point_collection::{Kind, SharedPacketProvider},
    segment::SegmentData,
    track::Track,
    track_projection::{is_close_to_track, TrackProjection},
    wheel::shorten::shorten_name,
};
use rstar::{RTree, AABB};

impl rstar::RTreeObject for InputPoint {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.euclidean.0, self.euclidean.1])
    }
}

impl rstar::PointDistance for InputPoint {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let p1 = self.euclidean.point2d();
        let p2 = math::Point2D::new(point[0], point[1]);
        math::distance2(&p1, &p2)
    }

    fn contains_point(&self, _point: &[f64; 2]) -> bool {
        false
    }
}

pub fn infer_controls_from_gpx_segments(
    track: &Track,
    waypoints: &Vec<InputPoint>,
) -> Vec<InputPoint> {
    let parts = &track.parts;
    if parts.len() == 1 {
        log::info!("cannot infer control from a single track/segment");
        return Vec::new();
    }

    struct Candidate {
        position: MercatorPoint,
        segment_name: String,
        segment_index: usize,
        waypoint_name: String,
        waypoint_description: String,
    }

    // construct candidates with the *end* of each segment.
    let mut candidates: BTreeMap<usize, Candidate> = BTreeMap::new();
    let mut part_end_index = 0;
    for (_index, part) in parts.iter().enumerate() {
        part_end_index += part.length;
        if part_end_index == track.len() {
            break;
        }
        assert!(part_end_index <= track.len());
        candidates.insert(
            part_end_index,
            Candidate {
                position: track.euclidean[part_end_index - 1].clone(),
                segment_name: part.name.clone(),
                segment_index: _index,
                waypoint_name: String::new(),
                waypoint_description: String::new(),
            },
        );
    }
    assert_eq!(candidates.len(), parts.len() - 1);
    assert!(candidates.len() > 0);

    let tree = RTree::bulk_load(waypoints.to_vec());
    let mut ret = Vec::new();
    let maxdist = 200f64;
    for (track_index, mut candidate) in candidates {
        let point = candidate.position.clone();
        let nearest = tree.nearest_neighbor(&[point.0, point.1]);
        (candidate.waypoint_name, candidate.waypoint_description) = match nearest {
            Some(neighbor) => {
                let mut name = String::new();
                let mut description = String::new();
                if math::distance2(&neighbor.euclidean.point2d(), &point.point2d()).sqrt() < maxdist
                {
                    if !neighbor.name().is_empty() {
                        name = neighbor.name();
                    }
                    if !neighbor.description().is_empty() {
                        description = neighbor.description();
                    }
                    log::trace!("waypoint {} near end of {}", name, candidate.segment_name);
                } else {
                    log::trace!("no waypoint near end of {}", candidate.segment_name);
                }
                (name, description)
            }
            None => {
                log::trace!("no waypoint near end of {}", candidate.segment_name);
                (String::new(), String::new())
            }
        };
        ret.push((
            track_index,
            InputPoint::create_control_on_track(
                track,
                TrackProjection::at_track_index(track, track_index),
                candidate.segment_index + 1,
                &candidate.segment_name,
                &candidate.waypoint_name,
                &candidate.waypoint_description,
            ),
        ));
    }
    debug_assert!(ret.is_sorted_by_key(|(index, _)| *index));
    ret.sort_by_key(|(index, _)| *index);
    for (index, point) in &ret {
        log::trace!("control index {} name:{} ", index, point.name());
    }
    ret.iter().map(|(_, w)| w.clone()).collect()
}

pub fn make_controls_with_waypoints(track: &Track, gpxpoints: &Vec<InputPoint>) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    let maxdist = 100f64;
    log::trace!("{} gpx waypoints", gpxpoints.len());
    for p in gpxpoints {
        assert!(p.track_projections.len() >= 1);
    }
    let projections = InputPoint::flatten_projections(&gpxpoints);
    assert!(projections.len() >= gpxpoints.len());
    for (index, projection) in projections {
        let point = &gpxpoints[index];
        let segment_name = String::new();
        if point.distance_to_track() < maxdist {
            let control = InputPoint::create_control_on_track(
                track,
                projection,
                ret.len() + 1,
                &segment_name,
                &point.name(),
                &point.description(),
            );
            ret.push(control);
            log::trace!("pushed {}", point.name());
        } else {
            log::info!("point {} is too far from track", point.name());
        }
    }
    ret
}

fn control_point_goodness(point: &InputPoint) -> i32 {
    let min_population = match point.kind() {
        Kind::Cities => 10000,
        Kind::Villages => 1000,
        Kind::Hamlets => 100,
        _ => 0,
    };
    match point.kind() {
        Kind::UserStep => {
            return i32::MIN;
        }
        Kind::GPXWaypoints | Kind::Controls => {
            return i32::MAX;
        }
        _ => {
            let population = point.population().unwrap_or(min_population);
            if population > 0 {
                return population;
            }
            return 0;
        }
    };
}

pub fn has_startend_controls(track: &Track, controls: &Vec<InputPoint>) -> (bool, bool) {
    if controls.is_empty() {
        return (false, false);
    }
    let mut indices: Vec<_> = InputPoint::flatten_projections(controls)
        .iter()
        .map(|(_, proj)| proj.track_index)
        .collect();
    indices.sort();
    let maxdist = 1000f64;
    let first = indices.first().unwrap();
    let has_start = track.distance(*first) <= maxdist;
    let last = indices.last().unwrap();
    let has_end = (track.total_distance() - track.distance(*last)).abs() <= maxdist;
    (has_start, has_end)
}

#[allow(dead_code)]
pub fn insert_start_end_controls(track: &Track, controls: &mut Vec<InputPoint>) {
    let length = track.len();
    let (has_start, has_end) = has_startend_controls(track, controls);

    if !has_start {
        let start = InputPoint::create_control_on_track(
            track,
            TrackProjection::at_track_index(track, 0),
            controls.len() + 1,
            "",
            "start",
            "start",
        );
        controls.push(start.clone());
    }
    if !has_end {
        let end = InputPoint::create_control_on_track(
            track,
            TrackProjection::at_track_index(track, length - 1),
            controls.len() + 1,
            "End",
            "end",
            "end",
        );
        controls.push(end.clone());
    }
}

pub fn make_controls_with_osm(
    track: &Arc<Track>,
    packet_provider: SharedPacketProvider,
) -> Vec<InputPoint> {
    let total = track.total_distance();
    let track_distance_km = total / 1000f64;
    let n_controls = ((track_distance_km / 70f64).ceil() as usize).max(4);
    let step_size = (total / n_controls as f64).ceil();
    // no control in first 10 and the last 10 kms.
    let mut start = 0f64;
    let mut segments = Vec::new();
    loop {
        let end = start + step_size;
        let range = track.subrange(start, end);
        if range.is_empty() {
            break;
        }
        let segment = Segment {
            id: segments.len() as i32,
            start,
            end,
        };
        let data = SegmentData::new(
            &segment,
            track.clone(),
            packet_provider.clone(),
            Parameters::default(),
        );
        segments.push(data);
        start = end;
    }

    struct Control {
        index: usize,
        osm_name: String,
    }

    let mut proto = Vec::new();
    let margin = 10_000f64;
    let mut last_control_distance = 0f64;
    for segment in &mut segments {
        let mut points = segment.potential_controls();
        log::trace!("segment id={} before={}", segment.id(), points.len());
        points.retain(|w| {
            let total_distance = track.total_distance();
            if w.track_projections.is_empty() {
                return false;
            }
            assert!(!w.track_projections.is_empty());
            for proj in &w.track_projections {
                let distance = proj.distance_on_track_to_projection;
                let is_far_from_last = distance > last_control_distance + margin;
                let is_far_from_end = distance < total_distance - margin;
                let good = is_close_to_track(w) && is_far_from_last && is_far_from_end;
                if good {
                    return true;
                }
            }
            false
        });
        log::trace!("segment id={} after={}", segment.id(), points.len());
        if points.is_empty() {
            continue;
        }
        points.sort_by_key(|w| -control_point_goodness(&w));
        let selected = points.first().unwrap().clone();
        // In case the selected point has several projection, take the first one on this segment.
        // Taking the first one is arbitrary.
        let indices_on_segment: Vec<_> = selected
            .track_projections
            .iter()
            .map(|proj| proj.track_index)
            .filter(|index| segment.range().contains(index))
            .collect();
        if indices_on_segment.len() > 1 {
            log::warn!("{} ambiguous projections", indices_on_segment.len());
        }
        let index = *indices_on_segment.first().unwrap();
        let name = selected.name();
        proto.push(Control {
            index,
            osm_name: name,
        });
        last_control_distance = selected
            .track_projections
            .first()
            .unwrap()
            .distance_on_track_to_projection;
    }
    proto.sort_by_key(|c| c.index);
    let mut ret = Vec::new();
    for k in 0..proto.len() {
        let p = &proto[k];
        let segment_name = "";
        let waypoint_name = shorten_name(&p.osm_name);
        let waypoint_description = p.osm_name.clone();
        let w = InputPoint::create_control_on_track(
            &track,
            TrackProjection::at_track_index(track, p.index),
            ret.len() + 1,
            &segment_name,
            &waypoint_name,
            &waypoint_description,
        );
        ret.push(w);
    }
    ret
}

#[cfg(test)]
mod tests {
    use crate::{
        event, gpsdata::GpxData, inputpoint::InputPoint, osm, parameters,
        point_collection::PacketProvider,
    };

    fn read(filename: &str) -> GpxData {
        use crate::gpsdata;
        let mut f = std::fs::File::open(filename).unwrap();
        let mut content = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut content).unwrap();
        let data = gpsdata::GpxData::read_content(&content).unwrap();
        let ordered = parameters::karl_order(&data.track_parts());
        let indices: Vec<_> = ordered.iter().map(|part| part.part_index).collect();
        data.reorder(&indices)
    }

    #[tokio::test]
    async fn controls_infer_brevet() {
        let _ = env_logger::try_init();
        use crate::controls::*;
        let gpxdata = read("data/ref/karl-400.gpx");
        let track = Track::from_tracks(&gpxdata.tracks).unwrap();
        let controls = infer_controls_from_gpx_segments(&track, &gpxdata.waypoints);
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        assert_eq!(controls.len(), 5);
        for k in 0..=4 {
            log::trace!("k={} => {}", k, controls[k].name());
        }
        assert!(controls[0].name().contains("K1"));
        assert!(controls[1].name().contains("K2"));
        assert!(controls[2].name().contains("K3"));
        assert!(controls[3].name().contains("K4"));
        assert!(controls[4].name().contains("K5"));
    }

    #[tokio::test]
    async fn controls_infer_self() {
        let _ = env_logger::try_init();
        use crate::controls::*;
        let gpxdata = read("data/blackforest.gpx");
        let track = Track::from_tracks(&gpxdata.tracks).unwrap();
        let controls = infer_controls_from_gpx_segments(&track, &gpxdata.waypoints);
        assert!(controls.is_empty());
        let mut gpxpoints = gpxdata.waypoints;
        for p in &mut gpxpoints {
            track.project_point(p);
        }
        let controls = make_controls_with_waypoints(&track, &gpxpoints);
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        assert_eq!(controls.len(), 4);
        assert!(controls[0].name().contains("K1"));
        assert!(controls[1].name().contains("K2"));
        assert!(controls[2].name().contains("K3"));
        assert!(controls[3].name().contains("K4"));
    }

    async fn get_controls(filename: &str) -> Vec<InputPoint> {
        let _ = env_logger::try_init();
        use crate::controls::*;
        let gpxdata = read(filename);
        let track = Arc::new(Track::from_tracks(&gpxdata.tracks).unwrap());

        let b: event::SenderHandler = Box::new(event::ConsoleEventSender {});
        let logger = std::sync::RwLock::new(Some(b));
        let mut osmpoints = osm::download_for_track(&track, &logger).await.unwrap();
        track.project_map(&mut osmpoints);

        let mut provider = PacketProvider::new();
        provider.collection.import_osm(&osmpoints.as_vector());
        let provider = SharedPacketProvider::new(provider.into());

        make_controls_with_osm(&track, provider)
    }

    #[tokio::test]
    async fn controls_infer_sectors() {
        let _ = env_logger::try_init();
        let controls = get_controls("data/blackforest.gpx").await;
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        assert_eq!(controls.len(), 4);
        for c in &controls {
            log::info!("c={} {}", c.name(), c.description());
        }
        assert!(controls[0].name().contains("K1"));
        assert!(controls[0].description().contains("Furtwangen"));
        assert!(controls[1].name().contains("K2"));
        assert!(controls[1].description().contains("Haslach"));
        assert!(controls[2].name().contains("K3"));
        assert!(controls[2].description().contains("Forbach"));
    }

    #[tokio::test]
    async fn controls_infer_sectors_2() {
        let _ = env_logger::try_init();
        let controls = get_controls("data/ref/roland-nowaypoints.gpx").await;
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        for c in &controls {
            log::info!("c={} {}", c.name(), c.description());
        }
        assert_eq!(controls.len(), 4);
        assert!(controls[0].name().contains("K1"));
        assert!(controls[0].description().contains("Wangen"));
        assert!(controls[1].name().contains("K2"));
        assert!(controls[1].description().contains("Isny"));
    }
}
