use std::future::Future;

use reqwest::Client;

use crate::{
    error::{GenericResult, TrackError},
    event::{self},
    osm::DownloadSideData,
};

use log;

#[cfg(target_arch = "wasm32")]
fn use_disk() -> bool {
    false
}

/* For debugging:
 * use_disk = true => write download in /tmp/last-dl.data
 *                 => read download from /tmp/dl.data if exists
 */

#[cfg(not(target_arch = "wasm32"))]
fn use_disk() -> bool {
    // true
    false
}

async fn handle_response(response: reqwest::Response) -> GenericResult<String> {
    log::info!("http response status = {}", response.status());
    if response.status() == 504 {
        return Err(TrackError::OSMDownloadTimeout.into());
    }
    if response.status() != 200 {
        return Err(TrackError::OSMDownloadFailed.into());
    }
    let text = response.text().await;
    if use_disk() {
        let filename = std::format!("/tmp/last-dl.data");
        let data = text.as_ref().unwrap().clone();
        // write overwrites.
        std::fs::write(filename, data).expect("Unable to write file");
    }
    match text {
        Ok(json) => Ok(json),
        Err(e) => Err(e.into()),
    }
}

#[allow(dead_code)]
fn fake_request(ms: u64) -> impl Future<Output = Result<reqwest::Response, reqwest::Error>> {
    async move {
        // 1. Wait for the specified duration
        tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
        // 2. Construct and return a simulated reqwest Error.
        let simulated_error = reqwest::Client::new()
            .get("") // Invalid empty URL causes an instant builder error
            .build() // Or we can let the builder fail
            .unwrap_err();

        Err(simulated_error)
    }
}

pub async fn dl_worker(req: &str, side: &DownloadSideData<'_>) -> GenericResult<String> {
    log::info!("download:{}", req);
    let url = "https://overpass-api.de/api/interpreter";
    // let url = "https://overpass.private.coffee/api/interpreter";
    let client = Client::new();
    let request = client
        .post(url)
        .header("User-Agent", "julien5/WPX")
        .header("Accept", "*/*")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Accept-Encoding", "gzip, deflate, br, zstd")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("Origin", "https://overpass-turbo.eu")
        .header("Connection", "keep-alive")
        .header("Referer", "-")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "cross-site")
        .header("Priority", "u=0")
        .body(format!("data={}", urlencoding::encode(&req)));
    log::debug!("request={:?}", request);
    event::send_worker(&side.logger, &format!("{}", "osm:wait-for-response"));
    //let tick = tokio::time::Duration::from_millis(750);
    //tokio::time::sleep(tick).await;
    let future = request.send();
    // let future = fake_request(500);
    tokio::select! {
        response = future => {
            match response {
                Ok(resp) => handle_response(resp).await,
                Err(e) => {
                    Err(e.into())
                },
            }
        }
        _ = side.cancel_token.cancelled() => {
            return Err(TrackError::OSMDownloadCancelled.into());
        }
    }
    /*
    match future.await {
        Ok(response) => handle_response(response).await,
        Err(e) => {
            log::trace!("e = {}", e);
            Err(e.into())
        }
    }*/
}
