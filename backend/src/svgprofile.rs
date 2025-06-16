#![allow(non_snake_case)]

use crate::gpsdata::ProfileBoundingBox;
use svg::node::element::path::Command;
use svg::node::element::path::Position;
type Data = svg::node::element::path::Data;
type Group = svg::node::element::Group;
type Rect = svg::node::element::Path;
type Circle = svg::node::element::Circle;
type Path = svg::node::element::Path;
type Text = svg::node::element::Text;
use crate::elevation;
use crate::gpsdata;

fn line(p1: (i32, i32), p2: (i32, i32)) -> Data {
    Data::new().move_to(p1).line_to(p2)
}

fn bbox(TL: (i32, i32), BR: (i32, i32)) -> Data {
    Data::new()
        .move_to((TL.0, TL.1))
        .line_to((TL.0, BR.1))
        .line_to((BR.0, BR.1))
        .line_to((BR.0, TL.1))
        .line_to((TL.0, TL.1))
}

fn _testpath() -> Data {
    Data::new().move_to((0, 0)).line_to((20, 20))
}

fn rect(id: &str, color: &str, data: Data) -> Rect {
    Rect::new().set("id", id).set("fill", color).set("d", data)
}

fn bbrect(id: &str, color: &str, TL: (i32, i32), BR: (i32, i32)) -> Rect {
    rect(id, color, bbox(TL, BR))
}

fn transformSL(_W: i32, H: i32, Mleft: i32, Mbottom: i32) -> String {
    format!("translate({} {}) scale(-1 -1)", Mleft, H - Mbottom)
}

fn transformSB(_W: i32, H: i32, Mleft: i32, Mbottom: i32) -> String {
    format!("translate({} {})", Mleft, H - Mbottom)
}

fn transformSD(_W: i32, H: i32, Mleft: i32, Mbottom: i32, _WD: i32) -> String {
    let alpha = 1; //WD as f64 / 100f64;
    format!(
        "translate({} {}) scale(1 -1) scale({} 1)",
        Mleft,
        H - Mbottom,
        alpha
    )
}

fn dashed(from: (i32, i32), to: (i32, i32)) -> Path {
    let p = Path::new()
        .set("stroke", "black")
        .set("stroke-dasharray", "1.0,2.5,5.0,5.0,10.0,5.0")
        .set("d", line(from, to));
    p
}

fn stroke(width: &str, from: (i32, i32), to: (i32, i32)) -> Path {
    let p = Path::new()
        .set("stroke-width", width)
        .set("stroke", "black")
        .set("d", line(from, to));
    p
}

fn textx(label: &str, pos: (i32, i32)) -> Text {
    let ret = Text::new(label)
        .set("font-family", "sans")
        .set("text-anchor", "middle")
        .set("transform", format!("translate({} {})", pos.0, pos.1));
    ret
}

fn texty(label: &str, pos: (i32, i32)) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "end")
        .set("font-family", "sans")
        .set(
            "transform",
            format!("translate({} {}) scale(-1 -1)", pos.0, pos.1),
        );
    ret
}

fn track(d: Data) -> Path {
    let p = Path::new()
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("shape-rendering", "geometricPrecision")
        .set("fill", "transparent")
        .set("d", d);
    p
}

fn toSD((x, y): (f64, f64), WD: i32, HD: i32, bbox: &gpsdata::ProfileBoundingBox) -> (i32, i32) {
    assert!(bbox.xmin <= bbox.xmax);
    assert!(bbox.ymin <= bbox.ymax);
    let f = |x: f64| -> f64 {
        let a = WD as f64 / (bbox.xmax - bbox.xmin);
        let b = -bbox.xmin * a;
        a * x + b
    };
    let g = |y: f64| -> f64 {
        let a = HD as f64 / (bbox.ymax - bbox.ymin);
        let b = -bbox.ymin * a;
        a * y + b
    };
    (f(x).floor() as i32, g(y).floor() as i32)
}

fn data(
    geodata: &gpsdata::Track,
    _range: &std::ops::Range<usize>,
    (WD, HD): (i32, i32),
    bbox: &ProfileBoundingBox,
) -> Data {
    let mut data = Data::new();
    let start = geodata.index_after(bbox.xmin);
    let end = geodata.index_before(bbox.xmax);
    let se = elevation::smooth(geodata);
    for k in start..end {
        //let e = geodata.elevation(k);
        let e = se[k];
        let (x, y) = (geodata.distance(k), e);
        let (xg, yg) = toSD((x, y), WD, HD, bbox);
        if data.is_empty() {
            data.append(Command::Move(Position::Absolute, (xg, yg).into()));
        }
        data.append(Command::Line(Position::Absolute, (xg, yg).into()));
    }
    data
}

pub fn xticks(bbox: &ProfileBoundingBox) -> Vec<f64> {
    let mut ret = Vec::new();
    let _D = bbox.xmax - bbox.xmin;
    let delta = 20000f64;
    let p0 = ((bbox.xmin / delta).ceil() * delta).floor();
    let mut p = p0;
    while p < bbox.xmax.floor() {
        ret.push(p);
        p = p + delta;
    }
    ret
}

