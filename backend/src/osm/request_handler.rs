use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{
    error::{GenericResult, TrackError},
    osm::{download, request::*, request_cache::*, request_parse::parse, DownloadSideData},
};

fn hash(data: &String) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let final_hash: u64 = hasher.finish();
    let hex_string = format!("{:x}", final_hash);
    let short_hash = &hex_string[0..4];
    format!("{}", short_hash)
}

async fn download(req_string: &String, side: &DownloadSideData<'_>) -> GenericResult<Vec<u8>> {
    //log::trace!("download:\n{}\n", req_string);
    let hash = hash(&req_string);

    let respfilename = format!("data/osm/response-{}.txt", hash);
    match std::fs::exists(&respfilename) {
        Ok(true) => {
            log::trace!("found response file {}", respfilename);
            return Ok(std::fs::read(&respfilename).unwrap());
        }
        _ => {}
    };
    log::trace!("not found {}", respfilename);

    let mut nretries = 0;
    loop {
        match download::dl_worker(&req_string, &side).await {
            Err(e) => {
                // todo: break on cancel
                log::error!("download failed, error = {}, retry = {}", e, nretries);
                if nretries > 3 {
                    return Err(e.into());
                }
                nretries += 1;
            }
            Ok(content) => {
                std::fs::write(format!("/tmp/request-{}.txt", hash), req_string).unwrap();
                std::fs::write(format!("/tmp/response-{}.txt", hash), &content).unwrap();
                return Ok(content.into_bytes());
            }
        }
    }
}

pub async fn get_response(
    request: &Request,
    side: &DownloadSideData<'_>,
) -> GenericResult<ChunkData> {
    let (chunk_data, missing_request) = read_cache(request).await;
    if missing_request.boxes.is_empty() {
        log::trace!("complete cache hit.");
        return Ok(chunk_data);
    }

    log::trace!("incomplete cache hit.");
    for (missing_zones, missing_req_string) in missing_request.strings() {
        log::trace!(
            "request with {} missing tile bboxes",
            missing_zones.tiles.len()
        );
        log::trace!(
            "request with {} missing chunk bboxes",
            missing_zones.chunks.len()
        );
        match download(&missing_req_string, &side).await {
            Ok(data) => {
                log::trace!("response length: {} bytes", data.len());
                match parse(&data) {
                    Ok(response) => {
                        let _ =
                            super::request_cache::write_cache(&missing_request, &response).await;
                    }
                    Err(e) => {
                        log::error!("could not parse response: {:?}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("error:{:?}", e);
            }
        }
    }

    let (chunk_data, missing) = super::request_cache::read_cache(&request).await;
    if missing.boxes.is_empty() {
        log::trace!("complete cache hit.");
        return Ok(chunk_data);
    }
    log::trace!("incomplete cache hit.");
    Err(TrackError::OSMDownloadFailed.into())
}
