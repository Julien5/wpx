use reqwest::Client;
use serde_json::Value;

use crate::{
    inputpoint::{InputPoint, InputPoints, Tags},
    mercator,
    wgs84point::WGS84Point,
};

use log;

async fn dl_worker(req: &str) -> Option<String> {
    log::info!("download:{}", req);
    let url = "https://overpass-api.de/api/interpreter";
    let client = Client::new();
    let response = client
        .post(url)
        .header("User-Agent", "jbo/WPX")
        .header("Accept", "*/*")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Accept-Encoding", "gzip, deflate, br, zstd")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("Origin", "https://overpass-turbo.eu")
        .header("Connection", "keep-alive")
        .header("Referer", "https://overpass-turbo.eu/")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "cross-site")
        .header("Priority", "u=0")
        .body(format!("data={}", urlencoding::encode(&req)))
        .send()
        .await
        .unwrap();

    let text = response.text().await;

    /*
        if true {
            let filename = std::format!("/tmp/dl.data");
            let data = text.as_ref().unwrap().clone();
            std::fs::write(filename, data).expect("Unable to write file");
    }
        */

    match text {
        Ok(json) => Some(json),
        Err(_) => None,
    }
}

/*

Grabener Höhe is tourism = viewpoint.
To get it: node["tourism"="viewpoint"]({{bbox}});
*/

pub async fn all(bbox: &str) -> Option<String> {
    let timeout = 250;
    let header = format!("[out:json][timeout:{}]", timeout);
    let mut reqs = Vec::new();
    reqs.push(format!("node[\"mountain_pass\"=\"yes\"]{}", bbox));
    reqs.push(format!("node[\"natural\"=\"peak\"]{}", bbox));
    reqs.push(format!("nwr[\"place\"=\"city\"]{}", bbox));
    reqs.push(format!("nwr[\"place\"=\"town\"]{}", bbox));
    reqs.push(format!("nwr[\"place\"=\"village\"]{}", bbox));
    reqs.push(format!("nwr[\"place\"=\"hamlet\"]{}", bbox));
    let body = reqs.join(";");
    let footer = "out geom".to_string();
    let request = format!("{};({};);{};", header, body, footer);
    dl_worker(&request).await
}

fn read_f64(map: &serde_json::Map<String, Value>, name: &str) -> f64 {
    map.get(name).unwrap().as_f64().unwrap()
}

fn read_tags(tags: &serde_json::Value) -> Tags {
    let mut ret = Tags::new();
    let map = tags.as_object().unwrap();
    for (k, v) in map {
        match v.as_str() {
            Some(text) => {
                ret.insert(k.to_string(), text.to_string());
            }
            None => {}
        }
    }
    ret
}

fn read_download_element(
    element: &serde_json::Value,
    projection: &mercator::WebMercatorProjection,
) -> Result<InputPoint, String> {
    assert!(element.is_object());
    let map = element.as_object().unwrap();
    match map.get("type") {
        Some(value) => {
            if value != "node" {
                return Err(format!("found {} (no node)", value));
            }
        }
        None => {
            return Err(format!("no city"));
        }
    }
    let lat = read_f64(map, "lat");
    let lon = read_f64(map, "lon");
    let tags = read_tags(map.get("tags").unwrap());
    let wgs = WGS84Point::new_lonlat(&lon, &lat);
    let euclidean = projection.project(&wgs);
    let ret = InputPoint {
        wgs84: wgs,
        euclidian: euclidean,
        tags,
        track_projection: None,
        label_placement_order: i32::MAX,
    };
    Ok(ret)
}

fn read_downloaded_elements(elements: &serde_json::Value) -> InputPoints {
    assert!(elements.is_array());
    let mut ret = Vec::new();
    let projection = mercator::WebMercatorProjection::make();
    for e in elements.as_array().unwrap() {
        match read_download_element(e, &projection) {
            Ok(point) => ret.push(point),
            Err(_msg) => {
                //log::info!("{} with {}", msg, e);
            }
        }
    }
    InputPoints { points: ret }
}

pub fn parse_osm_content(content: &[u8]) -> serde_json::Result<InputPoints> {
    let json: serde_json::Value = serde_json::from_slice(content)?;
    assert!(json.is_object());
    //assert!(json.as_object().unwrap().len() == 1);
    let mut ret = Vec::new();
    let map = json.as_object().unwrap();
    ret.extend(read_downloaded_elements(map.get("elements").unwrap()).points);
    Ok(InputPoints { points: ret })
}

#[cfg(test)]
mod tests {
    use crate::track::WGS84BoundingBox;

    use super::*;
    #[test]
    fn download() {
        //let bbox = "(47.86,9.66,48.17,10.80)";
        let bbox = WGS84BoundingBox::init((47.86, 9.66), (48.17, 10.80));
        let place = "village";
        let json = ""; //dl(&bbox, place).unwrap();
        let json = ""; //dl_passes(&bbox).unwrap();
        log::info!("ret={}", json);
    }
}
