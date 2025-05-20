#![allow(non_snake_case)]

use flutter_rust_bridge::frb;

#[frb(opaque)]
pub struct Frontend {
    backend: tracks::backend::Backend,
}

#[derive(Clone)]
#[frb(opaque)]
pub struct FSegment {
    _backend: tracks::backend::Segment,
}

use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

impl Frontend {
    pub fn create() -> Frontend {
        Frontend {
            backend: tracks::backend::Backend::new(),
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
    pub async fn renderSegmentTrack(&mut self, segment: &FSegment) -> String {
        self.backend.render_segment_track(&segment._backend)
    }
    pub async fn renderSegmentWaypoints(&mut self, segment: &FSegment) -> String {
        self.backend.render_segment_waypoints(&segment._backend)
    }
    #[frb(sync)]
    pub fn renderSegmentWaypointsSync(&mut self, segment: &FSegment) -> String {
        self.backend.render_segment_waypoints(&segment._backend)
    }
    #[frb(sync)]
    pub fn epsilon(&self) -> f32 {
        self.backend.epsilon()
    }

    #[frb(sync)]
    pub fn segments(&self) -> Vec<FSegment> {
        let segb = self.backend.segments();
        let mut ret = Vec::new();
        for s in segb {
            let f = FSegment {
                _backend: s.clone(),
            };
            ret.push(f);
        }
        ret
    }
}

impl FSegment {
    #[frb(sync)]
    pub fn id(&self) -> usize {
        self._backend.id
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
