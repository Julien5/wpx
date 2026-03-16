#![allow(non_snake_case)]

use flutter_rust_bridge::frb;

use std::collections::HashSet;
use tracks::parameters;
use tracks::waypoint;

// must be exported for mirroring Segment.
pub use std::ops::Range;
pub use tracks::backend::Segment as SegmentImplementation;
pub use tracks::backend::SegmentStatistics;
pub use tracks::error::RenderError;
pub use tracks::error::TrackError;
pub use tracks::mercator::MercatorPoint;
pub use tracks::parameters::ControlSource;
pub use tracks::parameters::MapOptions;
pub use tracks::parameters::Parameters;
pub use tracks::parameters::ProfileIndication;
pub use tracks::parameters::ProfileOptions;
pub use tracks::parameters::RenderFunction;
pub use tracks::parameters::RenderInput;
pub use tracks::parameters::RenderOutput;
pub use tracks::parameters::TrackPart;
pub use tracks::parameters::UserStepsOptions;
pub use tracks::point_collection::Kind;
pub use tracks::waypoint::Waypoint;
pub use tracks::waypoint::WaypointInfo;
pub use tracks::wgs84point::WGS84Point;

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
    fn send(&mut self, data: &str) {
        match self.sink.add(data.to_string()) {
            Ok(()) => {
                log::trace!("sent [{}]", data);
            }
            Err(e) => {
                log::error!("failed to send [{}] because {:?}", data, e);
            }
        }
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
}

#[frb(mirror(Kind))]
pub enum _Kind {
    Cities,
    Controls,
    GPXWaypoints,
    Hamlets,
    Mountains,
    Villages,
    UserStep,
}

#[frb(sync)]
pub fn allkinds() -> HashSet<Kind> {
    tracks::point_collection::allkinds()
}

#[frb(mirror(RenderFunction))]
pub enum _RenderFunction {
    Map,
    Profile,
    Wheel,
    WheelPages,
}

#[frb(mirror(RenderInput))]
pub struct _RenderInput {
    pub kinds: HashSet<Kind>,
    pub function: RenderFunction,
    pub size: (i32, i32),
}

#[frb(mirror(RenderError), unignore)]
pub enum _RenderError {
    Unknown,
}

#[frb(mirror(RenderOutput))]
pub struct _RenderOutput {
    pub svg: String,
    pub render_input: RenderInput,
    pub error: Option<RenderError>,
    pub waypoints: Vec<Waypoint>,
}

#[frb(mirror(TrackPart))]
pub struct _TrackPart {
    pub name: String,
    pub part_index: usize,
    pub length: usize,
}

#[frb(sync)]
pub fn karl_order(parts: &Vec<TrackPart>) -> Vec<TrackPart> {
    parameters::karl_order(parts)
}

#[frb(sync)]
pub fn decimate(segment: &Segment, waypoints: &Vec<Waypoint>, n: usize) -> Vec<Waypoint> {
    waypoint::decimate(&segment._impl, waypoints, n)
}

#[frb(mirror(ProfileIndication))]
pub enum _ProfileIndication {
    None,
    NumericSlope,
}

#[frb(mirror(ControlSource))]
pub enum _ControlSource {
    Segments,
    Waypoints,
    OSM,
}

#[frb(mirror(UserStepsOptions))]
pub struct _UserStepsOptions {
    pub step_distance: Option<f64>,
    pub step_elevation_gain: Option<f64>,
    pub gpx_name_format: String,
}

#[frb(mirror(ProfileOptions))]
pub struct _ProfileOptions {
    pub elevation_indicators: Vec<ProfileIndication>,
    pub max_area_ratio: f64,
}

#[frb(mirror(MapOptions))]
pub struct _MapOptions {
    pub max_area_ratio: f64,
}

