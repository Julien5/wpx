use std::{fs, path::Path};

use crate::{bbox::BoundingBox, osm::request::Boxes};

pub fn paint_svg(boxes: &Boxes, bboxes: &Vec<BoundingBox>) -> String {
    if boxes.len() == 0 {
        return r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100%" height="100%" fill="black"/></svg>"#.to_string();
    }

    // 1. Determine absolute bounds across all items to establish a clean view bounding box
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for bbox in boxes.bboxes() {
        min_x = min_x.min(bbox.get_min().x);
        min_y = min_y.min(bbox.get_min().y);
        max_x = max_x.max(bbox.get_max().x);
        max_y = max_y.max(bbox.get_max().y);
    }

    // 2. Setup structural viewing scales and padding
    let data_width = max_x - min_x;
    let data_height = max_y - min_y;

    // Resolution target for output image size
    let svg_width = 1200.0;
    let svg_height = 1200.0;

    let pad = 40.0; // Inner border padding in SVG pixels
    let scale_x = (svg_width - pad * 2.0) / data_width;
    let scale_y = (svg_height - pad * 2.0) / data_height;

    // Maintain uniform aspect ratio to protect geometry from stretching
    let scale = scale_x.min(scale_y);

    // Coordinate projector closure mapping world units directly to local SVG space
    // Standard SVG files invert the standard Cartesian Y axis (0,0 is Top-Left)
    let project = |p_x: f64, p_y: f64| -> (f64, f64) {
        let x = pad + (p_x - min_x) * scale;
        let y = svg_height - pad - (p_y - min_y) * scale; // Inverted Y coordinate mapping
        (x, y)
    };

    // 3. Construct raw XML payload string
    let mut svg = String::new();

    // Document opening signature
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{:.0}" height="{:.0}">"#,
        svg_width, svg_height
    ));
    svg.push('\n');

    // Requirement (0): Solid Black Background fill
    svg.push_str(&format!(
        r#"  <rect width="{:.0}" height="{:.0}" fill="black" />"#,
        svg_width, svg_height
    ));
    svg.push('\n');

    // Requirement (1): Bounding boxes rendered as Gray shapes with thin Blue margins
    svg.push_str("  <!-- BOUNDING BOXES LAYER -->\n");
    for bbox in bboxes {
        let (x1, y1) = project(bbox.get_min().x, bbox.get_min().y);
        let (x2, y2) = project(bbox.get_max().x, bbox.get_max().y);

        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x1 - x2).abs();
        let height = (y1 - y2).abs();

        svg.push_str(&format!(
            r#"  <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="gray" stroke="dodgerblue" stroke-width="1.5" opacity="0.85" />"#,
            x, y, width, height
        ));
        svg.push('\n');
    }

    // Requirement (2): Raw internal grid tiles highlighted with thick Blue borders and transparent fills
    svg.push_str("  <!-- TILES FOREGROUND LAYER -->\n");
    for bbox in boxes.bboxes() {
        let p_min = bbox.get_min();
        let p_max = bbox.get_max();

        let (x1, y1) = project(p_min.x, p_min.y);
        let (x2, y2) = project(p_max.x, p_max.y);

        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x1 - x2).abs();
        let height = (y1 - y2).abs();

        svg.push_str(&format!(
            r#"  <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="none" stroke="red" stroke-width="1.0" />"#,
            x, y, width, height
        ));
        svg.push('\n');
    }

    // Wrap up XML DOM Tree structure
    svg.push_str("</svg>");
    svg
}

pub fn save_debug_svg_incrementally(svg_content: &str) -> std::io::Result<String> {
    let mut n = 0;
    let mut file_path;

    // 1. Loop until we find a filename that does not exist yet
    loop {
        let filename = format!("optimize-{}.svg", n);
        file_path = Path::new("/tmp").join(filename);

        if !file_path.exists() {
            break;
        }
        n += 1; // Increment and try the next slot
    }

    // 2. Write the content out to the safe, fresh path location
    fs::write(&file_path, svg_content)?;

    // Return the final path string so you can print it out to your log console
    Ok(file_path.to_string_lossy().into_owned())
}
