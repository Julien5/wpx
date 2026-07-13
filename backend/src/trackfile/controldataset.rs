use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    cache,
    error::{GenericResult, TrackError},
    inputpoint::{ControlData, InputPoint, InputPointData},
    mercator::WebMercatorProjection,
    trackfile::TrackFile,
    wgs84point::WGS84Point,
};

static CONTROLDATA_FILENAME: &'static str = "controls.gpx";

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ControlCmtMeta {
    waypoint_name: String,
    segment_name: String,
    nearest_waypoint_id: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ControlDataset {
    gpx: String,
}

fn as_gpx_waypoint(c: &InputPoint) -> Option<gpx::Waypoint> {
    if let InputPointData::Control(cd) = &c.data {
        let mut wp = gpx::Waypoint::new(geo::Point::new(c.wgs84.longitude(), c.wgs84.latitude()));
        wp.elevation = Some(c.wgs84.z());
        wp.name = Some(cd.name.clone());
        wp.description = Some(cd.waypoint_description.clone());

        if let Some(t) = &cd.cutoff_time {
            let odt = time::OffsetDateTime::from_unix_timestamp(t.timestamp())
                .unwrap()
                .replace_nanosecond(t.timestamp_subsec_nanos())
                .unwrap();
            wp.time = Some(gpx::Time::from(odt));
        }

        let meta = ControlCmtMeta {
            waypoint_name: cd.waypoint_name.clone(),
            segment_name: cd.segment_name.clone(),
            nearest_waypoint_id: cd.nearest_waypoint_id,
        };
        wp.comment = Some(serde_json::to_string(&meta).unwrap());

        Some(wp)
    } else {
        None
    }
}

fn as_control(wp: &gpx::Waypoint, projection: &WebMercatorProjection) -> Option<InputPoint> {
    let (lon, lat) = wp.point().x_y();
    let ele = wp.elevation.unwrap_or(0.0);
    let wgs84 = WGS84Point::new(&lon, &lat, &ele);
    let euclidean = projection.project(&wgs84);

    let cutoff_time = wp.time.map(|t| {
        let odt: time::OffsetDateTime = t.into();
        chrono::DateTime::from_timestamp(odt.unix_timestamp(), odt.nanosecond())
            .unwrap()
            .with_timezone(&chrono::Local)
    });

    let comment_obj = wp.comment.as_deref().and_then(|cmt| {
        serde_json::from_str::<ControlCmtMeta>(cmt)
            .ok()
            .map(|meta| {
                (
                    meta.waypoint_name,
                    meta.segment_name,
                    meta.nearest_waypoint_id,
                )
            })
    });

    if comment_obj.is_none() {
        log::error!("failed to read control comment object");
        return None;
    }

    let (waypoint_name, segment_name, nearest_waypoint_id) = comment_obj.unwrap();

    let cd = ControlData {
        name: wp.name.clone().unwrap_or_default(),
        waypoint_name,
        waypoint_description: wp.description.clone().unwrap_or_default(),
        segment_name,
        nearest_waypoint_id,
        cutoff_time,
    };
    Some(InputPoint {
        wgs84,
        euclidean,
        data: InputPointData::Control(cd),
        track_projections: Default::default(),
        index: None,
    })
}

impl ControlDataset {
    pub fn empty() -> Self {
        Self { gpx: String::new() }
    }

    pub fn from_controls(controls: &[InputPoint]) -> Self {
        let mut gpx_obj = gpx::Gpx::default();
        gpx_obj.version = gpx::GpxVersion::Gpx11;
        gpx_obj.waypoints = controls.iter().filter_map(|c| as_gpx_waypoint(c)).collect();
        let mut data = Vec::new();
        gpx::write(&gpx_obj, &mut data).unwrap();
        Self {
            gpx: String::from_utf8(data).unwrap(),
        }
    }

    pub fn to_controls(&self) -> Result<Vec<InputPoint>, TrackError> {
        let bytes = self.gpx.as_bytes().to_vec();
        let reader = std::io::Cursor::new(bytes);
        let gpx_obj = gpx::read(reader).map_err(|_| TrackError::IOError)?;

        let projection = WebMercatorProjection::make();
        let mut controls = Vec::new();

        for wp in &gpx_obj.waypoints {
            if let Some(control) = as_control(wp, &projection) {
                controls.push(control);
            } else {
                return Err(TrackError::IOError.into());
            }
        }

        Ok(controls)
    }

    pub fn from_string(data: &str) -> Result<Self, TrackError> {
        match String::from_str(&data) {
            Ok(string) => Ok(Self { gpx: string }),
            Err(_) => Err(TrackError::IOError.into()),
        }
    }

    pub fn as_string(&self) -> Result<String, serde_json::Error> {
        Ok(self.gpx.clone())
    }

    pub async fn write(&self, trackfile: &TrackFile) -> GenericResult<()> {
        cache::write(
            &cache::Location::UserData,
            &super::basename(trackfile.number, CONTROLDATA_FILENAME),
            self.as_string().unwrap(),
        )
        .await
    }

    pub async fn read(trackfile: &TrackFile) -> GenericResult<Self> {
        let bytes = cache::read(
            &cache::Location::UserData,
            &super::basename(trackfile.number, CONTROLDATA_FILENAME),
        )
        .await?;
        Ok(Self::from_string(&bytes)?)
    }
}
