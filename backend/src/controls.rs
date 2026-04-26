use std::collections::BTreeSet;

use crate::{
    backend::Segment,
    inputpoint::InputPoint,
    math,
    mercator::MercatorPoint,
    parameters::Parameters,
    point_collection::{Kind, SharedPacketProvider},
    segment::SegmentData,
    track::Track,
    track_projection::{is_close_to_track, TrackProjection, TrackProjections},
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

pub fn set_control_names(controls: &mut Vec<InputPoint>) {
    let ncontrols = controls.len();
    for (index, p) in controls.iter_mut().enumerate() {
        let track_index = p.track_projections.first().unwrap().track_index;
        let control_name = if track_index == 0 {
            format!("START")
        } else if index < (ncontrols - 1) {
            format!("CP-{}", index)
        } else {
            format!("END")
        };
        p.tags.insert("name".to_string(), control_name);
    }
}

pub fn infer_controls_from_gpx_segments(
    track: &Track,
    waypoints: &Vec<InputPoint>,
) -> Vec<InputPoint> {
    struct Candidate {
        euc: MercatorPoint,
        segment_name: String,
        track_index: usize,
        waypoint_name: String,
        waypoint_description: String,
        nearest_waypoint_id: String,
    }

    // construct candidates with the *end* of each segment.
    let mut candidates: Vec<Candidate> = Vec::new();
    candidates.push(Candidate {
        euc: track.euclidean[0].clone(),
        segment_name: String::new(),
        track_index: 0,
        waypoint_name: String::new(),
        waypoint_description: String::new(),
        nearest_waypoint_id: String::new(),
    });

    let mut acc_length = 0;
    for part in &track.parts {
        acc_length += part.length;
        assert!(acc_length <= track.len());
        candidates.push(Candidate {
            euc: track.euclidean[acc_length - 1].clone(),
            segment_name: part.name.clone(),
            track_index: acc_length - 1,
            waypoint_name: String::new(),
            waypoint_description: String::new(),
            nearest_waypoint_id: String::new(),
        });
    }
    debug_assert_eq!(candidates.len(), track.parts.len() + 1);
    debug_assert!(candidates.len() > 0);

    let tree = RTree::bulk_load(waypoints.to_vec());
    let mut ret = Vec::new();
    let maxdist = 200f64;
    for mut candidate in candidates {
        let point = candidate.euc.clone();
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
                let distance =
                    math::distance2(&neighbor.euclidean.point2d(), &point.point2d()).sqrt();
                if distance < maxdist {
                    id = neighbor.gpxwaypoint_id();
                    name = neighbor.name();
                    description = neighbor.description();
                }
                (name, description, id)
            }
            None => (String::new(), String::new(), String::new()),
        };
        ret.push((
            candidate.track_index,
            InputPoint::create_control_on_track(
                track,
                TrackProjection::at_track_index(track, candidate.track_index),
                &candidate.segment_name,
                &candidate.waypoint_name,
                &candidate.waypoint_description,
                &candidate.nearest_waypoint_id,
            ),
        ));
    }
    debug_assert!(ret.is_sorted_by_key(|(index, _)| *index));
    ret.sort_by_key(|(index, _)| *index);
    let mut ret = ret.iter().map(|(_, w)| w.clone()).collect();
    set_control_names(&mut ret);
    ret
}

fn find_closest(projections: &TrackProjections, target_track_index: usize) -> TrackProjection {
    let target = target_track_index as f64;
    projections
        .iter()
        .min_by(|a, b| {
            let da = (a.track_floating_index - target).abs();
            let db = (b.track_floating_index - target).abs();
            da.total_cmp(&db)
        })
        .unwrap()
        .clone()
}

pub fn add_control_at_waypoint(
    track: &Track,
    controls: Vec<InputPoint>,
    waypoint: &Waypoint,
) -> Vec<InputPoint> {
    let mut ret = controls.clone();
    // we must recompute the projections because we mussing the floating track index.
    let mut position = InputPoint::from_wgs84(
        &waypoint.wgs84,
        &waypoint.euclidean,
        waypoint.origin.clone(),
    );
    track.project_point(&mut position);
    // now select the projection that is the closest to waypoint.track_index
    let projection = find_closest(&position.track_projections, waypoint.track_index.unwrap());
    let new = InputPoint::create_control_on_track(
        track,
        projection,
        &"",
        &waypoint.name,
        &waypoint.description,
        &waypoint.id,
    );
    ret.push(new);
    ret.sort_by(|a, b| {
        debug_assert!(a.track_projections.len() == 1);
        debug_assert!(b.track_projections.len() == 1);
        a.track_projections
            .first()
            .unwrap()
            .track_floating_index
            .total_cmp(&b.track_projections.first().unwrap().track_floating_index)
    });
    set_control_names(&mut ret);
    ret
}

