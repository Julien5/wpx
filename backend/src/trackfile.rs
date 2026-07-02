use crate::{
    backend::SegmentStatistics,
    cache,
    error::{GenericResult, TrackError},
    gpsdata::GpxData,
    inputpoint::InputPoint,
    parameters::Parameters,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TrackFile {
    pub number: usize,
    pub name: String,
    pub last_modified: String,
    pub length: f64,
    pub elevation_gain: f64,
}

impl TrackFile {
    pub async fn read_all() -> GenericResult<Vec<TrackFile>> {
        let mut entries = cache::allfiles(&cache::Location::UserData).await?;
        entries.retain(|e| e.ends_with(&".meta"));
        let mut ret = Vec::new();
        for meta in &entries {
            let content = cache::read(&cache::Location::UserData, meta).await?;
            ret.push(Self::from_string(&content)?);
        }
        Ok(ret)
    }
    pub fn from_string(data: &str) -> GenericResult<Self> {
        serde_json::from_str(data).map_err(|_| TrackError::IOError.into())
    }

    pub fn as_string(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
    pub async fn write_meta(&self) -> GenericResult<()> {
        let filename = format!("{}.meta", self.number);
        let _ = cache::write(&cache::Location::UserData, &filename, self.as_string()).await;
        Ok(())
    }
    pub async fn create(name: &String, stats: &SegmentStatistics) -> GenericResult<Self> {
        let mut entries = cache::allfiles(&cache::Location::UserData).await?;
        entries.retain(|e| e.ends_with(&".meta"));
        let ret = TrackFile {
            number: entries.len(),
            name: name.clone(),
            last_modified: String::new(),
            length: stats.length,
            elevation_gain: stats.elevation_gain,
        };

        let _ = ret.write_meta().await?;
        Ok(ret)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SmallParameters {
    pub parameters: Parameters,
    pub controls: Vec<InputPoint>,
}

impl SmallParameters {
    pub fn from_string(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    pub fn as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }
}

static SMALLPARAMETERS_FILENAME: &'static str = "small-parameters";

pub async fn write_smallparameters(
    trackfile: &TrackFile,
    data: &SmallParameters,
) -> GenericResult<()> {
    cache::write(
        &cache::Location::UserData,
        &format!("{}.{}", trackfile.number, SMALLPARAMETERS_FILENAME),
        data.as_string().unwrap(),
    )
    .await
    // TODO: update last modified in meta.
}

pub async fn read_smallparameters(trackfile: &TrackFile) -> Option<SmallParameters> {
    match cache::read(
        &cache::Location::UserData,
        &format!("{}.{}", trackfile.number, SMALLPARAMETERS_FILENAME),
    )
    .await
    {
        Ok(bytes) => match SmallParameters::from_string(&bytes) {
            Ok(d) => Some(d),
            Err(e) => {
                log::error!("coud not read data {:?}", e);
                None
            }
        },
        Err(e) => {
            log::error!("coud not read data {:?}", e);
            None
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TrackDataset {
    gpx: String,
}

impl TrackDataset {
    pub fn from_track_and_waypoints(
        track: &crate::track::Track,
        waypoints: &Vec<InputPoint>,
    ) -> Self {
        // Convert each TrackPart into a separate GPX track using the point ranges.
        use crate::trackparts::parts_to_ranges;
        let ranges = parts_to_ranges(&track.parts);
        let mut tracks = Vec::new();
        for (part, range) in track.parts.iter().zip(ranges.iter()) {
            // Skip empty parts just in case.
            if range.start >= range.end {
                continue;
            }
            let mut t = gpx::Track::new();
            let mut segment = gpx::TrackSegment::new();
            for wgs in &track.wgs84[range.clone()] {
                let mut wp = gpx::Waypoint::new(geo::Point::new(wgs.longitude(), wgs.latitude()));
                wp.elevation = Some(wgs.z());
                segment.points.push(wp);
            }
            t.segments.push(segment);
            tracks.push((part.name.clone(), t));
        }
        let gpxdata = GpxData {
            waypoints: waypoints.clone(),
            tracks,
        };
        Self::from_gpxdata(&gpxdata)
    }

    pub fn from_gpxdata(gpxdata: &GpxData) -> Self {
        let mut gpx = gpx::Gpx::default();
        gpx.version = gpx::GpxVersion::Gpx11;

        let waypoints: Vec<gpx::Waypoint> = gpxdata
            .waypoints
            .iter()
            .map(|w| {
                let mut wp =
                    gpx::Waypoint::new(geo::Point::new(w.wgs84.longitude(), w.wgs84.latitude()));
                wp.elevation = Some(w.wgs84.z());
                wp.name = Some(w.name());
                wp.description = Some(w.description());
                wp
            })
            .collect();

        let tracks: Vec<gpx::Track> = gpxdata
            .tracks
            .iter()
            .map(|(name, track)| {
                let mut t = track.clone();
                t.name = Some(name.clone());
                t
            })
            .collect();

        gpx.waypoints = waypoints;
        gpx.tracks = tracks;

        let mut data = Vec::new();
        gpx::write(&gpx, &mut data).unwrap();
        Self {
            gpx: String::from_utf8(data).unwrap(),
        }
    }

    pub fn to_gpxdata(&self) -> Result<GpxData, TrackError> {
        let bytes = self.gpx.as_bytes().to_vec();
        GpxData::read_content(&bytes)
    }

    pub fn from_string(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    pub fn as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }
}

static TRACKDATA_FILENAME: &'static str = "track-data";

pub async fn write_trackdata(trackfile: &TrackFile, data: &TrackDataset) -> GenericResult<()> {
    cache::write(
        &cache::Location::UserData,
        &format!("{}.{}", trackfile.number, TRACKDATA_FILENAME),
        data.as_string().unwrap(),
    )
    .await
    // TODO: update last modified in meta.
}

pub async fn read_trackdata(trackfile: &TrackFile) -> Option<GpxData> {
    match cache::read(
        &cache::Location::UserData,
        &format!("{}.{}", trackfile.number, TRACKDATA_FILENAME),
    )
    .await
    {
        Ok(bytes) => match TrackDataset::from_string(&bytes) {
            Ok(data) => match data.to_gpxdata() {
                Ok(gpxdata) => Some(gpxdata),
                Err(e) => {
                    log::error!("could not read data {:?}", e);
                    None
                }
            },
            Err(e) => {
                log::error!("could not read data {:?}", e);
                None
            }
        },
        Err(e) => {
            log::error!("could not read data {:?}", e);
            None
        }
    }
}
