use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TrackError {
    #[error("GPX file not found")]
    GPXNotFound,

    #[error("GPX file is invalid or malformed")]
    GPXInvalid,

    #[error("GPX file contains no segments")]
    GPXHasNoSegment,

    #[error("Missing elevation data at index {index}")]
    MissingElevation { index: usize },

    #[error("OSM download failed")]
    OSMDownloadFailed,

    #[error("OSM download timed out")]
    OSMDownloadTimeout,

    #[error("OSM download cancelled")]
    OSMDownloadCancelled,

    #[error("OSM download running")]
    OSMDownloadAlreadyRunning,

    #[error("input/output error")]
    IOError,

    #[error("An unknown error occurred")]
    Unknown,
}

#[derive(Error, Debug, Clone)]
pub enum RenderError {
    #[error("An unknown error occurred")]
    Unknown,
}

pub type GenericResult<T> = anyhow::Result<T>;

impl TrackError {
    pub fn from(e: anyhow::Error) -> TrackError {
        if let Some(e) = e.downcast_ref::<TrackError>() {
            return e.clone();
        }

        if let Some(_) = e.downcast_ref::<reqwest::Error>() {
            return TrackError::OSMDownloadFailed;
        }

        if let Some(_) = e.downcast_ref::<std::io::Error>() {
            return TrackError::GPXNotFound;
        }

        // 3. Fallback for everything else
        return TrackError::Unknown;
    }
}
