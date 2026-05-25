use std::collections::BTreeMap;

use crate::{
    bbox::BoundingBox,
    osm::request::Zones,
    point_collection::{Kind, Kinds},
};

pub type DensityMap = BTreeMap<Kind, f64>;

pub fn filter_string(kinds: &Kinds) -> Vec<String> {
    let mut ret = Vec::new();
    let mut place_values: Vec<&str> = Vec::new();
    let passes = r#"["mountain_pass"="yes"]"#.to_string();
    let peaks = r#"["natural"="peak"]"#.to_string();
    for kind in kinds {
        match kind {
            Kind::Cities => place_values.push("city|town"),
            Kind::Villages => place_values.push("village"),
            Kind::Hamlets => place_values.push("hamlet"),
            Kind::Controls => place_values.push("locality"),
            Kind::Mountains => {
                ret.push(passes.clone());
                ret.push(peaks.clone());
            }
            _ => {}
        }
    }

    if !place_values.is_empty() {
        ret.insert(0, format!(r#"["place"~"^({})$"]"#, place_values.join("|")));
    }
    ret
}

pub fn tile_kinds() -> Kinds {
    Kinds::from([Kind::Hamlets, Kind::Villages, Kind::Mountains])
}

pub fn chunk_kinds() -> Kinds {
    Kinds::from([Kind::Cities])
}

fn calculate_bbox_weight(
    bbox: &BoundingBox,
    kinds: &Kinds,
    density_map: &BTreeMap<Kind, f64>,
) -> f64 {
    let area_km2 = bbox.area() / 1_000_000f64;
    /*
    if kinds.contains(&Kind::Cities) {
        return 50f64;
    }
    return 5f64;
    */
    kinds
        .iter()
        .map(|kind| area_km2 * density_map.get(kind).unwrap_or(&0.0))
        .sum()
}

pub fn split_zones(zones: Zones, density_map: &DensityMap, max: f64) -> Vec<Zones> {
    let mut result = Vec::new();
    let mut current_zones = Zones::default();
    let mut current_weight = 0.0;

    // 1. Process Chunks
    for bbox in zones.chunks {
        let weight = calculate_bbox_weight(&bbox, &chunk_kinds(), density_map);

        if current_weight + weight > max
            && (!current_zones.chunks.is_empty() || !current_zones.tiles.is_empty())
        {
            result.push(std::mem::take(&mut current_zones));
            current_weight = 0.0;
        }

        current_zones.chunks.push(bbox);
        current_weight += weight;
    }

    // 2. Process Tiles
    for bbox in zones.tiles {
        let weight = calculate_bbox_weight(&bbox, &tile_kinds(), density_map);

        if current_weight + weight > max
            && (!current_zones.chunks.is_empty() || !current_zones.tiles.is_empty())
        {
            result.push(std::mem::take(&mut current_zones));
            current_weight = 0.0;
        }

        current_zones.tiles.push(bbox);
        current_weight += weight;
    }

    // 3. Push residual data
    if !current_zones.chunks.is_empty() || !current_zones.tiles.is_empty() {
        result.push(current_zones);
    }

    result
}
