mod filesystem;
#[cfg(target_arch = "wasm32")]
mod indexdb;
use crate::error::GenericResult;

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

// read

#[cfg(not(target_arch = "wasm32"))]
pub async fn read(b: &Location, filename: &str) -> GenericResult<String> {
    match filesystem::read(&filesystem::Directory::from_location(b), filename) {
        Ok(data) => Ok(data),
        Err(e) => {
            log::error!("could not read {filename} because of {:?}", e);
            Err(e)
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn read(b: &Location, filename: &str) -> GenericResult<String> {
    match indexdb::read(&indexdb::IndexdbLocation::from_location(b), filename).await {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            log::error!("error: {:?}", e);
            Err(crate::error::TrackError::IOError.into())
        }
    }
}

// write

#[cfg(not(target_arch = "wasm32"))]
pub async fn write(b: &Location, filename: &str, data: String) -> GenericResult<()> {
    filesystem::write(&filesystem::Directory::from_location(b), filename, data)
}

#[cfg(target_arch = "wasm32")]
pub async fn write(b: &Location, filename: &str, data: String) -> GenericResult<()> {
    match indexdb::write(&indexdb::IndexdbLocation::from_location(b), filename, data).await {
        Ok(()) => Ok(()),
        Err(e) => {
            log::error!("error: {:?}", e);
            return Err(crate::error::TrackError::IOError.into());
        }
    }
}

// list

#[cfg(not(target_arch = "wasm32"))]
pub async fn list(b: &Location) -> GenericResult<Vec<String>> {
    filesystem::allfiles(&filesystem::Directory::from_location(b))
}

#[cfg(target_arch = "wasm32")]
pub async fn list(b: &Location) -> GenericResult<Vec<String>> {
    match indexdb::allfiles(&indexdb::IndexdbLocation::from_location(b)).await {
        Ok(keys) => Ok(keys),
        Err(e) => {
            log::error!("error: {:?}", e);
            Err(crate::error::TrackError::IOError.into())
        }
    }
}

// remove

#[cfg(not(target_arch = "wasm32"))]
pub async fn remove(b: &Location, filename: &str) -> GenericResult<()> {
    filesystem::remove(&filesystem::Directory::from_location(b), filename)
}

#[cfg(target_arch = "wasm32")]
pub async fn remove(b: &Location, filename: &str) -> GenericResult<()> {
    match indexdb::remove(&indexdb::IndexdbLocation::from_location(b), filename).await {
        Ok(()) => Ok(()),
        Err(e) => {
            log::error!("error: {:?}", e);
            Err(crate::error::TrackError::IOError.into())
        }
    }
}
