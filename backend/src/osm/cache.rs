use crate::error::GenericResult;
use crate::inputpoint::InputPointMap;
use crate::tile::Chunk;
use std::env;
static OSM_CACHE_SCHEME_VERSION: usize = 1;

#[cfg(test)]
fn cache_dir() -> String {
    env::var("CACHE_DIR")
        .unwrap_or_else(|_| format!("data/ref/cache/osm/{}", OSM_CACHE_SCHEME_VERSION))
}

#[cfg(not(test))]
fn cache_dir() -> String {
    env::var("CACHE_DIR").unwrap_or_else(|_| {
        let standart_cache_dir = dirs::cache_dir()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        return format!(
            "{}/{}/osm/{}",
            standart_cache_dir, "WPX", OSM_CACHE_SCHEME_VERSION
        );
    })
}

fn cache_path(filename: &str) -> String {
    format!("{}/{}", cache_dir(), filename)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn write_worker(filename: &str, data: String) {
    super::filesystem::write(&cache_path(filename), data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn read_worker(filename: &str) -> GenericResult<String> {
    super::filesystem::read(&cache_path(filename))
}

#[cfg(target_arch = "wasm32")]
pub async fn write_worker(path: &str, data: String) {
    super::indexdb::write(&path, data).await
}

#[cfg(target_arch = "wasm32")]
pub async fn read_worker(path: &str) -> GenericResult<String> {
    match super::indexdb::read(path).await {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(e.into()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn _hit_cache_worker(filename: &str) -> bool {
    super::filesystem::hit_cache(&cache_path(filename))
}

#[cfg(target_arch = "wasm32")]
async fn _hit_cache_worker(path: &String) -> bool {
    super::indexdb::hit_cache(&path).await
}

async fn _valid_cache(key: &str) -> bool {
    match read_worker(key).await {
        Ok(data) => match InputPointMap::from_string(&data) {
            Ok(_map) => {
                return true;
            }
            _ => {
                log::info!("invalid cache at {}", key);
                return false;
            }
        },
        Err(_) => {
            panic!("this should not happen");
        }
    }
}

async fn _hit_cache(chunk: &Chunk) -> bool {
    let filename = chunk.basename();
    return _hit_cache_worker(&filename).await && _valid_cache(&filename).await;
}
