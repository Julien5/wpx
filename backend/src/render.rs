#![allow(non_snake_case)]

use crate::backend::BackendData;
use crate::{track, waypoint};

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
        let info = &waypoints[k].info.as_ref().unwrap();
        let mut copy = template_line.clone();
        copy = copy.replace("{name}", info.name.as_str());
        let datetime = chrono::DateTime::parse_from_rfc3339(info.time.as_str()).unwrap();
        let time_str = format!("{}", datetime.format("%H:%M"));

        copy = copy.replace("{time}", &time_str);

        copy = copy.replace(
            "{distance}",
            format!("{:4.0}", info.distance / 1000f64).as_str(),
        );
        let elevation = info.elevation;
        copy = copy.replace("{elevation}", format!("{:5.0} m", elevation).as_str());
        let hm = info.inter_elevation_gain;
        let percent = 100f64 * info.inter_slope;
        copy = copy.replace("{d+}", format!("{:5.0}", hm).as_str());
        copy = copy.replace("{slope}", format!("{:2.1}%", percent).as_str());
        copy = copy.replace("{desc}", info.description.as_str());
        let dist = info.inter_distance / 1000f64;
        copy = copy.replace("{dist}", format!("{:2.1}", dist).as_str());
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

pub fn make_typst_document(backend: &mut BackendData) -> String {
    let debug = backend.get_parameters().debug;
    let templates = Templates::new();
    let mut document = templates.header.clone();
    let segments = backend.segments();
    for segment in &segments {
        let range = segment.range();
        if range.is_empty() {
            break;
        }
        let mut waypoints_table = backend.get_waypoint_table(&segment);
        waypoints_table.truncate(15);
        log::trace!("segment: {}", segment.id);
        log::trace!("points table");
        let table = points_table(&templates, &backend.track, &waypoints_table);
        log::trace!("render profile");
        let rendered_profile = segment.render_profile();
        if backend.get_parameters().debug {
            let f = format!("/tmp/segment-{}.svg", segment.id);
            std::fs::write(&f, &rendered_profile.svg).unwrap();
        }
        log::trace!("render map");
        let map_options = &backend.get_parameters().map_options;
        let m = segment.render_map(&map_options.size2d());
        if debug {
            let f = format!("/tmp/map-{}.svg", segment.id);
            std::fs::write(&f, &m).unwrap();
        }
        log::trace!("link");
        link(&templates, &rendered_profile.svg, &m, &table, &mut document);
        if range.end == backend.track.len() {
            break;
        }
    }
    document
}
