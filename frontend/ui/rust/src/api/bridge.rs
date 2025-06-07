#![allow(non_snake_case)]

use flutter_rust_bridge::frb;

// must be exported for mirroring Segment.
pub use std::ops::Range;
pub use tracks::backend::Segment;

#[frb(opaque)]
pub struct Bridge {
    backend: tracks::backend::Backend,
}

#[frb(mirror(Segment))]
pub struct _Segment {
    pub id: usize,
    pub range: Range<usize>,
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
    pub fn changeParameter(&mut self, eps: f32) {
        self.backend.changeParameter(eps);
    }
    pub async fn renderTrack(&mut self) -> String {
        self.backend.render_track()
    }
    pub async fn renderWaypoints(&mut self) -> String {
        self.backend.render_waypoints()
    }
    pub async fn renderSegmentTrack(&mut self, segment: &Segment) -> String {
        let delay = std::time::Duration::from_millis(50);
        std::thread::sleep(delay);
        self.backend.render_segment_track(&segment)
    }
    pub async fn renderSegmentWaypoints(&mut self, segment: &Segment) -> String {
        let delay = std::time::Duration::from_millis(50);
        std::thread::sleep(delay);
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
