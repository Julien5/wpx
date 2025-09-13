use std::path::{Path, PathBuf};

pub fn read(path: &String) -> Option<String> {
    let path = Path::new(path.as_str());
    if !path.exists() {
        return None;
    }
    match std::fs::read_to_string(path) {
        Ok(data) => Some(data),
        _ => None,
    }
}

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