pub fn xticks_dashed(bbox: &ProfileBoundingBox) -> Vec<f64> {
    let mut ret = Vec::new();
    let _D = bbox.xmax - bbox.xmin;
    let delta = 20000f64;
    let p0 = ((bbox.xmin / delta).ceil() * delta).floor();
    let mut p = p0;
    while p < bbox.xmax.floor() {
        ret.push(p + delta / 2f64);
        p = p + delta;
    }
    ret
}

pub fn yticks(bbox: &ProfileBoundingBox) -> Vec<f64> {
    let mut ret = Vec::new();
    let _D = bbox.ymax - bbox.ymin;
    let delta = 200f64;
    let p0 = ((bbox.ymin / delta).ceil() * delta).floor();
    let mut p = p0;
    while p < bbox.ymax.floor() {
        ret.push(p);
        p = p + delta;
    }
    ret
}

pub fn yticks_dashed(bbox: &ProfileBoundingBox) -> Vec<f64> {
    let mut ret = Vec::new();
    let _D = bbox.ymax - bbox.ymin;
    let delta = 200f64;
    let p0 = ((bbox.ymin / delta).ceil() * delta).floor();
    let mut p = p0;
    while p < bbox.ymax.floor() {
        ret.push(p + delta / 2f64);
        p = p + delta;
    }
    ret
}

fn dot((x, y): (i32, i32)) -> Circle {
    let dot = svg::node::element::Circle::new()
        .set("cx", x)
        .set("cy", y)
        .set("r", 10);
    dot
}

pub fn canvas(
    geodata: &gpsdata::Track,
    waypoints: Option<&Vec<gpsdata::Waypoint>>,
    range: &std::ops::Range<usize>,
    bbox: &gpsdata::ProfileBoundingBox,
) -> svg::Document {
    let W = 1400;
    let H = 400;
    let Mleft = 50;
    let Mbottom = 50;
    let WD = W - Mleft;
    let HD = H - Mbottom;

    let BG = Group::new().set("id", "frame");

    let mut SL = Group::new()
        .set("id", "SL")
        .set("transform", transformSL(W, H, Mleft, Mbottom));

    let mut SB = Group::new()
        .set("id", "SB")
        .set("transform", transformSB(W, H, Mleft, Mbottom));

    let mut SD = Group::new()
        .set("id", "SD")
        .set("transform", transformSD(W, H, Mleft, Mbottom, WD))
        .add(bbrect("bg", "lightgray", (0, 0), (WD, HD)))
        .add(stroke("3", (0, 0), (WD, 0)))
        .add(stroke("3", (0, 0), (0, HD)))
        .add(stroke("3", (0, HD), (WD, HD)))
        .add(stroke("3", (WD, 0), (WD, HD)));

    for xtick in xticks(bbox) {
        let xd = toSD((xtick, 0f64), WD, HD, bbox).0;
        if xd > WD {
            break;
        }
        if xtick < 0f64 {
            continue;
        }
        SD = SD.add(stroke("1", (xd, 0), (xd, HD)));
        SB = SB.add(textx(
            format!("{}", (xtick / 1000f64).floor() as i32).as_str(),
            (xd, 25),
        ));
    }

    for xtick in xticks_dashed(bbox) {
        let xd = toSD((xtick, 0f64), WD, HD, bbox).0;
        if xd > WD {
            break;
        }
        SD = SD.add(dashed((xd, 0), (xd, HD)));
    }

    for ytick in yticks(bbox) {
        let yd = toSD((bbox.xmin, ytick), WD, HD, bbox).1;
        if yd > HD {
            break;
        }
        SD = SD.add(stroke("1", (0, yd), (WD, yd)));
        SL = SL.add(texty(
            format!("{}", ytick.floor() as i32).as_str(),
            (10, yd - 5),
        ));
    }

    for ytick in yticks_dashed(bbox) {
        let yd = toSD((bbox.xmin, ytick), WD, HD, bbox).1;
        if yd > HD {
            break;
        }
        SD = SD.add(dashed((0, yd), (WD, yd)));
    }

    SD = SD.add(track(data(geodata, range, (WD, HD), bbox)));
    match waypoints {
        Some(W) => {
            for w in W {
                let k = w.track_index;
                if !range.contains(&k) {
                    continue;
                }
                let e = geodata.elevation(k);
                //let e = se[k];
                let (x, y) = toSD((geodata.distance(k), e), WD, HD, bbox);
                SD = SD.add(dot((x, y)));
            }
        }
        _ => {}
    }

    let world = Group::new()
        .set("id", "world")
        .set("shape-rendering", "crispEdges")
        .set("transform", "translate(5 5)")
        .add(BG)
        .add(SL)
        .add(SB)
        .add(SD);

    let document = svg::Document::new()
        .set("width", W + 20)
        .set("height", H)
        .add(world);

    document
}
