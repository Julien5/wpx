mod filesystem;
#[cfg(target_arch = "wasm32")]
mod indexdb;

use crate::error::GenericResult;
use crate::inputpoint::InputPointMap;
use crate::tile::Chunk;

#[cfg(not(target_arch = "wasm32"))]
pub async fn write_worker(filename: &str, data: String) {
    filesystem::write(filename, data)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn read_worker(filename: &str) -> GenericResult<String> {
    filesystem::read(&filename)
}

#[cfg(target_arch = "wasm32")]
pub async fn write_worker(path: &str, data: String) {
    indexdb::write(&path, data).await
}

#[cfg(target_arch = "wasm32")]
pub async fn read_worker(path: &str) -> GenericResult<String> {
    match indexdb::read(path).await {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(e.into()),
    }
}

/*
#[cfg(not(target_arch = "wasm32"))]
async fn _hit_cache_worker(filename: &str) -> bool {
    filesystem::hit_cache(filename)
}

#[cfg(target_arch = "wasm32")]
async fn _hit_cache_worker(path: &String) -> bool {
    indexdb::hit_cache(&path).await
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
}*/
