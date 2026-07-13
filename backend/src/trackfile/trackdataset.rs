use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    cache,
    error::{GenericResult, TrackError},
    gpsdata::GpxData,
    inputpoint::InputPoint,
    trackfile::TrackFile,
};

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

    pub async fn write(&self, trackfile: &TrackFile) -> GenericResult<()> {
        cache::write(
            &cache::Location::UserData,
            &super::basename(trackfile.number, TRACKDATA_FILENAME),
            self.as_string().unwrap(),
        )
        .await
    }

    pub async fn read(trackfile: &TrackFile) -> GenericResult<GpxData> {
        let bytes = cache::read(
            &cache::Location::UserData,
            &super::basename(trackfile.number, TRACKDATA_FILENAME),
        )
        .await?;
        let data = Self::from_string(&bytes)?;
        Ok(data.to_gpxdata()?)
    }
}

static TRACKDATA_FILENAME: &'static str = "track.gpx";
