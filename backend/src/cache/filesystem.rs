use std::path::{Path, PathBuf};

use crate::error::GenericResult;

pub fn read(path: &str) -> GenericResult<String> {
    let path = Path::new(path);
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

pub fn write(path: &str, data: String) {
    let pathbuf = PathBuf::from(path);
    let dirname = pathbuf.parent().unwrap().to_str().unwrap();
    let _ = std::fs::create_dir_all(dirname);
    std::fs::write(path, data).unwrap();
}
