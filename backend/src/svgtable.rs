use crate::format::round_time;
use crate::pdf::render::TableInfo;
use crate::{label_placement::features::text_width, waypoint::WaypointInfo};
use chrono::DateTime;

const FONT_SIZE: f64 = 2.5f64;
const FONT_WEIGHT: &str = "normal";
const FONT_STYLE: &str = "normal";
const HEADER_FONT_WEIGHT: &str = "bold";
const PADDING: f64 = 1.5;
const COL_DIST_W: f64 = 8.0;
const COL_TIME_W: f64 = 8.0;

pub fn name_description(w: &WaypointInfo, max_width: f64) -> String {
    let mut ret = match (w.name.is_empty(), w.description.is_empty()) {
        (false, false) => format!("{}, {}", w.name, w.description),
        (true, false) => format!("{}", w.description),
        (false, true) => format!("{}", w.name),
        (true, true) => format!("unknown"),
    };
    while text_width(&ret, FONT_SIZE, FONT_WEIGHT, FONT_STYLE) > max_width {
        ret.pop();
    }
    ret
}

pub fn waypoints_to_svg(table_info: TableInfo, row_height_mm: f64) -> String {
    let waypoint_infos: Vec<_> = table_info
        .waypoints
        .iter()
        .map(|w| w.get_info().clone())
        .collect();

    let show_name = true; //waypoints.iter().any(|w| !w.name.is_empty());
    let show_desc = false; //waypoints.iter().any(|w| !w.description.is_empty());

    let user_step_info = if table_info.user_steps_options.step_distance.is_some() {
        format!(
            "(cutoffs every {:.0} km)",
            table_info.user_steps_options.step_distance.unwrap() / 1000f64
        )
    } else if table_info.user_steps_options.step_elevation_gain.is_some() {
        format!(
            "(cutoffs every {:.0} m)",
            table_info.user_steps_options.step_elevation_gain.unwrap()
        )
    } else {
        String::new()
    };
    let info_line = format!(
        "{:.0} m elevation gain {}",
        table_info.elevation_gain, user_step_info
    );
    let info_line_w = text_width(&info_line, FONT_SIZE, HEADER_FONT_WEIGHT, FONT_STYLE);
    let col_name_w = if show_name {
        let data_w = waypoint_infos
            .iter()
            .map(|w| text_width(&w.name, FONT_SIZE, FONT_WEIGHT, FONT_STYLE))
            .fold(0.0_f64, f64::max)
            .max(info_line_w)
            .min(70f64);
        let header_w = text_width("NAME", FONT_SIZE, HEADER_FONT_WEIGHT, FONT_STYLE);
        data_w.max(header_w) + 2.0 * PADDING
    } else {
        0.0
    };

    let col_desc_w = if show_desc {
        let data_w = waypoint_infos
            .iter()
            .map(|w| text_width(&w.description, FONT_SIZE, FONT_WEIGHT, FONT_STYLE))
            .fold(0.0_f64, f64::max);
        let header_w = text_width("DESCRIPTION", FONT_SIZE, HEADER_FONT_WEIGHT, FONT_STYLE);
        data_w.max(header_w) + 2.0 * PADDING
    } else {
        0.0
    };

    let x_dist = 0.0_f64;
    let x_time = x_dist + COL_DIST_W;
    let x_name = x_time + COL_TIME_W;
    let x_desc = x_name + col_name_w;
    let total_width = x_desc + col_desc_w;

    // +1 row for the header
    let total_height = row_height_mm * (waypoint_infos.len() + 1) as f64;

    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg"
     width="{total_width}mm" height="{total_height}mm"
     viewBox="0 0 {total_width} {total_height}">
  <style>
    text {{ font-family: serif; font-size: {FONT_SIZE}px; }}
  </style>
"##
    );

    // --- Header row ---
    let header_y_top = 0.0_f64;
    let header_y_baseline = header_y_top + row_height_mm * 0.72;

    // "KM" — centered in distance column
    let dist_x = x_dist + COL_DIST_W / 2.0;
    svg.push_str(&format!(
        r##"  <text x="{dist_x}" y="{header_y_baseline}" text-anchor="middle" font-weight="{HEADER_FONT_WEIGHT}">KM</text>"##
    ));
    svg.push('\n');

    // "TIME" — centered in time column
    let time_x = x_time + COL_TIME_W / 2.0;
    svg.push_str(&format!(
        r##"  <text x="{time_x}" y="{header_y_baseline}" text-anchor="middle" font-weight="{HEADER_FONT_WEIGHT}">TIME</text>"##
    ));
    svg.push('\n');

    let name_x = x_name + PADDING;

    svg.push_str(&format!(
        r##"  <text x="{name_x}" y="{header_y_baseline}" text-anchor="start" font-weight="{HEADER_FONT_WEIGHT}">{info_line}</text>"##
        ));
    svg.push('\n');

    // "DESCRIPTION" — left-aligned in description column
    if show_desc {
        let desc_x = x_desc + PADDING;
        svg.push_str(&format!(
            r##"  <text x="{desc_x}" y="{header_y_baseline}" text-anchor="start" font-weight="{HEADER_FONT_WEIGHT}">DESCRIPTION</text>"##
        ));
        svg.push('\n');
    }

    // --- Data rows (offset by one row_height_mm) ---
    for (i, wp) in waypoint_infos.iter().enumerate() {
        let y_top = (i + 1) as f64 * row_height_mm;
        let y_baseline = y_top + row_height_mm * 0.72;

        /*
            if i % 2 == 0 {
                svg.push_str(&format!(
                    r##"  <rect x="{x_dist}" y="{y_top}" width="{total_width}" height="{row_height_mm}" fill="gray"/>"##
                ));
                svg.push('\n');
        }*/
        svg.push_str(&format!(
			r##"  <line x1="{x_dist}" y1="{y_top}" x2="{total_width}" y2="{y_top}" stroke="black" stroke-width="0.2"/>"##
    ));
        svg.push('\n');

        let dist_km = wp.distance / 1000.0;
        let dist_str = format!("{:.1}", dist_km);
        let dist_x = x_dist + COL_DIST_W / 2.0;
        svg.push_str(&format!(
            r##"  <text x="{dist_x}" y="{y_baseline}" text-anchor="middle">{dist_str}</text>"##
        ));
        svg.push('\n');

        let time_str = DateTime::parse_from_rfc3339(&wp.time)
            .map(|dt| round_time(&dt.into()).format("%H:%M").to_string())
            .unwrap_or_else(|_| "--:--".to_string());
        let time_x = x_time + COL_TIME_W / 2.0;
        svg.push_str(&format!(
            r##"  <text x="{time_x}" y="{y_baseline}" text-anchor="middle">{time_str}</text>"##
        ));
        svg.push('\n');

        if show_name {
            let name_x = x_name + PADDING;
            let text = name_description(&wp, col_name_w);
            svg.push_str(&format!(
                r##"  <text x="{name_x}" y="{y_baseline}" text-anchor="start">{}</text>"##,
                escape_xml(&text)
            ));
            svg.push('\n');
        }

        if show_desc {
            let desc_x = x_desc + PADDING;
            svg.push_str(&format!(
                r##"  <text x="{desc_x}" y="{y_baseline}" text-anchor="start">{}</text>"##,
                escape_xml(&wp.description)
            ));
            svg.push('\n');
        }
    }

    // --- Column separator lines (full height including header) ---
    let draw_vline = |x: f64| -> String {
        format!(
            r##"  <line x1="{x}" y1="0" x2="{x}" y2="{total_height}" stroke="black" stroke-width="0.2"/>"##
        )
    };
    svg.push_str(&draw_vline(x_time));
    svg.push('\n');
    if show_name {
        svg.push_str(&draw_vline(x_name));
        svg.push('\n');
    }
    if show_desc {
        svg.push_str(&draw_vline(x_desc));
        svg.push('\n');
    }

    // Horizontal line separating header from data
    svg.push_str(&format!(
        r##"  <line x1="0" y1="{row_height_mm}" x2="{total_width}" y2="{row_height_mm}" stroke="black" stroke-width="0.2"/>"##
    ));
    svg.push('\n');

    // Outer border
    svg.push_str(&format!(
        r##"  <rect x="0" y="0" width="{total_width}" height="{total_height}" fill="none" stroke="black" stroke-width="0.5"/>"##
    ));
    svg.push('\n');

    svg.push_str("</svg>\n");
    svg
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
