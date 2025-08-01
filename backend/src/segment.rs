use crate::gpsdata;
use crate::svgprofile;
use crate::waypoint;
use crate::waypoints_table;

#[derive(Clone)]
pub struct Segment {
    pub id: usize,
    pub range: std::ops::Range<usize>,
    pub profile: svgprofile::Profile,
}

impl Segment {
    pub fn shows_waypoint(&self, wp: &waypoint::Waypoint) -> bool {
        self.profile.shows_waypoint(wp)
    }
    pub fn show_waypoints_in_table(&self, waypoints: &Vec<waypoint::Waypoint>) -> Vec<usize> {
        waypoints_table::show_waypoints_in_table(&waypoints, &self.profile.bbox)
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
            profile: svgprofile::Profile::init(&bbox),
        }
    }
}
