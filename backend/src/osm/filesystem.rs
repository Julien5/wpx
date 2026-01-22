use std::path::{Path, PathBuf};

use crate::error::{GenericError, GenericResult};

pub fn read(path: &String) -> GenericResult<String> {
    let path = Path::new(path.as_str());
    match std::fs::read_to_string(path) {
        Ok(data) => Ok(data),
        Err(e) => Err(GenericError::from(e)),
    }
}

#[allow(dead_code)]
pub fn hit_cache(path: &String) -> bool {
    let path = Path::new(path.as_str());
    if !path.exists() {
        return false;
    }
    true
}

pub fn write(path: &String, data: String) {
    let pathbuf = PathBuf::from(&path);
    let dirname = pathbuf.parent().unwrap().to_str().unwrap();
    let _ = std::fs::create_dir_all(dirname);
    std::fs::write(path, data).unwrap();
}
