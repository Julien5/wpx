#![allow(non_snake_case)]

use flutter_rust_bridge::frb;

pub struct Frontend {
    backend: Box<tracks::worker::Backend>,
}

use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

impl Frontend {
    pub fn create() -> Frontend {
        Frontend {
            backend: Box::new(tracks::worker::Backend::new()),
        }
    }
    #[frb(sync)]
    pub fn changeParameter(&mut self, eps: f32) {
        self.backend.changeParameter(eps);
    }
    pub async fn svg(&self) -> String {
        sleep(Duration::from_secs(1)).await;
        self.backend.testSvg()
    }
}

pub fn svgCircle() -> String {
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
