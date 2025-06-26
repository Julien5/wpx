#![allow(non_snake_case)]

use crate::backend::WayPoint;
use crate::gpsdata::ProfileBoundingBox;
use svg::node::element::path::Command;
use svg::node::element::path::Position;
use svg::Node;
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
        .set("text-anchor", "middle")
        .set("transform", format!("translate({} {})", pos.0, pos.1));
    ret
}

fn texty(label: &str, pos: (i32, i32)) -> Text {
    let ret = Text::new(label).set("text-anchor", "end").set(
        "transform",
        format!("translate({} {}) scale(-1 -1)", pos.0, pos.1),
    );
    ret
}

fn trackpath(d: Data) -> Path {
    let p = Path::new()
        .set("id", "track")
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

fn waypoint_circle((x, y): (i32, i32), waypoint: &WayPoint) -> Circle {
    match waypoint.origin {
        gpsdata::WaypointOrigin::GPX => svg::node::element::Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("fill", "black")
            .set("r", 8),
        gpsdata::WaypointOrigin::DouglasPeucker => svg::node::element::Circle::new()
            .set("cx", x)
            .set("cy", y)
            .set("fill", "gray")
            .set("stroke", "black")
            .set("stroke-width", "2")
            .set("r", 5),
    }
}

fn waypoint_text((x, y): (i32, i32), waypoint: &WayPoint) -> Text {
    let label = &waypoint.name;
    let ret = Text::new(label).set("text-anchor", "middle").set(
        "transform",
        format!("translate({} {}) scale(1 -1)", x, y - 30),
    );
    ret
}

fn waypoint_elevation_text((x, y): (i32, i32), waypoint: &WayPoint) -> Text {
    let label = format!("{:.0}", waypoint.wgs84.2);
    let ret = Text::new(label)
        .set("text-anchor", "middle")
        .set("font-size", "14")
        .set(
            "transform",
            format!("translate({} {}) scale(1 -1)", x, y + 15),
        );
    ret
}

#[derive(Clone)]
pub struct Profile {
    W: i32,
    H: i32,
    Mleft: i32,
    Mbottom: i32,
    BG: Group,
    SL: Group,
    SB: Group,
    range: std::ops::Range<usize>,
    pub SD: Group,
    bbox: gpsdata::ProfileBoundingBox,
}

impl Profile {
    pub fn init(range: &std::ops::Range<usize>, bbox: &gpsdata::ProfileBoundingBox) -> Profile {
        let W = 1400;
        let H = 400;
        let Mleft = 50;
        let Mbottom = 40;
        Profile {
            W,
            H,
            Mleft,
            Mbottom,
            range: range.clone(),
            bbox: bbox.clone(),
            BG: Group::new().set("id", "BG"),
            SL: Group::new()
                .set("id", "SL")
                .set("transform", transformSL(W, H, Mleft, Mbottom)),
            SB: Group::new()
                .set("id", "SB")
                .set("transform", transformSB(W, H, Mleft, Mbottom)),
            SD: Group::new()
                .set("id", "SD")
                .set("transform", transformSD(W, H, Mleft, Mbottom, W - Mleft)),
        }
    }
    pub fn toSD(&self, (x, y): (f64, f64)) -> (i32, i32) {
        assert!(self.bbox.xmin <= self.bbox.xmax);
        assert!(self.bbox.ymin <= self.bbox.ymax);
        let f = |x: f64| -> f64 {
            let a = self.WD() as f64 / (self.bbox.xmax - self.bbox.xmin);
            let b = -self.bbox.xmin * a;
            a * x + b
        };
        let g = |y: f64| -> f64 {
            let a = self.HD() as f64 / (self.bbox.ymax - self.bbox.ymin);
            let b = -self.bbox.ymin * a;
            a * y + b
        };
        (f(x).floor() as i32, g(y).floor() as i32)
    }
    pub fn WD(&self) -> i32 {
        self.W - self.Mleft
    }
    pub fn HD(&self) -> i32 {
        self.H - self.Mbottom
    }

    pub fn addSD<T>(&mut self, node: T)
    where
        T: Into<Box<dyn svg::Node>>,
    {
        self.SD.append(node);
    }

    pub fn render(&self) -> String {
        let mut world = Group::new()
            .set("id", "world")
            .set("shape-rendering", "crispEdges")
            .set("font-family", "ui-serif")
            .set("font-size", "20")
            .set("transform", "translate(5 5)");
        world.append(self.BG.clone());
        world.append(self.SL.clone());
        world.append(self.SB.clone());
        world.append(self.SD.clone());

        let document = svg::Document::new()
            .set("width", self.W + 20)
            .set("height", self.H)
            .add(world);

        document.to_string()
    }
}

impl Profile {
    pub fn add_canvas(&mut self) {
        let WD = self.WD();
        let HD = self.HD();
        self.SD.append(bbrect("bg", "lightgray", (0, 0), (WD, HD)));
        self.SD.append(stroke("3", (0, 0), (WD, 0)));
        self.SD.append(stroke("3", (0, 0), (0, HD)));
        self.SD.append(stroke("3", (0, HD), (WD, HD)));
        self.SD.append(stroke("3", (WD, 0), (WD, HD)));

        for xtick in xticks(&self.bbox) {
            let xd = toSD((xtick, 0f64), WD, HD, &self.bbox).0;
            if xd > WD {
                break;
            }
            if xtick < 0f64 {
                continue;
            }
            self.SD.append(stroke("1", (xd, 0), (xd, HD)));
            self.SB.append(textx(
                format!("{}", (xtick / 1000f64).floor() as i32).as_str(),
                (xd, 25),
            ));
        }

        for xtick in xticks_dashed(&self.bbox) {
            let xd = toSD((xtick, 0f64), WD, HD, &self.bbox).0;
            if xd > WD {
                break;
            }
            self.SD.append(dashed((xd, 0), (xd, HD)));
        }

        for ytick in yticks(&self.bbox) {
            let yd = toSD((self.bbox.xmin, ytick), WD, HD, &self.bbox).1;
            if yd > HD {
                break;
            }
            self.SD.append(stroke("1", (0, yd), (WD, yd)));
            self.SL.append(texty(
                format!("{}", ytick.floor() as i32).as_str(),
                (10, yd - 5),
            ));
        }

        for ytick in yticks_dashed(&self.bbox) {
            let yd = toSD((self.bbox.xmin, ytick), WD, HD, &self.bbox).1;
            if yd > HD {
                break;
            }
            self.SD.append(dashed((0, yd), (WD, yd)));
        }
    }

    fn add_waypoint(&mut self, w: &WayPoint) {
        let (x, y) = self.toSD((w.distance, w.elevation));
        self.addSD(waypoint_circle((x, y), &w));
        self.addSD(waypoint_elevation_text((x, y), &w));
        if !w.name.is_empty() {
            self.addSD(waypoint_text((x, y), &w));
        }
    }
    pub fn shows_waypoint(&self, w: &WayPoint) -> bool {
        self.bbox.xmin <= w.distance && w.distance <= self.bbox.xmax
    }
    pub fn add_waypoints(&mut self, waypoints: &Vec<WayPoint>) {
        for w in waypoints {
            if !self.shows_waypoint(w) {
                continue;
            }
            self.add_waypoint(w);
        }
    }

    pub fn add_track(&mut self, track: &gpsdata::Track) {
        self.SD.append(trackpath(data(
            track,
            &self.range,
            (self.WD(), self.HD()),
            &self.bbox,
        )))
    }
}
