#![allow(non_snake_case)]

use flutter_rust_bridge::frb;

// must be exported for mirroring Segment.
pub use std::ops::Range;
pub use tracks::backend::WayPoint;
pub use tracks::gpsdata::WaypointOrigin;
pub use tracks::utm::UTMPoint;

pub use tracks::backend::Segment as SegmentImplementation;

#[frb(opaque)]
pub struct Bridge {
    backend: tracks::backend::Backend,
}

#[frb(opaque)]
pub struct Segment {
    _impl: SegmentImplementation,
}

impl Segment {
    pub fn create(d: SegmentImplementation) -> Segment {
        Segment { _impl: d }
    }

    #[frb(sync)]
    pub fn id(&self) -> usize {
        self._impl.id
    }

    #[frb(sync)]
    pub fn shows_waypoint(&self, wp: &WayPoint) -> bool {
        self._impl.shows_waypoint(wp)
    }
}

#[frb(mirror(UTMPoint))]
pub struct _UTMPoint(pub f64, pub f64);

#[frb(mirror(WaypointOrigin))]
pub enum _WaypointOrigin {
    GPX,
    DouglasPeucker,
}

#[frb(mirror(WayPoint))]
pub struct _WayPoint {
    wgs84: (f64, f64, f64),
    utm: UTMPoint,
    origin: WaypointOrigin,
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
    pub async fn create(filename: &str) -> Bridge {
        Bridge {
            backend: tracks::backend::Backend::new(filename),
        }
    }
    pub async fn fromContent(content: &Vec<u8>) -> Bridge {
        Bridge {
            backend: tracks::backend::Backend::from_content(content),
        }
    }
    pub async fn adjustEpsilon(&mut self, eps: f32) {
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
    #[frb(sync)] //TODO: add segment parameter
    pub fn getWayPoints(&mut self) -> Vec<WayPoint> {
        self.backend.get_waypoints()
    }
    #[frb(sync)]
    pub fn elevation_gain(&mut self, from: usize, to: usize) -> f64 {
        self.backend.elevation_gain(from, to)
    }
    pub async fn renderSegmentTrack(&mut self, segment: &Segment, W: i32, H: i32) -> String {
        //let delay = std::time::Duration::from_millis(50);
        //std::thread::sleep(delay);
        self.backend.render_segment_track(&segment._impl, (W, H))
    }
    pub async fn renderSegmentWaypoints(&mut self, segment: &Segment, W: i32, H: i32) -> String {
        //let delay = std::time::Duration::from_millis(50);
        //std::thread::sleep(delay);
        println!("{}x{}", W, H);
        self.backend
            .render_segment_waypoints(&segment._impl, (W, H))
    }
    #[frb(sync)]
    pub fn renderSegmentWaypointsSync(&mut self, segment: &Segment, W: i32, H: i32) -> String {
        self.backend
            .render_segment_waypoints(&segment._impl, (W, H))
    }
    #[frb(sync)]
    pub fn epsilon(&self) -> f32 {
        self.backend.epsilon()
    }

    #[frb(sync)]
    pub fn segments(&self) -> Vec<Segment> {
        let S = self.backend.segments();
        let mut ret = Vec::new();
        for s in S {
            ret.push(Segment::create(s));
        }
        ret
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
