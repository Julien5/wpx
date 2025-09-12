use std::path::{Path, PathBuf};

use super::osmpoint::*;
use crate::track::WGS84BoundingBox;

#[cfg(test)]
fn cache_dir() -> String {
    "data/ref/cache".to_string()
}

#[cfg(not(test))]
fn cache_dir() -> String {
    dirs::cache_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn cache_filename(bbox: &WGS84BoundingBox, kind: &str) -> String {
    let mut s = super::osm3(bbox).to_string();
    // (49.000,8.400,49.100,8.500)
    // ->49.000/8.400/49.100/8.500
    s = s.replace("(", "");
    s = s.replace(")", "");
    s = s.replace(",", "/");

    let root = format!("{}/{}", cache_dir(), "WPX");
    format!("{}/{}/{}", root, s, kind)
}

pub(crate) fn read_chunk_cache(
    bbox: &WGS84BoundingBox,
    kind: &str,
) -> Option<super::osmpoint::OSMPoints> {
    let filename = cache_filename(bbox, kind);
    let path = Path::new(filename.as_str());
    if !path.exists() {
        return None;
    }
    match std::fs::read_to_string(path) {
        Ok(data) => Some(OSMPoints::from_string(&data)),
        _ => None,
    }
}

pub(crate) fn write_chunk_cache(bboxes: &Vec<WGS84BoundingBox>, points: &OSMPoints, kind: &str) {
    for atom in bboxes {
        let local = points
            .clone()
            .points
            .iter()
            .filter(|p| {
                let coord = (p.lon, p.lat);
                atom.contains(&coord)
            })
            .cloned()
            .collect::<Vec<_>>();
        let path = cache_filename(atom, kind);
        let pathbuf = PathBuf::from(&path);
        let dirname = pathbuf.parent().unwrap().to_str().unwrap();
        let _ = std::fs::create_dir_all(dirname);
        let out = OSMPoints { points: local };
        std::fs::write(path, out.as_string()).unwrap();
    }
}
