use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    cache::Location,
    error::{GenericResult, TrackError},
};

pub enum Directory {
    UserData,
    OsmCache,
}

impl Directory {
    pub fn from_location(b: &Location) -> Self {
        match b {
            Location::UserData => Self::UserData,
            Location::OsmCache => Self::OsmCache,
        }
    }

    fn name(&self) -> String {
        let version = match self {
            Directory::UserData => 1,
            Directory::OsmCache => 1,
        };
        let name = match self {
            Directory::UserData => {
                let dir = env::var("DATA_DIR").unwrap_or_else(|_| {
                    dirs::data_local_dir()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string()
                });
                format!("{}/{}/{}", dir, "WPX", version)
            }
            Directory::OsmCache => {
                let dir = env::var("CACHE_DIR").unwrap_or_else(|_| {
                    if cfg!(test) {
                        // env!("CARGO_MANIFEST_DIR") resolves at compile time to the directory
                        // containing Cargo.toml (i.e. backend/
                        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                        p.push("data/ref/cache");
                        p.to_string_lossy().into_owned()
                    } else {
                        dirs::cache_dir().unwrap().to_str().unwrap().to_string()
                    }
                });
                format!("{}/{}/{}", dir, "WPX/osm", version)
            }
        };
        name
    }
}

pub fn read(directory: &Directory, filename: &str) -> GenericResult<String> {
    let abspath = format!("{}/{}", directory.name(), filename);
    let path = Path::new(&abspath);
    log::trace!("read: [{}]", abspath);
    match std::fs::read_to_string(path) {
        Ok(data) => Ok(data),
        Err(e) => Err(e.into()),
    }
}

#[allow(dead_code)]
pub fn hit_cache(path: &str) -> bool {
    let path = Path::new(path);
    if !path.exists() {
        return false;
    }
    true
}

pub fn write(directory: &Directory, filename: &str, data: String) -> GenericResult<()> {
    let abspath = format!("{}/{}", directory.name(), filename);
    let pathbuf = PathBuf::from(&abspath);
    let dirname = pathbuf.parent().unwrap().to_str().unwrap();
    match std::fs::create_dir_all(dirname) {
        Ok(()) => {}
        Err(e) => {
            log::error!("error mkdir in {}: {:?}", directory.name(), e);
            return Err(TrackError::IOError.into());
        }
    }
    match std::fs::write(abspath, data) {
        Ok(()) => Ok(()),
        Err(e) => {
            log::error!("error writing in {}: {:?}", directory.name(), e);
            Err(TrackError::IOError.into())
        }
    }
}
