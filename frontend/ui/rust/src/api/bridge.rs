#![allow(non_snake_case)]

use flutter_rust_bridge::frb;

// must be exported for mirroring Segment.
pub use std::ops::Range;
pub use tracks::backend::SegmentStatistics;
pub use tracks::error::Error;
pub use tracks::parameters::Parameters;
pub use tracks::render_device::RenderDevice;
pub use tracks::utm::UTMPoint;
pub use tracks::waypoint::Waypoint;
pub use tracks::waypoint::WaypointInfo;
pub use tracks::waypoint::WaypointOrigin;

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
    pub fn shows_waypoint(&self, wp: &Waypoint) -> bool {
        self._impl.shows_waypoint(wp)
    }
}

#[frb(mirror(UTMPoint))]
pub struct _UTMPoint(pub f64, pub f64);

#[frb(mirror(WaypointOrigin))]
pub enum _WaypointOrigin {
    GPX,
    DouglasPeucker,
    MaxStepSize,
    OpenStreetMap,
}

#[frb(mirror(Parameters))]
pub struct _Parameters {
    pub debug: bool,
    pub epsilon: f64,
    pub max_step_size: f64,
    pub start_time: String,
    pub speed: f64,
    pub segment_length: f64,
    pub smooth_width: f64,
}

#[frb(mirror(WaypointInfo))]
pub struct _WaypointInfo {
    pub wgs84: (f64, f64, f64),
    pub utm: UTMPoint,
    pub origin: WaypointOrigin,
    pub distance: f64,
    pub elevation: f64,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub description: String,
    pub time: String,
    pub track_index: usize,
    pub value: Option<usize>,
}

#[frb(mirror(Waypoint))]
pub struct _Waypoint {
    pub wgs84: (f64, f64, f64),
    pub utm: UTMPoint,
    pub track_index: Option<usize>,
    pub origin: WaypointOrigin,
    pub name: Option<String>,
    pub description: Option<String>,
    pub info: Option<WaypointInfo>,
}

#[frb(mirror(SegmentStatistics))]
pub struct _SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

#[frb(mirror(Error))]
pub enum _Error {
    GPXNotFound,
    GPXInvalid,
    GPXHasNoSegment,
    MissingElevation { index: usize },
}

impl Bridge {
    pub async fn create(filename: &str) -> Result<Bridge, Error> {
        match tracks::backend::Backend::from_filename(filename).await {
            Ok(b) => Ok(Bridge { backend: b }),
            Err(e) => Err(e),
        }
    }
    pub async fn fromContent(content: &Vec<u8>) -> Result<Bridge, Error> {
        match tracks::backend::Backend::from_content(content).await {
            Ok(b) => Ok(Bridge { backend: b }),
            Err(e) => Err(e),
        }
    }
    pub async fn initDemo() -> Result<Bridge, Error> {
        match tracks::backend::Backend::demo().await {
            Ok(b) => Ok(Bridge { backend: b }),
            Err(e) => Err(e),
        }
    }
    pub async fn generatePdf(&mut self) -> Vec<u8> {
        self.backend.generatePdf()
    }
    pub async fn generateGpx(&mut self) -> Vec<u8> {
        self.backend.generateGpx()
    }
    #[frb(sync)] //TODO: add segment parameter
    pub fn getWaypoints(&mut self) -> Vec<Waypoint> {
        self.backend.waypoints.clone()
    }
    #[frb(sync)]
    pub fn elevation_gain(&mut self, from: usize, to: usize) -> f64 {
        self.backend.elevation_gain(from, to)
    }
    #[frb(sync)]
    pub fn get_parameters(&mut self) -> Parameters {
        self.backend.get_parameters()
    }
    #[frb(sync)]
    pub fn set_parameters(&mut self, parameters: &Parameters) {
        self.backend.set_parameters(parameters);
    }
    #[frb(sync)]
    pub fn waypoints_table(&self, segment: &Segment) -> Vec<Waypoint> {
        self.backend.get_waypoint_table(&segment._impl)
    }
    pub async fn renderSegmentWhat(
        &mut self,
        segment: &Segment,
        what: String,
        W: i32,
        H: i32,
    ) -> String {
        //let delay = std::time::Duration::from_millis(50);
        //std::thread::sleep(delay);
        println!("{}x{}", W, H);
        self.backend
            .render_segment_what(&segment._impl, what, (W, H), RenderDevice::Native)
    }
    #[frb(sync)]
    pub fn renderSegmentWhatSync(
        &mut self,
        segment: &Segment,
        what: String,
        W: i32,
        H: i32,
    ) -> String {
        //let delay = std::time::Duration::from_millis(50);
        //std::thread::sleep(delay);
        println!("{}x{}", W, H);
        self.backend
            .render_segment_what(&segment._impl, what, (W, H), RenderDevice::Native)
    }
    #[frb(sync)]
    pub fn statistics(&self) -> SegmentStatistics {
        self.backend.statistics()
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

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
