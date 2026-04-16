use std::collections::{BTreeMap, BTreeSet};

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
    waypoint::Waypoint,
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
        nearest_waypoint_id: String,
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
                nearest_waypoint_id: String::new(),
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
        (
            candidate.waypoint_name,
            candidate.waypoint_description,
            candidate.nearest_waypoint_id,
        ) = match nearest {
            Some(neighbor) => {
                let mut name = String::new();
                let mut description = String::new();
                let mut id = String::new();
                if math::distance2(&neighbor.euclidean.point2d(), &point.point2d()).sqrt() < maxdist
                {
                    id = neighbor.id();
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
                (name, description, id)
            }
            None => {
                log::trace!("no waypoint near end of {}", candidate.segment_name);
                (String::new(), String::new(), String::new())
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
                &candidate.nearest_waypoint_id,
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

pub fn add_control_at_waypoint(
    track: &Track,
    controls: Vec<InputPoint>,
    waypoint: &Waypoint,
) -> Vec<InputPoint> {
    let mut ret = controls.clone();
    ret.sort_by_key(|p| p.track_projections.first().unwrap().track_index);
    let index = {
        let mut k = 0;
        for c in &ret {
            if c.track_projections.first().unwrap().track_index > waypoint.track_index.unwrap() {
                break;
            }
            k += 1;
        }
        k
    };
    let projection = TrackProjection::at_track_index(track, waypoint.track_index.unwrap());
    let new = InputPoint::create_control_on_track(
        track,
        projection,
        0,
        &"foo",
        &waypoint.name,
        &waypoint.description,
        &waypoint.id,
    );
    ret.insert(index, new);
    for (index, p) in ret.iter_mut().enumerate() {
        p.tags
            .insert("control_index".to_string(), format!("K{}", index + 1));
    }
    ret
}

pub fn remove_control_at_waypoint(
    controls: Vec<InputPoint>,
    waypoint: &Waypoint,
) -> Vec<InputPoint> {
    let mut ret = controls.clone();
    ret.retain(|p| {
        p.track_projections.first().unwrap().track_index != waypoint.track_index.unwrap()
    });
    for (index, p) in ret.iter_mut().enumerate() {
        p.tags
            .insert("control_index".to_string(), format!("K{}", index + 1));
    }
    ret
}

fn _control_point_goodness(point: &InputPoint) -> i32 {
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
            "",
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
            "",
        );
        controls.push(end.clone());
    }
}

pub fn _select_osm_points_on_segment(
    segment: &SegmentData,
    start: f64,
    end: f64,
) -> Vec<InputPoint> {
    let mut points = segment.potential_controls();
    points.retain(|w| {
        if w.track_projections.is_empty() {
            return false;
        }
        assert!(!w.track_projections.is_empty());
        for proj in &w.track_projections {
            let distance = proj.distance_on_track_to_projection;
            let is_far_from_last = distance > start;
            let is_far_from_end = distance < end;
            let good = is_close_to_track(w) && is_far_from_last && is_far_from_end;
            if good {
                return true;
            }
        }
        false
    });
    points.sort_by_key(|w| -_control_point_goodness(&w));
    points
}

pub fn _make_with_osm(
    bigsegment: &SegmentData,
    packet_provider: SharedPacketProvider,
    typical_distance: f64,
    newkind: &Kind,
) -> Vec<InputPoint> {
    let track = &bigsegment.track;
    let total_distance = bigsegment.end() - bigsegment.start();
    let n_controls = ((total_distance / typical_distance).ceil() as usize).max(4);
    let step_size = (total_distance / n_controls as f64).ceil();
    let mut start = bigsegment.start();
    let mut subsegments = Vec::new();
    loop {
        let end = start + step_size;
        let range = bigsegment.track.subrange(start, end);
        if end > bigsegment.end() || range.is_empty() {
            break;
        }
        let subsegment = Segment {
            id: subsegments.len() as i32,
            start,
            end: end.min(bigsegment.end()),
        };
        let data = SegmentData::new(
            &subsegment,
            track.clone(),
            packet_provider.clone(),
            Parameters::default(),
        );
        subsegments.push(data);
        start = end;
    }

    struct ProtoPoint {
        index: usize,
        osm_name: String,
        nearest_osm_id: String,
    }

    // no control in first 10 and the last 10 kms.
    let mut proto = Vec::new();
    let margin = typical_distance * 0.1;
    let mut last_control_distance = 0f64;
    for subsegment in &subsegments {
        let points = _select_osm_points_on_segment(
            &subsegment,
            last_control_distance + margin,
            bigsegment.start() + total_distance - margin,
        );
        if points.is_empty() {
            continue;
        }
        let selected = points.first().unwrap().clone();
        // In case the selected point has several projection, take the first one on this segment.
        // Taking the first one is arbitrary.
        let indices_on_segment: Vec<_> = selected
            .track_projections
            .iter()
            .map(|proj| proj.track_index)
            .filter(|index| subsegment.range().contains(index))
            .collect();
        if indices_on_segment.len() > 1 {
            log::warn!("{} ambiguous projections", indices_on_segment.len());
        }
        let index = *indices_on_segment.first().unwrap();
        let name = selected.name();
        proto.push(ProtoPoint {
            index,
            osm_name: name,
            nearest_osm_id: selected.id(),
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
        let proj = TrackProjection::at_track_index(&track, p.index);
        let wgs84 = &track.wgs84[proj.track_index];
        let eucli = &track.euclidean[proj.track_index];
        let w = match newkind {
            &Kind::Controls => InputPoint::create_control_on_track(
                &track,
                proj,
                ret.len() + 1,
                &segment_name,
                &waypoint_name,
                &waypoint_description,
                &p.nearest_osm_id,
            ),
            _ => {
                let mut i = InputPoint::from_gpx(wgs84, eucli, &Some(p.osm_name.clone()), &None);
                i.track_projections = BTreeSet::from([{ proj }]);
                i
            }
        };
        ret.push(w);
    }
    ret
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio_util::sync::CancellationToken;

    use crate::{
        event,
        gpsdata::GpxData,
        inputpoint::InputPoint,
        osm::{self, DownloadSideData},
        parameters,
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
        let gpxdata = read("data/ref/roland.gpx");
        let track = Track::from_tracks(&gpxdata.tracks).unwrap();
        let controls = infer_controls_from_gpx_segments(&track, &gpxdata.waypoints);
        assert!(controls.is_empty());
        let mut gpxpoints = gpxdata.waypoints;
        for p in &mut gpxpoints {
            track.project_point(p);
        }
        let controls = infer_controls_from_gpx_segments(&track, &gpxpoints);
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
        let token = CancellationToken::new();
        let side = DownloadSideData {
            logger: &logger,
            cancel_token: &token,
        };
        let mut osmpoints = osm::download_for_track(&track, &side).await.unwrap();
        track.project_map(&mut osmpoints);

        let mut provider = PacketProvider::new();
        provider.collection.import_osm(&osmpoints.as_vector());
        let provider = SharedPacketProvider::new(provider.into());

        let segment = SegmentData::new(
            &Segment {
                id: -1,
                start: 0f64,
                end: track.total_distance(),
            },
            track.clone(),
            provider.clone(),
            Parameters::default(),
        );
        _make_with_osm(&segment, provider, 70_000f64, &Kind::Controls)
    }

    #[tokio::test]
    async fn controls_infer_sectors_1() {
        let _ = env_logger::try_init();
        let controls = get_controls("data/blackforest.gpx").await;
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        for c in &controls {
            log::info!("c={} {}", c.name(), c.description());
        }
        assert_eq!(controls.len(), 3);
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
        assert_eq!(controls.len(), 3);
        assert!(controls[0].name().contains("K1"));
        assert!(controls[0].description().contains("Wangen"));
        assert!(controls[1].name().contains("K2"));
        assert!(controls[1].description().contains("Isny"));
        assert!(controls[2].name().contains("K3"));
        assert!(controls[2].description().contains("Bad Waldsee"));
    }
}