#[frb(mirror(Parameters))]
pub struct _Parameters {
    pub control_gpx_name_format: String,
    pub debug: bool,
    pub map_options: MapOptions,
    pub profile_options: ProfileOptions,
    pub segment_length: f64,
    pub segment_overlap: f64,
    pub smooth_width: f64,
    pub speed: f64,
    pub start_time: String,
    pub user_steps_options: UserStepsOptions,
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
    pub origin: Kind,
    pub time: String,
    pub track_index: Option<usize>,
    pub description: String,
}

#[frb(mirror(Waypoint))]
pub struct _Waypoint {
    pub wgs84: WGS84Point,
    pub euclidean: MercatorPoint,
    pub track_index: Option<usize>,
    pub origin: Kind,
    pub name: String,
    pub description: String,
    pub info: Option<WaypointInfo>,
}

#[frb(mirror(SegmentStatistics))]
pub struct _SegmentStatistics {
    pub length: f64,
    pub elevation_gain: f64,
    pub distance_start: f64,
    pub distance_end: f64,
}

#[frb(mirror(TrackError), unignore)]
pub enum _TrackError {
    GPXNotFound,
    GPXInvalid,
    GPXHasNoSegment,
    MissingElevation { index: usize },
    OSMDownloadFailed,
    OSMDownloadTimeout,
    Unknown,
}

#[frb(sync)]
pub fn demo_bytes() -> Vec<u8> {
    include_bytes!("../../../../../backend/data/ref/roland.gpx").to_vec()
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
    pub fn set_sink(&mut self, sink: StreamSink<String>) -> Result<(), TrackError> {
        let cell = Box::new(EventSender { sink });
        self.backend.set_sink(cell);
        Ok(())
    }

    pub async fn load_controls(&self, source: ControlSource) -> Result<usize, TrackError> {
        self.backend.load_controls(source).await
    }

    pub async fn load_osm(&self) -> Result<(), TrackError> {
        self.backend.load_osm().await
    }

    pub async fn load_contents(&mut self, contents: &Vec<Vec<u8>>) -> Result<(), TrackError> {
        self.backend.load_contents(contents).await
    }

    pub async fn load_track_parts(
        &self,
        contents: &Vec<Vec<u8>>,
    ) -> Result<Vec<TrackPart>, TrackError> {
        self.backend.load_track_parts(contents).await
    }

    pub async fn load_ordered(&mut self, parts: &Vec<TrackPart>) -> Result<(), TrackError> {
        self.backend.load_ordered(parts).await
    }

    pub async fn generatePdf(&mut self) -> Vec<u8> {
        self.backend.generatePdf().await
    }
    pub async fn generateGpx(&mut self) -> Vec<u8> {
        self.backend.generateGpx()
    }
    pub async fn generateZip(&mut self) -> Vec<u8> {
        self.backend.generateZip().await
    }
    #[frb(sync)]
    pub fn get_waypoints(&self, segment: &Segment, kinds: HashSet<Kind>) -> Vec<Waypoint> {
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
    pub fn set_user_step_options(&mut self, user_steps_options: &UserStepsOptions) {
        self.backend.set_user_step_options(user_steps_options);
    }

    #[frb(sync)]
    pub fn set_profile_indications(&mut self, indications: &Vec<ProfileIndication>) {
        self.backend.set_profile_indications(indications);
    }

    #[frb(sync)]
    pub fn set_userstep_gpx_name_format(&mut self, format: &String) {
        self.backend.set_userstep_gpx_name_format(format);
    }

    #[frb(sync)]
    pub fn set_control_gpx_name_format(&mut self, format: &String) {
        self.backend.set_control_gpx_name_format(format);
    }

    #[frb(sync)]
    pub fn is_loaded(&self) -> bool {
        self.backend.loaded()
    }

    pub async fn renderSegment(
        &mut self,
        segment: &Segment,
        inputs: &Vec<RenderInput>,
    ) -> Vec<RenderOutput> {
        assert!(self.backend.loaded());
        self.backend.render_segment(&segment._impl, inputs)
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
