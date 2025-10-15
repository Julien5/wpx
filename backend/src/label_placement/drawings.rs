use crate::{inputpoint::InputType, label_placement::PointFeatureDrawing};

fn make_circle(
    (cx, cy): &(f64, f64),
    id: &String,
    fill: &str,
    stroke_width: &f64,
    stroke_color: &str,
) -> svg::node::element::Circle {
    let mut ret = svg::node::element::Circle::new();
    ret = ret.set("id", format!("{}", id));
    ret = ret.set("cx", format!("{}", cx));
    ret = ret.set("cy", format!("{}", cy));
    ret = ret.set("fill", format!("{}", fill));
    if *stroke_width > 0.0 {
        ret = ret.set("stroke", format!("{}", stroke_color));
        ret = ret.set("stroke-width", format!("{}", stroke_width));
    }
    ret
}

pub fn draw_for_profile(center: &(f64, f64), id: &str, kind: &InputType) -> PointFeatureDrawing {
    let (r, fill) = match kind {
        InputType::City => (5f64, "Black"),
        InputType::Village => (4f64, "Black"),
        InputType::Hamlet => (2f64, "Gray"),
        InputType::MountainPass => (3f64, "Green"),
        InputType::Peak => (3f64, "Green"),
        InputType::GPX => (5f64, "Blue"),
    };

    let mut circle = make_circle(center, &format!("{}", id), fill, &0.0, "");
    circle = circle.set("r", format!("{}", r));

    let mut group = svg::node::element::Group::new();
    group = group.add(circle);

    if *kind == InputType::City || *kind == InputType::Village || *kind == InputType::Hamlet {
        let mut white = make_circle(center, &format!("{}-little-white", id), "white", &0.0, "");
        white = white.set("r", format!("{}", (r - 1.5).max(0.0)));
        group = group.add(white);

        if *kind == InputType::City {
            let mut black = make_circle(center, &format!("{}-little-white", id), "black", &0.0, "");
            black = black.set("r", format!("{}", (r - 2.5).max(0.0)));
            group = group.add(black);
        }
    }

    PointFeatureDrawing {
        group,
        cx: center.0,
        cy: center.1,
    }
}

pub fn draw_for_map((cx, cy): &(f64, f64), id: &str, kind: &InputType) -> PointFeatureDrawing {
    draw_for_profile(&(*cx, *cy), id, kind)
}
