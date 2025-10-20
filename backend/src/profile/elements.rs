pub type Group = svg::node::element::Group;
pub type Path = svg::node::element::Path;

type Data = svg::node::element::path::Data;
type Text = svg::node::element::Text;

fn line(p1: (f64, f64), p2: (f64, f64)) -> Data {
    Data::new().move_to(p1).line_to(p2)
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

pub fn dashed(from: (f64, f64), to: (f64, f64)) -> Path {
    let p = Path::new()
        .set("stroke", "black")
        .set("stroke-dasharray", "1.0,2.5,5.0,5.0,10.0,5.0")
        .set("d", line(from, to));
    p
}

pub fn stroke(width: &str, from: (f64, f64), to: (f64, f64)) -> Path {
    let p = Path::new()
        .set("stroke-width", width)
        .set("stroke", "black")
        .set("d", line(from, to));
    p
}

pub fn text_middle(label: &str, pos: (f64, f64)) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "middle")
        .set("x", pos.0)
        .set("y", pos.1);
    ret
}

pub fn text_end(label: &str, pos: (f64, f64)) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "end")
        .set("x", pos.0)
        .set("y", pos.1);
    ret
}

pub fn text(label: &str, pos: (f64, f64), anchor: &str) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", anchor)
        .set("x", pos.0)
        .set("y", pos.1);
    ret
}

pub fn texty_overlay(label: &str, pos: (f64, f64)) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "end")
        .set("transform", format!("translate({} {})", pos.0, pos.1))
        .set("font-size", "10");
    ret
}
