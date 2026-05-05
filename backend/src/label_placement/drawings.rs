use crate::{
    inputpoint::InputPoint,
    label_placement::{
        features::{Label, PointFeatureDrawing},
        FONTSIZE,
    },
    math::Point2D,
    point_collection::Kind,
};

pub struct LabelFormat {
    pub fontsize: f64,
    pub fontweight: String,
    pub fontstyle: String,
}

pub fn format_for_kind(kind: &Kind) -> LabelFormat {
    let base_font_size = FONTSIZE;
    let normal = "normal";
    let light = "lighter";
    let bold = "bold";
    let italic = "italic";
    let (fontsize, fontweight, fontstyle) = match kind {
        Kind::Villages => (base_font_size - 1f64, normal, normal),
        Kind::Hamlets => (base_font_size - 2f64, light, normal),
        Kind::Mountains => (base_font_size - 1f64, normal, italic),
        Kind::Cities => (base_font_size, bold, normal),
        Kind::Controls => (base_font_size, bold, normal),
        _ => (base_font_size, normal, normal),
    };
    LabelFormat {
        fontsize,
        fontweight: fontweight.to_string(),
        fontstyle: fontstyle.to_string(),
    }
}

pub fn make_label_text(w: &InputPoint) -> Label {
    let text = match w.kind() {
        Kind::Villages | Kind::Cities | Kind::Mountains | Kind::Hamlets => {
            w.name().clone().trim().to_string()
        }
        //Kind::GPXWaypoints => format!("{}", "ⓘ"),
        Kind::GPXWaypoints => w.name().clone().trim().to_string(),
        Kind::CutOff => String::new(),
        Kind::Controls => w.name(),
    };

    let format = format_for_kind(&w.kind());
    if text.is_empty() {
        return Label::empty();
    }
    Label::new(
        &text,
        format.fontsize,
        &format.fontweight,
        &format.fontstyle,
    )
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
        Kind::GPXWaypoints => (4f64, "Blue"),
        Kind::CutOff => (2f64, "Gray"),
        Kind::Controls => (5f64, "Blue"),
    };

    let mut circle = make_circle(center, &format!("{}", id), fill, &0.0, "");
    circle = circle.set("r", format!("{}", r));
    if w.kind() == Kind::CutOff {
        circle = circle.set("stroke", format!("{}", "black"));
        circle = circle.set("stroke-width", format!("{}", "2"));
    }

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
