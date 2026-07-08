use std::str::FromStr;

use crate::{
    backend::SegmentStatistics,
    cache,
    error::{GenericResult, TrackError},
    gpsdata::GpxData,
    inputpoint::InputPoint,
    parameters::{current_time_as_string, Parameters},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TrackFile {
    pub number: usize,
    pub name: String,
    pub last_modified: String,
    pub start_time: String,
    pub length: f64,
    pub elevation_gain: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JsonParameters {
    pub parameters: Parameters,
    pub controls: Vec<InputPoint>,
    pub trackfile: TrackFile,
}

fn basename(number: usize, suffix: &str) -> String {
    format!("{:08}.{}", number, suffix)
}

static JSONPARAMETERS_FILENAME: &'static str = "parameters.json";
impl JsonParameters {
    fn from_string(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    fn as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    pub async fn list() -> GenericResult<Vec<TrackFile>> {
        let mut entries = cache::list(&cache::Location::UserData).await?;
        entries.retain(|e| e.ends_with(&JSONPARAMETERS_FILENAME));
        let mut ret = Vec::new();
        for meta in &entries {
            let content = match cache::read(&cache::Location::UserData, meta).await {
                Ok(content) => content,
                Err(e) => {
                    log::error!("failed to read file:{}", meta);
                    return Err(e.into());
                }
            };
            match Self::from_string(&content) {
                Ok(item) => ret.push(item.trackfile),
                Err(e) => {
                    log::error!("failed to read file content for file:{}", meta);
                    return Err(e.into());
                }
            }
        }
        Ok(ret)
    }

    pub async fn create(name: &String, stats: &SegmentStatistics) -> GenericResult<TrackFile> {
        let mut entries = cache::list(&cache::Location::UserData).await?;
        entries.retain(|e| e.ends_with(&JSONPARAMETERS_FILENAME));
        entries.sort();

        let mut candidate = 0usize;
        while entries.contains(&basename(candidate, &JSONPARAMETERS_FILENAME)) {
            candidate = candidate + 1;
            if candidate > 1_000_000 {
                return Err(TrackError::IOError.into());
            }
        }

        let parameters = Parameters::default();

        let trackfile = TrackFile {
            number: entries.len(),
            name: name.clone(),
            last_modified: current_time_as_string(),
            start_time: parameters.start_time.clone(),
            length: stats.length,
            elevation_gain: stats.elevation_gain,
        };

        let parameters = JsonParameters {
            parameters: parameters.clone(),
            controls: Vec::new(),
            trackfile: trackfile.clone(),
        };

        let _ = parameters.write().await?;
        Ok(trackfile)
    }

    pub async fn remove(trackfile: &TrackFile) -> GenericResult<()> {
        let mut entries = cache::list(&cache::Location::UserData).await?;
        let empty = "";
        entries.retain(|e| e.starts_with(&basename(trackfile.number, &empty)));
        let mut all_good = true;
        for e in entries {
            match cache::remove(&cache::Location::UserData, &e).await {
                Ok(()) => {}
                Err(_) => {
                    all_good = false;
                }
            }
        }
        match all_good {
            true => Ok(()),
            false => Err(TrackError::IOError.into()),
        }
    }

    pub async fn write(&self) -> GenericResult<()> {
        cache::write(
            &cache::Location::UserData,
            &basename(self.trackfile.number, JSONPARAMETERS_FILENAME),
            self.as_string().unwrap(),
        )
        .await
    }

    pub async fn read(trackfile: &TrackFile) -> Option<Self> {
        match cache::read(
            &cache::Location::UserData,
            &basename(trackfile.number, JSONPARAMETERS_FILENAME),
        )
        .await
        {
            Ok(bytes) => match JsonParameters::from_string(&bytes) {
                Ok(d) => {
                    debug_assert!(d.trackfile.number == trackfile.number);
                    Some(d)
                }
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

    pub fn from_string(data: &str) -> Result<Self, TrackError> {
        match String::from_str(&data) {
            Ok(string) => Ok(Self { gpx: string }),
            Err(_) => Err(TrackError::IOError.into()),
        }
    }

    pub fn as_string(&self) -> Result<String, serde_json::Error> {
        Ok(self.gpx.clone())
    }
}

static TRACKDATA_FILENAME: &'static str = "track.gpx";

pub async fn write_trackdata(trackfile: &TrackFile, data: &TrackDataset) -> GenericResult<()> {
    cache::write(
        &cache::Location::UserData,
        &basename(trackfile.number, TRACKDATA_FILENAME),
        data.as_string().unwrap(),
    )
    .await
    // TODO: update last modified in meta.
}

pub async fn read_trackdata(trackfile: &TrackFile) -> Option<GpxData> {
    match cache::read(
        &cache::Location::UserData,
        &basename(trackfile.number, TRACKDATA_FILENAME),
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
