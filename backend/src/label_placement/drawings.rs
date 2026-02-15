use crate::{
    inputpoint::InputPoint, label_placement::features::PointFeatureDrawing, math::Point2D,
    parameters::Parameters, point_collection::Kind, segment::SegmentData, speed,
    track_projection::TrackProjection,
};

pub fn timestr(proj: &TrackProjection, parameters: &Parameters) -> String {
    let t = speed::time_at_distance(&proj.distance_on_track_to_projection, &parameters);
    format!("{}", t.format("%H:%M"))
}

pub fn make_label_text(
    w: &InputPoint,
    proj: Option<&TrackProjection>,
    segment: &SegmentData,
) -> String {
    match w.kind() {
        Kind::Villages | Kind::Cities | Kind::Mountains | Kind::Hamlets => {
            return w.name().clone().trim().to_string();
        }
        Kind::GPXWaypoints => {
            return w.name().clone().trim().to_string();
        }
        Kind::UserStep => {
            //return format!("{}", timestr(proj, &segment.parameters));
            return String::new();
        }

        Kind::Controls => {
            let projection = proj.unwrap();
            return format!(
                "{} ({})",
                w.name(),
                timestr(&projection, &segment.parameters)
            );
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
        Kind::Cities => (5f64, "Black"),
        Kind::Villages => (4f64, "Black"),
        Kind::Hamlets => (2f64, "Gray"),
        Kind::Mountains => (3f64, "Green"),
        Kind::GPXWaypoints => (5f64, "Blue"),
        Kind::UserStep => (3f64, "Black"),
        Kind::Controls => (5f64, "Blue"),
    };

    let mut circle = make_circle(center, &format!("{}", id), fill, &0.0, "");
    circle = circle.set("r", format!("{}", r));

    let mut group = svg::node::element::Group::new();
    group = group.add(circle);
    if w.kind() == Kind::Cities || w.kind() == Kind::Villages || w.kind() == Kind::Hamlets {
        let mut white = make_circle(center, &format!("{}-little-white", id), "white", &0.0, "");
        white = white.set("r", format!("{}", (r - 1.5).max(0.0)));
        group = group.add(white);

        if w.kind() == Kind::Cities {
            let mut black = make_circle(center, &format!("{}-little-white", id), "black", &0.0, "");
            black = black.set("r", format!("{}", (r - 2.5).max(0.0)));
            group = group.add(black);
        }
    }
    PointFeatureDrawing {
        group,
        center: center.clone(),
    }
}

pub fn draw_for_map(point: &Point2D, id: &str, w: &InputPoint) -> PointFeatureDrawing {
    draw_for_profile(point, id, w)
}
