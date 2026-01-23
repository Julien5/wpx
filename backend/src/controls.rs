use std::{collections::BTreeMap, sync::Arc};

use crate::{
    backend::Segment,
    inputpoint::{InputPoint, InputType, OSMType, SharedPointMaps},
    math,
    mercator::MercatorPoint,
    parameters::Parameters,
    segment::SegmentData,
    track::Track,
    track_projection::is_close_to_track,
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

fn extract_last_part(name: &String) -> String {
    name.split(|c: char| !c.is_alphanumeric()) // Split by anything not a letter/digit
        .filter(|s| !s.is_empty()) // Ignore empty strings (e.g., if it ends in '!')
        .last() // Take the very last element
        .unwrap_or("") // If nothing found, return empty string
        .to_string()
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
    }

    // construct candidates with the *end* of each segment.
    let mut candidates: BTreeMap<usize, Candidate> = BTreeMap::new();
    for index in 0..parts.len() {
        let part = &parts[index];
        if part.end < track.len() {
            candidates.insert(
                index,
                Candidate {
                    position: track.euclidean[part.end].clone(),
                    segment_name: part.name.clone(),
                },
            );
        }
    }
    assert_eq!(candidates.len(), parts.len() - 1);
    assert!(candidates.len() > 0);

    let tree = RTree::bulk_load(waypoints.to_vec());
    let mut ret = Vec::new();
    let maxdist = 200f64;
    for (index, candidate) in candidates {
        let mut name = extract_last_part(&candidate.segment_name);
        let point = candidate.position.clone();
        let mut description = String::new();
        let nearest = tree.nearest_neighbor(&[point.0, point.1]);
        match nearest {
            Some(neighbor) => {
                if math::distance2(&neighbor.euclidean.point2d(), &point.point2d()).sqrt() < maxdist
                {
                    if !neighbor.name().is_empty() {
                        name = neighbor.name();
                    }
                    if !neighbor.description().is_empty() {
                        description = neighbor.description();
                    }
                }
            }
            None => {}
        }

        ret.push(InputPoint::create_control_on_track(
            track,
            parts[index].end,
            &name,
            &description,
        ));
    }
    ret.sort_by_key(|w| w.single_track_index().unwrap_or(0));
    ret
}

pub fn make_controls_with_waypoints(track: &Track, gpxpoints: &Vec<InputPoint>) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    let maxdist = 100f64;
    for point in gpxpoints {
        let mut clone = point.clone();
        track.trees.project_single(&mut clone, &track);
        if clone.distance_to_track() < maxdist {
            let control = InputPoint::create_control_on_track(
                track,
                clone.single_track_index().unwrap(),
                &point.name(),
                &point.description(),
            );
            ret.push(control);
        } else {
            log::info!("point {} is too far from track", point.name());
        }
    }
    ret.sort_by_key(|w| w.single_track_index().unwrap_or(0));
    ret
}

fn control_point_goodness(point: &InputPoint) -> i32 {
    match point.kind() {
        InputType::UserStep => {
            return i32::MIN;
        }
        InputType::GPX | InputType::Control => {
            return i32::MAX;
        }
        InputType::OSM => {
            let min_population = match point.osmkind().unwrap() {
                OSMType::City => 10000,
                OSMType::Village => 1000,
                OSMType::Hamlet => 100,
                _ => 0,
            };
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
    let mut indices: Vec<_> = controls
        .iter()
        .map(|p| p.single_track_index().unwrap())
        .collect();
    indices.sort();
    let maxdist = 1000f64;
    let first = indices.first().unwrap();
    let has_start = track.distance(*first) <= maxdist;
    let last = indices.last().unwrap();
    let has_end = (track.total_distance() - track.distance(*last)).abs() <= maxdist;
    (has_start, has_end)
}

pub fn insert_start_end_controls(track: &Track, controls: &mut Vec<InputPoint>) {
    let length = track.len();
    let (has_start, has_end) = has_startend_controls(track, controls);
    let start = InputPoint::create_control_on_track(track, 0, "Start", "start");
    let end = InputPoint::create_control_on_track(track, length - 1, "End", "end");
    if !has_start {
        controls.push(start.clone());
    }
    if !has_end {
        controls.push(end.clone());
    }
}

pub fn make_controls_with_osm(track: &Arc<Track>, inputpoints: SharedPointMaps) -> Vec<InputPoint> {
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
            inputpoints.clone(),
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
        // it has all the osm points, not only those from the segment!
        let mut points = segment.osmpoints();
        points.retain(|w| {
            let total_distance = track.total_distance();
            if w.track_projections.is_empty() {
                return false;
            }
            let distance = w
                .track_projections
                .first()
                .unwrap()
                .distance_on_track_to_projection;
            let is_far_from_last = distance > last_control_distance + margin;
            let is_far_from_end = distance < total_distance - margin;
            is_close_to_track(w) && is_far_from_last && is_far_from_end
        });
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
        let short_name = shorten_name(&p.osm_name);
        let name = format!("K{} - {}", k + 1, short_name);
        let w = InputPoint::create_control_on_track(&track, p.index, &name, &p.osm_name);
        ret.push(w);
    }
    ret
}

#[cfg(test)]
mod tests {
    use crate::{
        event, gpsdata::GpxData, inputpoint::InputPoint, inputpoint::InputPointMaps, osm,
        track_projection::ProjectionTrees,
    };

    fn read(filename: &str) -> GpxData {
        use crate::gpsdata;
        let mut f = std::fs::File::open(filename).unwrap();
        let mut content = Vec::new();
        // read the whole file
        use std::io::prelude::*;
        f.read_to_end(&mut content).unwrap();
        gpsdata::read_content(&content).unwrap()
    }

    #[tokio::test]
    async fn controls_infer_brevet() {
        let _ = env_logger::try_init();
        use crate::controls::*;
        let gpxdata = read("data/ref/karl-400.gpx");
        let track = Track::from_tracks(&gpxdata.tracks).unwrap();
        let controls = infer_controls_from_gpx_segments(&track, &gpxdata.waypoints.as_vector());
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        assert_eq!(controls.len(), 5);
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
        let controls = infer_controls_from_gpx_segments(&track, &gpxdata.waypoints.as_vector());
        assert!(controls.is_empty());
        let controls = make_controls_with_waypoints(&track, &gpxdata.waypoints.as_vector());
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
        let mut inputpoints = BTreeMap::new();
        let mut osmpoints = osm::download_for_track(&track, &logger).await.unwrap();
        track.trees.iter_on(&mut osmpoints, &track);
        inputpoints.insert(InputType::OSM, osmpoints);
        let shared = SharedPointMaps::new(InputPointMaps { maps: inputpoints }.into());
        make_controls_with_osm(&track, shared)
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
