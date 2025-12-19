use crate::{
    backend::Segment,
    inputpoint::{InputPoint, InputType, OSMType},
    label_placement::features::PointFeatureDrawing,
    math::Point2D,
    speed,
};

pub fn timestr(w: &InputPoint, segment: &Segment) -> String {
    let index = w.round_track_index().unwrap();
    let track = &segment.track;
    let t = speed::time_at_distance(&track.distance(index), &segment.parameters);
    format!("{}", t.format("%H:%M"))
}

pub fn make_label_text(w: &InputPoint, segment: &Segment) -> String {
    match w.kind() {
        InputType::OSM => {
            return w.name().clone().trim().to_string();
        }
        InputType::UserStep => {
            return format!("{}", timestr(w, segment));
        }
        InputType::GPX => {
            return format!("{} ({})", w.name(), timestr(w, segment));
        }
        InputType::Control => {
            return format!("{} ({})", w.name(), timestr(w, segment));
        }
    }
}

fn make_circle(
    center: &Point2D,
    id: &String,
    fill: &str,
    stroke_width: &f64,
    stroke_color: &str,
) -> svg::node::element::Circle {
    let mut ret = svg::node::element::Circle::new();
    ret = ret.set("id", format!("{}", id));
    ret = ret.set("cx", format!("{}", center.x));
    ret = ret.set("cy", format!("{}", center.y));
    ret = ret.set("fill", format!("{}", fill));
    if *stroke_width > 0.0 {
        ret = ret.set("stroke", format!("{}", stroke_color));
        ret = ret.set("stroke-width", format!("{}", stroke_width));
    }
    ret
}

pub fn draw_for_profile(center: &Point2D, id: &str, w: &InputPoint) -> PointFeatureDrawing {
    let (r, fill) = match w.kind() {
        InputType::OSM => match w.osmkind().unwrap() {
            OSMType::City => (5f64, "Black"),
            OSMType::Village => (4f64, "Black"),
            OSMType::Hamlet => (2f64, "Gray"),
            OSMType::MountainPass => (3f64, "Green"),
            OSMType::Peak => (3f64, "Green"),
        },
        InputType::GPX => (5f64, "Blue"),
        InputType::UserStep => (3f64, "Black"),
        InputType::Control => (5f64, "Blue"),
    };

    let mut circle = make_circle(center, &format!("{}", id), fill, &0.0, "");
    circle = circle.set("r", format!("{}", r));

    let mut group = svg::node::element::Group::new();
    group = group.add(circle);

    match w.kind() {
        InputType::OSM => {
            let osm = w.osmkind().unwrap();
            if osm == OSMType::City || osm == OSMType::Village || osm == OSMType::Hamlet {
                let mut white =
                    make_circle(center, &format!("{}-little-white", id), "white", &0.0, "");
                white = white.set("r", format!("{}", (r - 1.5).max(0.0)));
                group = group.add(white);

                if osm == OSMType::City {
                    let mut black =
                        make_circle(center, &format!("{}-little-white", id), "black", &0.0, "");
                    black = black.set("r", format!("{}", (r - 2.5).max(0.0)));
                    group = group.add(black);
                }
            }
        }
        _ => {}
    }

    PointFeatureDrawing {
        group,
        center: center.clone(),
    }
}

pub fn draw_for_map(point: &Point2D, id: &str, w: &InputPoint) -> PointFeatureDrawing {
    draw_for_profile(point, id, w)
}
