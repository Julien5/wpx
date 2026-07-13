use serde::{Deserialize, Serialize};

use crate::{
    backend::SegmentStatistics,
    cache,
    error::{GenericResult, TrackError},
    gpsdata::GpxData,
    parameters::{current_time_as_string, Parameters},
    trackfile::{
        controldataset::ControlDataset, jsonparameters::JsonParameters, trackdataset::TrackDataset,
    },
};

pub mod controldataset;
pub mod jsonparameters;
pub mod trackdataset;
pub mod v0;
pub(crate) mod v1;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TrackFile {
    pub number: usize,
    pub name: String,
    pub last_modified: String,
    pub start_time: String,
    pub length: f64,
    pub elevation_gain: f64,
}

static TRACKFILE_FILENAME: &'static str = "trackfile.json";
impl TrackFile {
    fn from_string(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    fn as_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    pub async fn read_all(&self) -> GenericResult<(JsonParameters, ControlDataset, GpxData)> {
        let params = JsonParameters::read(self).await.inspect_err(|e| {
            log::error!("{:?}", e);
        })?;
        let controls = ControlDataset::read(self).await.inspect_err(|e| {
            log::error!("{:?}", e);
        })?;
        let gpxdata = TrackDataset::read(self).await.inspect_err(|e| {
            log::error!("{:?}", e);
        })?;
        Ok((params, controls, gpxdata))
    }

    pub async fn list() -> GenericResult<Vec<TrackFile>> {
        let mut entries = cache::list(&cache::Location::UserData).await?;
        entries.retain(|e| e.ends_with(&TRACKFILE_FILENAME));
        let mut ret = Vec::new();
        for meta in &entries {
            match cache::read(&cache::Location::UserData, meta).await {
                Ok(content) => match TrackFile::from_string(&content) {
                    Ok(item) => ret.push(item),
                    Err(e) => {
                        log::error!("failed to read file content for file:{}", meta);
                        log::error!("error:{:?}", e);
                    }
                },
                Err(e) => {
                    log::error!("failed to read file:{}", meta);
                    log::error!("error:{:?}", e);
                }
            };
        }
        Ok(ret)
    }

    async fn write(&self) -> GenericResult<()> {
        cache::write(
            &cache::Location::UserData,
            &basename(self.number, TRACKFILE_FILENAME),
            self.as_string().unwrap(),
        )
        .await
    }

    pub async fn write_quick(
        &self,
        params: &JsonParameters,
        controls: &ControlDataset,
    ) -> GenericResult<()> {
        self.write().await?;
        params.write(self).await?;
        controls.write(self).await?;
        Ok(())
    }

    pub async fn write_all(
        &self,
        params: &JsonParameters,
        controls: &ControlDataset,
        gpxdata: &GpxData,
    ) -> GenericResult<()> {
        self.write_quick(params, controls).await?;
        let data = TrackDataset::from_gpxdata(&gpxdata);
        match data.write(&self).await {
            Ok(()) => Ok(()),
            Err(e) => {
                log::error!("write user data failed: {:?}", e);
                Err(TrackError::IOError.into())
            }
        }
    }

    pub async fn create(
        name: &String,
        stats: &SegmentStatistics,
        parameters: &Parameters,
    ) -> GenericResult<TrackFile> {
        let mut entries = cache::list(&cache::Location::UserData).await?;
        entries.retain(|e| e.ends_with(&TRACKFILE_FILENAME));
        entries.sort();

        let mut candidate = 0usize;
        while entries.contains(&basename(candidate, &TRACKFILE_FILENAME)) {
            candidate = candidate + 1;
            if candidate > 1_000_000 {
                return Err(TrackError::IOError.into());
            }
        }
        let trackfile = TrackFile {
            number: candidate,
            name: name.clone(),
            last_modified: current_time_as_string(),
            start_time: parameters.start_time.clone(),
            length: stats.length,
            elevation_gain: stats.elevation_gain,
        };
        trackfile.write().await?;
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
}

pub fn basename(number: usize, suffix: &str) -> String {
    format!("{:08}.{}", number, suffix)
}
