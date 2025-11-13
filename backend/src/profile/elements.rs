use crate::math::Point2D;

pub type Group = svg::node::element::Group;
pub type Path = svg::node::element::Path;

type Data = svg::node::element::path::Data;
type Text = svg::node::element::Text;

fn line(p1: Point2D, p2: Point2D) -> Data {
    Data::new().move_to(p1.to_tuple()).line_to(p2.to_tuple())
}

pub fn transformSL(_W: f64, _H: f64, _Mleft: f64, _Mbottom: f64) -> String {
    format!("translate({} {})", 0, 0)
}

pub fn transformSB(_W: f64, H: f64, Mleft: f64, Mbottom: f64) -> String {
    format!("translate({} {})", Mleft, H - Mbottom)
}

pub fn transformSD(_W: f64, _H: f64, Mleft: f64, _Mbottom: f64, _WD: f64) -> String {
    format!("translate({} {})", Mleft, 0)
}

pub fn dashed(from: Point2D, to: Point2D) -> Path {
    let p = Path::new()
        .set("stroke", "black")
        .set("stroke-dasharray", "1.0,2.5,5.0,5.0,10.0,5.0")
        .set("d", line(from, to));
    p
}

pub fn stroke(width: &str, from: Point2D, to: Point2D) -> Path {
    let p = Path::new()
        .set("stroke-width", width)
        .set("stroke", "black")
        .set("d", line(from, to));
    p
}

pub fn text_middle(label: &str, pos: Point2D) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "middle")
        .set("x", pos.x)
        .set("y", pos.y);
    ret
}

pub fn text_end(label: &str, pos: Point2D) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "end")
        .set("x", pos.x)
        .set("y", pos.y);
    ret
}

pub fn text(label: &str, pos: Point2D, anchor: &str) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", anchor)
        .set("x", pos.x)
        .set("y", pos.y);
    ret
}

pub fn texty_overlay(label: &str, pos: Point2D) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "end")
        .set("transform", format!("translate({} {})", pos.x, pos.y))
        .set("font-size", "10");
    ret
}
