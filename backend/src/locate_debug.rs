use crate::mercator::MercatorPoint;

pub fn svg(
    track: &[MercatorPoint],
    begin: usize,
    end: usize,
    point: &MercatorPoint,
    indices: &[usize],
) -> String {
    let slice = &track[begin..=end];

    let all_x: Vec<f64> = slice.iter().map(|p| p.0).chain([point.0]).collect();
    let all_y: Vec<f64> = slice.iter().map(|p| p.1).chain([point.1]).collect();

    let min_x = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_x = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_y = all_y.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y = all_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let padding = 40.0_f64;
    let svg_w = 800.0_f64;
    let svg_h = 600.0_f64;
    let draw_w = svg_w - 2.0 * padding;
    let draw_h = svg_h - 2.0 * padding;

    let x_range = (max_x - min_x).max(1e-9);
    let y_range = (max_y - min_y).max(1e-9);

    let to_svg = |mp: &MercatorPoint| -> (f64, f64) {
        let sx = padding + (mp.0 - min_x) / x_range * draw_w;
        let sy = padding + (1.0 - (mp.1 - min_y) / y_range) * draw_h;
        (sx, sy)
    };

    let mut out = String::new();

    out.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_w}" height="{svg_h}" style="background:#1a1a2e;font-family:monospace">"##
    ));

    // --- Polyline (track) ---
    let points_attr: String = slice
        .iter()
        .map(|p| {
            let (sx, sy) = to_svg(p);
            format!("{:.2},{:.2}", sx, sy)
        })
        .collect::<Vec<_>>()
        .join(" ");

    out.push_str(&format!(
        r##"<polyline points="{points_attr}" fill="none" stroke="#00d4ff" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>"##
    ));

    // --- Track point dots + index labels ---
    let label_set: std::collections::HashSet<usize> = indices.iter().cloned().collect();

    for (i, mp) in slice.iter().enumerate() {
        let global_idx = begin + i;
        let (sx, sy) = to_svg(mp);

        out.push_str(&format!(
            r##"<circle cx="{:.2}" cy="{:.2}" r="3" fill="#00d4ff" opacity="0.7"/>"##,
            sx, sy
        ));

        if label_set.is_empty() || label_set.contains(&global_idx) {
            out.push_str(&format!(
                r##"<text x="{:.2}" y="{:.2}" fill="#e0e0ff" font-size="11" text-anchor="start" dominant-baseline="auto" dx="5" dy="-4">{}</text>"##,
                sx, sy, global_idx
            ));
        }
    }

    // --- Highlighted circle for `point` ---
    let (px, py) = to_svg(point);

    out.push_str(&format!(
        r##"<circle cx="{:.2}" cy="{:.2}" r="3" fill="none" stroke="#ff6b6b" stroke-width="1.5" opacity="0.4"/>"##,
        px, py
    ));
    out.push_str(&format!(
        r##"<circle cx="{:.2}" cy="{:.2}" r="2" fill="#ff6b6b" stroke="#ffffff" stroke-width="1.5"/>"##,
        px, py
    ));

    out.push_str("</svg>");
    out
}
