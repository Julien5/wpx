use reqwest::blocking::Client;
use serde_json::Value;

use crate::osmpoint::{self, OSMPoint, OSMPoints};

fn dl_worker(req: &str) -> Option<String> {
    println!("download:{}", req);
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
        .unwrap();

    let text = response.text();

    if true {
        let filename = std::format!("/tmp/dl.data");
        let data = text.as_ref().unwrap().clone();
        std::fs::write(filename, data).expect("Unable to write file");
    }

    match text {
        Ok(json) => Some(json),
        Err(_) => None,
    }
}

pub fn places(bbox: &str, place: &str) -> Option<String> {
    let timeout = 250;
    let req = format!(
        "[out:json][timeout:{}];nwr[\"place\"=\"{}\"]{};out geom;",
        timeout, place, bbox
    );
    dl_worker(&req)
}

pub fn passes(bbox: &str) -> Option<String> {
    let timeout = 250;
    let req = format!(
        "[out:json][timeout:{}];node[mountain_pass=\"yes\"]{};out geom;",
        timeout, bbox
    );
    dl_worker(&req)
}

fn read_f64(map: &serde_json::Map<String, Value>, name: &str) -> f64 {
    map.get(name).unwrap().as_f64().unwrap()
}

fn read_tags(tags: &serde_json::Value) -> (Option<String>, Option<f64>) {
    let map = tags.as_object().unwrap();
    let mut name = None;
    for (k, v) in map {
        if k.contains("name") {
            name = Some(v.as_str().unwrap().to_string());
            break;
        }
    }
    let ele = match map.get("ele") {
        Some(value) => {
            let s = value.as_str().unwrap();
            match s.parse::<f64>() {
                Ok(f) => Some(f),
                Err(e) => {
                    println!("could not parse as f64: {} because {}", s, e);
                    None
                }
            }
        }
        None => None,
    };
    (name, ele)
}

fn read_download_element(element: &serde_json::Value) -> Result<OSMPoint, String> {
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
    let (name, ele) = read_tags(map.get("tags").unwrap());
    let city = OSMPoint {
        lat,
        lon,
        ele,
        name,
    };
    Ok(city)
}

fn read_downloaded_elements(elements: &serde_json::Value) -> OSMPoints {
    assert!(elements.is_array());
    let mut ret = Vec::new();
    for e in elements.as_array().unwrap() {
        match read_download_element(e) {
            Ok(city) => ret.push(city),
            Err(_msg) => {
                //println!("{} with {}", msg, e);
            }
        }
    }
    OSMPoints { points: ret }
}

pub fn parse_osm_content(content: &[u8]) -> serde_json::Result<OSMPoints> {
    let json: serde_json::Value = serde_json::from_slice(content)?;
    assert!(json.is_object());
    //assert!(json.as_object().unwrap().len() == 1);
    let mut ret = Vec::new();
    let map = json.as_object().unwrap();
    ret.extend(read_downloaded_elements(map.get("elements").unwrap()).points);
    Ok(osmpoint::OSMPoints { points: ret })
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
        println!("ret={}", json);
    }
}