pub fn remove_control_at_waypoint(
    controls: Vec<InputPoint>,
    waypoint: &Waypoint,
) -> Vec<InputPoint> {
    let mut ret = controls.clone();
    ret.retain(|control| {
        // We should not remove control that are not associated with a waypoint
        // because we cannot re-create them (since there is no waypoint to create
        // them from).
        control.track_projections.first().unwrap().track_index != waypoint.track_index.unwrap()
            || control.tags.get("nearest_waypoint_id").unwrap().is_empty()
    });
    set_control_names(&mut ret);
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
        Kind::CutOff => {
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
            nearest_osm_id: selected.gpxwaypoint_id(),
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
    // add the start of the track
    ret.insert(
        0,
        InputPoint::create_control_on_track(
            &track,
            TrackProjection::at_track_index(&track, 0),
            &"",
            &"",
            &"",
            &"",
        ),
    );
    // add the end of the track
    ret.push(InputPoint::create_control_on_track(
        &track,
        TrackProjection::at_track_index(&track, track.len() - 1),
        &"",
        &"",
        &"",
        &"",
    ));
    set_control_names(&mut ret);
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
        assert_eq!(controls.len(), 7);
        for k in 0..=4 {
            log::trace!("k={} => {}", k, controls[k].name());
        }
        assert!(controls[0].name().contains("START"));
        assert!(controls[1].name().contains("CP-1"));
        assert!(controls[2].name().contains("CP-2"));
        assert!(controls[3].name().contains("CP-3"));
        assert!(controls[4].name().contains("CP-4"));
        assert!(controls[5].name().contains("CP-5"));
        assert!(controls[6].name().contains("END"));
    }

    #[tokio::test]
    async fn controls_infer_self() {
        let _ = env_logger::try_init();
        use crate::controls::*;
        let gpxdata = read("data/ref/roland.gpx");
        let track = Track::from_tracks(&gpxdata.tracks).unwrap();
        let controls = infer_controls_from_gpx_segments(&track, &gpxdata.waypoints);
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        assert_eq!(controls.len(), 5);
        assert!(controls[0].name().contains("START"));
        assert!(controls[1].name().contains("CP-1"));
        assert!(controls[2].name().contains("CP-2"));
        assert!(controls[3].name().contains("CP-3"));
        assert!(controls[4].name().contains("END"));
    }

    async fn get_controls_from_osm(filename: &str) -> Vec<InputPoint> {
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
        let controls = get_controls_from_osm("data/blackforest.gpx").await;
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        for c in &controls {
            log::info!("c={} {}", c.name(), c.description());
        }
        assert_eq!(controls.len(), 5);
        assert!(controls[1].name().contains("CP-1"));
        assert!(controls[1].description().contains("Furtwangen"));
        assert!(controls[2].name().contains("CP-2"));
        assert!(controls[2].description().contains("Haslach"));
        assert!(controls[3].name().contains("CP-3"));
        assert!(controls[3].description().contains("Forbach"));
        assert!(controls[4].name().contains("END"));
        assert!(controls[0].name().contains("START"));
    }

    #[tokio::test]
    async fn controls_infer_sectors_2() {
        let _ = env_logger::try_init();
        let controls = get_controls_from_osm("data/ref/roland-nowaypoints.gpx").await;
        assert!(!controls.is_empty());
        for control in &controls {
            log::info!("found:{}", control.name());
        }
        for c in &controls {
            log::info!("c={} {}", c.name(), c.description());
        }
        assert_eq!(controls.len(), 5);
        assert!(controls[1].name().contains("CP-1"));
        assert!(controls[1].description().contains("Wangen"));
        assert!(controls[2].name().contains("CP-2"));
        assert!(controls[2].description().contains("Isny"));
        assert!(controls[3].name().contains("CP-3"));
        assert!(controls[3].description().contains("Bad Waldsee"));
        assert!(controls[4].name().contains("END"));
    }
}
