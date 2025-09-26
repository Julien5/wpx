use crate::gpsdata;
use crate::waypoint;

#[derive(Clone)]
pub struct Segment {
    pub id: usize,
    pub range: std::ops::Range<usize>,
    pub bbox: gpsdata::ProfileBoundingBox,
}

impl Segment {
    pub fn shows_waypoint(&self, wp: &waypoint::Waypoint) -> bool {
        match wp.track_index {
            Some(index) => self.range.contains(&index),
            _ => false,
        }
    }
}

pub struct SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

impl Segment {
    pub fn new(
        id: usize,
        range: std::ops::Range<usize>,
        bbox: &gpsdata::ProfileBoundingBox,
    ) -> Segment {
        Segment {
            id,
            range: range.clone(),
            bbox: bbox.clone(),
        }
    }
}
