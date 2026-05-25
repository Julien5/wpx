use std::collections::BTreeMap;

use crate::{
    inputpoint::Tags,
    mercator::{self, WebMercatorProjection},
    tile::Tile,
    wgs84point::WGS84Point,
};
use serde_json::Value;

use super::request::*;

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

fn read_f64(map: &serde_json::Map<String, Value>, name: &str) -> f64 {
    map.get(name).unwrap().as_f64().unwrap()
}

fn read_feature(
    element: &serde_json::Value,
    projection: &WebMercatorProjection,
) -> Result<OSMFeature, String> {
    assert!(element.is_object());
    let feature = element.as_object().unwrap();
    match feature.get("type") {
        Some(value) => {
            if value != "node" {
                return Err(format!("found {} (no node)", value));
            }
        }
        None => {
            return Err(format!("no OSM type found"));
        }
    }
    let lat = read_f64(feature, "lat");
    let lon = read_f64(feature, "lon");
    let tags = read_tags(feature.get("tags").unwrap());
    let wgs = WGS84Point::new_lonlat(&lon, &lat);
    let euc = projection.project(&wgs);
    Ok(OSMFeature {
        id: feature["id"].to_string(),
        wgs84: wgs,
        euc: euc,
        tags: tags,
    })
}

fn read_downloaded_elements(elements: &serde_json::Value) -> BTreeMap<Tile, OSMFeatures> {
    assert!(elements.is_array());
    let mut ret = BTreeMap::new();
    let projection = mercator::WebMercatorProjection::make();
    for e in elements.as_array().unwrap() {
        match read_feature(e, &projection) {
            Ok(point) => {
                let euc = projection.project(&point.wgs84);
                let chunk = Tile::for_point(&euc);
                let ret_features = ret.entry(chunk).or_insert_with(Vec::new);
                ret_features.push(point);
            }
            Err(_msg) => {
                //log::info!("{} with {}", msg, e);
            }
        }
    }
    ret
}

pub fn parse(content: &[u8]) -> serde_json::Result<Response> {
    let json: serde_json::Value = serde_json::from_slice(content)?;
    assert!(json.is_object());
    //assert!(json.as_object().unwrap().len() == 1);
    let map = json.as_object().unwrap();
    let ret = read_downloaded_elements(map.get("elements").unwrap());
    Ok(Response { features: ret })
}
