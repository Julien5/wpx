#![allow(non_snake_case)]

use flutter_rust_bridge::frb;

use std::collections::HashSet;
// must be exported for mirroring Segment.
pub use std::ops::Range;
pub use tracks::backend::Segment as SegmentImplementation;
pub use tracks::backend::SegmentStatistics;
pub use tracks::error::Error;
pub use tracks::inputpoint::InputType;
pub use tracks::mercator::MercatorPoint;
pub use tracks::parameters::MapOptions;
pub use tracks::parameters::Parameters;
pub use tracks::parameters::ProfileIndication;
pub use tracks::parameters::ProfileOptions;
pub use tracks::parameters::UserStepsOptions;
pub use tracks::waypoint::Waypoint;
pub use tracks::waypoint::WaypointInfo;
pub use tracks::wgs84point::WGS84Point;

use tracks::math::IntegerSize2D;

#[frb(opaque)]
pub struct Bridge {
    backend: tracks::backend::Backend,
}
use crate::frb_generated::StreamSink;

#[frb(opaque)]
#[derive(Clone)]
pub struct EventSender {
    sink: StreamSink<String>,
}

use tracks::backend::Sender;

impl Sender for EventSender {
    fn send(&mut self, data: &String) {
        let _ = self.sink.add(data.clone());
    }
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
    pub fn id(&self) -> i32 {
        self._impl.id
    }

    #[frb(sync)]
    pub fn set_profile_indication(&mut self, p: &ProfileIndication) {
        self._impl.set_profile_indication(p);
    }
}

#[frb(mirror(WaypointOrigin))]
pub enum _WaypointOrigin {
    GPX,
    DouglasPeucker,
    OpenStreetMap,
}

#[frb(mirror(InputType))]
pub enum _InputType {
    GPX,
    OSM,
    UserStep,
    Control,
}

#[frb(sync)]
pub fn allkinds() -> HashSet<InputType> {
    tracks::inputpoint::allkinds()
}

#[frb(mirror(ProfileIndication))]
pub enum _ProfileIndication {
    None,
    GainTicks,
    NumericSlope,
}

#[frb(mirror(UserStepsOptions))]
pub struct _UserStepsOptions {
    pub step_distance: Option<f64>,
    pub step_elevation_gain: Option<f64>,
}

#[frb(mirror(ProfileOptions))]
pub struct _ProfileOptions {
    pub elevation_indicators: std::collections::HashSet<ProfileIndication>,
    pub max_area_ratio: f64,
    pub size: (i32, i32),
}

#[frb(mirror(MapOptions))]
pub struct _MapOptions {
    pub max_area_ratio: f64,
    pub size: (i32, i32),
}

#[frb(mirror(Parameters))]
pub struct _Parameters {
    pub debug: bool,
    pub map_options: MapOptions,
    pub profile_options: ProfileOptions,
    pub segment_length: f64,
    pub segment_overlap: f64,
    pub smooth_width: f64,
    pub speed: f64,
    pub start_time: String,
    pub user_steps_options: UserStepsOptions,
    pub waypoint_gpx_name_format: String,
}

#[frb(mirror(WaypointInfo))]
pub struct _WaypointInfo {
    pub distance: f64,
    pub elevation: f64,
    pub gpx_name: String,
    pub inter_distance: f64,
    pub inter_elevation_gain: f64,
    pub inter_slope: f64,
    pub name: String,
    pub origin: InputType,
    pub time: String,
    pub track_index: Option<usize>,
    pub description: String,
}

#[frb(mirror(Waypoint))]
pub struct _Waypoint {
    pub wgs84: WGS84Point,
    pub euclidean: MercatorPoint,
    pub track_index: Option<usize>,
    pub origin: InputType,
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

use tracks::backend;
impl Bridge {
    #[frb(sync)]
    pub fn make() -> Bridge {
        Bridge {
            backend: backend::Backend::make(),
        }
    }
    #[frb(sync)]
    pub fn set_sink(&mut self, sink: StreamSink<String>) -> anyhow::Result<()> {
        let cell = Box::new(EventSender { sink });
        self.backend.set_sink(cell);
        Ok(())
    }
    pub async fn load_filename(&mut self, filename: &str) -> Result<(), Error> {
        self.backend.load_filename(filename).await
    }
    pub async fn load_content(&mut self, content: &Vec<u8>) -> Result<(), Error> {
        self.backend.load_content(content).await
    }
    pub async fn load_demo(&mut self) -> Result<(), Error> {
        self.backend.load_demo().await
    }
    pub async fn generatePdf(&mut self) -> Vec<u8> {
        self.backend.generatePdf().await
    }
    pub async fn generateGpx(&mut self) -> Vec<u8> {
        self.backend.generateGpx()
    }
    #[frb(sync)]
    pub fn get_waypoints(&mut self, segment: &Segment, kinds: HashSet<InputType>) -> Vec<Waypoint> {
        self.backend.get_waypoints(&segment._impl, kinds)
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
    pub fn set_user_step_options(
        &mut self,
        segment: &mut Segment,
        user_steps_options: &UserStepsOptions,
    ) {
        segment._impl.set_user_step_options(user_steps_options);
    }

    #[frb(sync)]
    pub fn set_waypoint_gpx_name_format(&mut self, format: &String) {
        self.backend.set_waypoint_gpx_name_format(format);
    }

    #[frb(sync)]
    pub fn get_user_step_options(&mut self, segment: &mut Segment) -> UserStepsOptions {
        segment._impl.get_user_step_options()
    }

    #[frb(sync)]
    pub fn is_loaded(&self) -> bool {
        self.backend.loaded()
    }

    pub async fn renderSegmentWhat(
        &mut self,
        segment: &Segment,
        what: &String,
        size: &(i32, i32),
        kinds: HashSet<InputType>,
    ) -> String {
        assert!(self.backend.loaded());
        self.backend.render_segment_what(
            &segment._impl,
            what,
            &IntegerSize2D::new(size.0, size.1),
            kinds,
        )
    }

    #[frb(sync)]
    pub fn renderSegmentWhatSync(
        &mut self,
        segment: &Segment,
        what: &String,
        size: &(i32, i32),
        kinds: HashSet<InputType>,
    ) -> String {
        self.backend.render_segment_what(
            &segment._impl,
            what,
            &IntegerSize2D::new(size.0, size.1),
            kinds,
        )
    }

    #[frb(sync)]
    pub fn statistics(&self) -> SegmentStatistics {
        self.backend.statistics()
    }

    #[frb(sync)]
    pub fn segment_statistics(&self, segment: &Segment) -> SegmentStatistics {
        self.backend.segment_statistics(&segment._impl)
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

    #[frb(sync)]
    pub fn trackSegment(&self) -> Segment {
        let backend_segment = self.backend.trackSegment();
        Segment::create(backend_segment)
    }
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
    crate::setup::setup();
}
