#![allow(non_snake_case)]

use flutter_rust_bridge::frb;

// must be exported for mirroring Segment.
pub use std::ops::Range;
pub use tracks::backend::gpsdata::UTMPoint;
pub use tracks::backend::Segment;
pub use tracks::backend::WayPoint;

#[frb(opaque)]
pub struct Bridge {
    backend: tracks::backend::Backend,
}

#[frb(mirror(Segment))]
pub struct _Segment {
    pub id: usize,
    pub range: Range<usize>,
}

#[frb(mirror(WayPoint))]
pub struct _WayPoint {
    wgs84: (f64, f64, f64),
    utm: UTMPoint,
    distance: f64,
    elevation: f64,
    inter_distance: f64,
    inter_elevation_gain: f64,
    inter_slope: f64,
    name: String,
    time: i64, // seconds since epoch
    track_index: usize,
}

use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

impl Bridge {
    pub fn create() -> Bridge {
        Bridge {
            backend: tracks::backend::Backend::new("/tmp/track.gpx"),
        }
    }
    #[frb(sync)]
    pub fn adjustEpsilon(&mut self, eps: f32) {
        self.backend.adjustEpsilon(eps);
    }
    #[frb(sync)]
    pub fn setStartTime(&mut self, seconds_since_epoch: i64) {
        self.backend.setStartTime(seconds_since_epoch)
    }
    #[frb(sync)]
    pub fn setSpeed(&mut self, meter_per_second: f64) {
        self.backend.setSpeed(meter_per_second)
    }
    #[frb(sync)]
    pub fn getWayPoints(&mut self) -> Vec<WayPoint> {
        self.backend.get_waypoints()
    }
    #[frb(sync)]
    pub fn elevation_gain(&mut self, from: usize, to: usize) -> f64 {
        self.backend.elevation_gain(from, to)
    }
    pub async fn renderTrack(&mut self) -> String {
        self.backend.render_track()
    }
    pub async fn renderWaypoints(&mut self) -> String {
        self.backend.render_waypoints()
    }
    pub async fn renderSegmentTrack(&mut self, segment: &Segment) -> String {
        //let delay = std::time::Duration::from_millis(50);
        //std::thread::sleep(delay);
        self.backend.render_segment_track(&segment)
    }
    pub async fn renderSegmentWaypoints(&mut self, segment: &Segment) -> String {
        //let delay = std::time::Duration::from_millis(50);
        //std::thread::sleep(delay);
        self.backend.render_segment_waypoints(&segment)
    }
    #[frb(sync)]
    pub fn renderSegmentWaypointsSync(&mut self, segment: &Segment) -> String {
        self.backend.render_segment_waypoints(&segment)
    }
    #[frb(sync)]
    pub fn epsilon(&self) -> f32 {
        self.backend.epsilon()
    }

    #[frb(sync)]
    pub fn segments(&self) -> Vec<Segment> {
        self.backend.segments()
    }
}

pub async fn svgCircle() -> String {
    sleep(Duration::from_secs(1)).await;
    let s = r#"<svg height="100" width="100" xmlns="http://www.w3.org/2000/svg">
  <circle r="45" cx="50" cy="50" fill="red" />
</svg>"#;
    String::from_str(s).unwrap()
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
