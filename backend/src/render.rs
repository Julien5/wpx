#![allow(non_snake_case)]

use euclid::Size2D;

use crate::backend::Backend;
use crate::point_collection::Kind;
use crate::waypoint::decimate;
use crate::{track, waypoint};

use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;

struct Templates {
    header: String,
    table_large: String,
    table_points: String,
}

impl Templates {
    fn new() -> Templates {
        Templates {
            header: String::from_str(include_str!("../templates/header.typ")).unwrap(),
            table_large: String::from_str(include_str!("../templates/table-large.typ")).unwrap(),
            table_points: String::from_str(include_str!("../templates/table-points.typ")).unwrap(),
        }
    }
}

fn points_table(
    templates: &Templates,
    _track: &track::Track,
    waypoints: &Vec<waypoint::Waypoint>,
) -> String {
    let table = templates.table_points.clone();
    let mut template_line_orig = String::new();
    let mut template_line = String::new();
    for line in table.split("\n") {
        if line.contains("/* #line-template") {
            template_line_orig = String::from_str(line).unwrap();
            template_line = template_line_orig.clone();
            template_line = template_line.replace("/* #line-template", "");
            template_line = template_line.replace("*/", "");
        }
    }
    debug_assert!(!template_line.is_empty());
    // TODO: avoid recomputing the automatic points
    let mut lines = Vec::new();
    for k in 0..waypoints.len() {
        let info = &waypoints[k].get_info();
        let mut copy = template_line.clone();

        copy = copy.replace("{name}", &info.name);
        copy = copy.replace("{description}", &info.description);
        let datetime = chrono::DateTime::parse_from_rfc3339(&info.time).unwrap();
        let time_str = format!("{}", datetime.format("%H:%M"));
        copy = copy.replace("{time}", &time_str);

        let dist = info.distance / 1000f64;
        copy = copy.replace("{distance}", format!("{:2.1}", dist).as_str());
        lines.push(copy);
    }
    let joined = lines.join("\n");
    table.replace(&template_line_orig, joined.as_str())
}

fn get_typst_bytes(utf8: &str) -> String {
    let mut ret = Vec::new();
    let chars = utf8.as_bytes();
    for c in chars {
        let code = format!("{}", *c as u32);
        ret.push(code);
    }
    let rc = ret.join(",");
    format!("bytes(({}))", rc)
}

fn link(
    templates: &Templates,
    profilesvg: &str,
    mapsvg: &str,
    points_table: &String,
    document: &mut String,
) {
    let mut table = templates.table_large.clone();
    table = table.replace("{table-points}", points_table.as_str());
    table = table.replace("{profile.svg}", get_typst_bytes(profilesvg).as_str());
    table = table.replace("{map.svg}", get_typst_bytes(mapsvg).as_str());
    //table = table.replace("{map.svg}", format!("\"{}\"", "map-0.svg").as_str());
    document.push_str(table.as_str());
}

pub fn make_typst_document(backend: &Backend) -> String {
    let debug = backend.get_parameters().debug;
    let templates = Templates::new();
    let mut document = templates.header.clone();
    let fsegments = backend.segments();
    let segments: Vec<_> = fsegments
        .iter()
        .map(|f| backend.make_segment_data(&f))
        .collect();

    let controls = HashSet::from([Kind::Controls, Kind::GPXWaypoints]);
    let mut all_points = BTreeMap::new();
    for segment in &fsegments {
        let mut segment_waypoints = backend.get_points(&segment, controls.clone());
        if segment_waypoints.is_empty() {
            segment_waypoints = backend.get_points(&segment, HashSet::new());
        }
        for w in segment_waypoints {
            for proj in &w.track_projections {
                let index = proj.track_index;
                all_points.insert(index, w.clone());
            }
        }
    }

    let allkinds = HashSet::from([
        Kind::UserStep,
        Kind::GPXWaypoints,
        Kind::Controls,
        Kind::Cities,
        Kind::Villages,
        Kind::Mountains,
        Kind::Hamlets,
    ]);
    let allkinds = HashSet::from([Kind::GPXWaypoints]);
    for segment in &segments {
        let range = segment.range();
        if range.is_empty() {
            continue;
        }

        let profile_size = Size2D::new(1000, 300);
        let map_size = Size2D::new(400, 400);
        let both = backend.render_segment_map_profile(
            &segment.segment,
            &map_size,
            &profile_size,
            allkinds.clone(),
        );
        let [rendered_map, rendered_profile]: [_; 2] = both.try_into().unwrap();
        let waypoints_table = &rendered_profile.waypoints;
        let waypoints_table = decimate(&segment.segment, &waypoints_table, 15);
        log::trace!(
            "segment {} => {} points include",
            segment.id(),
            waypoints_table.len(),
        );
        let table = points_table(&templates, &backend.d().track, &waypoints_table);
        if backend.get_parameters().debug {
            let f = format!("/tmp/segment-{}.svg", segment.id());
            std::fs::write(&f, &rendered_profile.svg).unwrap();
        }
        if debug {
            let f = format!("/tmp/map-{}.svg", segment.id());
            std::fs::write(&f, &rendered_map.svg).unwrap();
        }
        link(
            &templates,
            &rendered_profile.svg,
            &rendered_map.svg,
            &table,
            &mut document,
        );
        if range.end == backend.d().track.len() {
            break;
        }
    }
    document
}
