mod filesystem;
#[cfg(target_arch = "wasm32")]
mod indexdb;
use crate::error::GenericResult;
#[cfg(target_arch = "wasm32")]
use crate::error::TrackError;

/*
    $XDG_CONFIG_HOME
        Where user-specific configurations should be written (analogous to /etc).
        Should default to $HOME/.config.
    $XDG_CACHE_HOME
        Where user-specific non-essential (cached) data should be written (analogous to /var/cache).
        Should default to $HOME/.cache.
    $XDG_DATA_HOME
        Where user-specific data files should be written (analogous to /usr/share).
        Should default to $HOME/.local/share.
    $XDG_STATE_HOME
        Where user-specific state files should be written (analogous to /var/lib).
        Should default to $HOME/.local/state.
*/

pub enum Location {
    UserData,
    OsmCache,
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn write(b: &Location, filename: &str, data: String) -> GenericResult<()> {
    filesystem::write(&filesystem::Directory::from_location(b), filename, data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn read(b: &Location, filename: &str) -> GenericResult<String> {
    filesystem::read(&filesystem::Directory::from_location(b), filename)
}

#[cfg(target_arch = "wasm32")]
pub async fn write(b: &Location, filename: &str, data: String) -> GenericResult<()> {
    match indexdb::write(&indexdb::Database::from_location(b), filename, data).await {
        Ok(()) => Ok(()),
        Err(e) => {
            log::error!("error: {:?}", e);
            return Err(TrackError::IOError.into());
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn read(b: &Location, filename: &str) -> GenericResult<String> {
    match indexdb::read(&indexdb::Database::from_location(b), filename).await {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            log::error!("error: {:?}", e);
            Err(TrackError::IOError.into())
        }
    }
}
