use crate::inputpoint::InputPoints;
use crate::track::WGS84BoundingBox;

#[cfg(test)]
fn cache_dir() -> String {
    "data/ref/cache".to_string()
}

#[cfg(not(test))]
fn cache_dir() -> String {
    let standart_cache_dir = dirs::cache_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string();
    return format!("{}/{}", standart_cache_dir, "WPX");
}

pub fn cache_filename(bbox: &WGS84BoundingBox) -> String {
    let mut s = super::osm3(bbox).to_string();
    // (49.000,8.400,49.100,8.500)
    // ->49.000/8.400/49.100/8.500
    s = s.replace("(", "");
    s = s.replace(")", "");
    s = s.replace(",", "/");

    format!("{}/{}", s, "data")
}

fn cache_path(filename: &String) -> String {
    format!("{}/{}", cache_dir(), filename)
}

#[cfg(not(target_arch = "wasm32"))]
async fn write_worker(filename: &String, data: String) {
    super::filesystem::write(&cache_path(filename), data)
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_worker(filename: &String) -> Option<String> {
    super::filesystem::read(&cache_path(filename))
}

#[cfg(target_arch = "wasm32")]
async fn write_worker(path: &String, data: String) {
    super::indexdb::write(&path, data).await
}

#[cfg(target_arch = "wasm32")]
async fn read_worker(path: &String) -> Option<String> {
    super::indexdb::read(&path).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn hit_cache_worker(filename: &String) -> bool {
    super::filesystem::hit_cache(&cache_path(filename))
}

#[cfg(target_arch = "wasm32")]
async fn hit_cache_worker(path: &String) -> bool {
    super::indexdb::hit_cache(&path).await
}

pub async fn hit_cache(bbox: &WGS84BoundingBox) -> bool {
    let filename = cache_filename(bbox);
    return hit_cache_worker(&filename).await;
}

pub async fn read(bbox: &WGS84BoundingBox) -> Option<InputPoints> {
    let filename = cache_filename(bbox);
    match read_worker(&filename).await {
        Some(data) => Some(InputPoints::from_string(&data)),
        _ => None,
    }
}

pub async fn write(bboxes: &Vec<WGS84BoundingBox>, points: &InputPoints) {
    for atom in bboxes {
        let local = points
            .clone()
            .points
            .iter()
            .filter(|p| {
                let coord = (p.wgs84.longitude(), p.wgs84.latitude());
                atom.contains(&coord)
            })
            .cloned()
            .collect::<Vec<_>>();
        let path = cache_filename(atom);
        let out = InputPoints { points: local };
        write_worker(&path, out.as_string()).await;
    }
}
