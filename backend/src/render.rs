#![allow(non_snake_case)]

use crate::backend::Backend;
use crate::render_device::RenderDevice;
use crate::waypoint;
use crate::{gpsdata, svgmap};

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
    _track: &gpsdata::Track,
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

fn get_typst_bytes(ascii: &str) -> String {
    let mut ret = Vec::new();
    for c in ascii.chars() {
        let code = format!("{:?}", c as u32);
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
    document.push_str(table.as_str());
}

pub fn make_typst_document(backend: &mut Backend, (W, H): (i32, i32)) -> String {
    let debug = backend.get_parameters().debug;
    let templates = Templates::new();
    let mut document = templates.header.clone();
    let mut start = 0f64;
    let mut k = 0usize;
    let segments = backend.segments();
    let waypoints = backend.get_waypoints();
    for segment in &segments {
        let range = &segment.range;
        if range.is_empty() {
            break;
        }
        let waypoints_table = backend.get_waypoint_table(&segment);
        let table = points_table(&templates, &backend.track, &waypoints_table);
        let p = backend.render_segment(segment, (W, H), RenderDevice::PDF);
        if backend.get_parameters().debug {
            let f = format!("/tmp/segment-{}.svg", segment.id);
            std::fs::write(&f, &p).unwrap();
        }
        let Wm = 400i32;
        let Hm = 400i32;
        let m = svgmap::map(&backend.track, &waypoints, &segment, Wm, Hm, debug);
        if debug {
            let f = format!("/tmp/map-{}.svg", segment.id);
            std::fs::write(&f, &m).unwrap();
        }
        link(&templates, &p, &m, &table, &mut document);
        if range.end == backend.track.len() {
            break;
        }
        start = start + backend.get_parameters().segment_length;
        k = k + 1;
    }
    document
}
