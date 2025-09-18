#![allow(non_snake_case)]

use std::str::FromStr;

use crate::backend;
use crate::elevation;
use crate::gpsdata::distance_wgs84;
use crate::gpsdata::ProfileBoundingBox;
use crate::label_placement;
use crate::label_placement::bbox::LabelBoundingBox;
use crate::label_placement::set_attr;
use crate::label_placement::Attributes;
use crate::label_placement::PointFeature;
use crate::label_placement::Polyline;
use crate::render_device::RenderDevice;
use crate::segment;
use crate::track;
use crate::waypoint::WaypointOrigin;
use crate::waypoints_table;
use svg::Node;

type Data = svg::node::element::path::Data;
type Group = svg::node::element::Group;
type Rect = svg::node::element::Path;
type Path = svg::node::element::Path;
type Text = svg::node::element::Text;

use crate::gpsdata;

fn line(p1: (f64, f64), p2: (f64, f64)) -> Data {
    Data::new().move_to(p1).line_to(p2)
}

fn bbox(TL: (f64, f64), BR: (f64, f64)) -> Data {
    Data::new()
        .move_to((TL.0, TL.1))
        .line_to((TL.0, BR.1))
        .line_to((BR.0, BR.1))
        .line_to((BR.0, TL.1))
        .line_to((TL.0, TL.1))
}

fn rect(id: &str, color: &str, data: Data) -> Rect {
    Rect::new().set("id", id).set("fill", color).set("d", data)
}

fn bbrect(id: &str, color: &str, TL: (f64, f64), BR: (f64, f64)) -> Rect {
    rect(id, color, bbox(TL, BR))
}

fn transformSL(_W: f64, _H: f64, _Mleft: f64, _Mbottom: f64) -> String {
    format!("translate({} {})", 0, 0)
}

fn transformSB(_W: f64, H: f64, Mleft: f64, Mbottom: f64) -> String {
    format!("translate({} {})", Mleft, H - Mbottom)
}

fn transformSD(_W: f64, _H: f64, Mleft: f64, _Mbottom: f64, _WD: f64) -> String {
    format!("translate({} {})", Mleft, 0)
}

fn dashed(from: (f64, f64), to: (f64, f64)) -> Path {
    let p = Path::new()
        .set("stroke", "black")
        .set("stroke-dasharray", "1.0,2.5,5.0,5.0,10.0,5.0")
        .set("d", line(from, to));
    p
}

fn stroke(width: &str, from: (f64, f64), to: (f64, f64)) -> Path {
    let p = Path::new()
        .set("stroke-width", width)
        .set("stroke", "black")
        .set("d", line(from, to));
    p
}

fn textx(label: &str, pos: (f64, f64)) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "middle")
        .set("x", pos.0)
        .set("y", pos.1);
    ret
}

fn ytick_text(label: &str, pos: (f64, f64)) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "end")
        .set("x", pos.0)
        .set("y", pos.1);
    ret
}

fn texty_overlay(label: &str, pos: (f64, f64)) -> Text {
    let ret = Text::new(label)
        .set("text-anchor", "end")
        .set("transform", format!("translate({} {})", pos.0, pos.1))
        .set("font-size", "10");
    ret
}

fn _toSD((x, y): (f64, f64), WD: f64, HD: f64, bbox: &gpsdata::ProfileBoundingBox) -> (f64, f64) {
    assert!(bbox.xmin <= bbox.xmax);
    assert!(bbox.ymin <= bbox.ymax);
    let f = |x: f64| -> f64 {
        let a = WD as f64 / (bbox.xmax - bbox.xmin);
        let b = -bbox.xmin * a;
        a * x + b
    };
    let g = |y: f64| -> f64 {
        let a = HD as f64 / (bbox.ymin - bbox.ymax);
        let b = -bbox.ymax * a;
        a * y + b
    };
    (f(x), g(y))
}

fn _slope(track: &track::Track, smooth: &Vec<f64>) -> Vec<f64> {
    let mut ret = Vec::new();
    debug_assert!(track.wgs84.len() == smooth.len());
    debug_assert!(!smooth.is_empty());
    for k in 0..(track.len() - 1) {
        let dx = track.distance(k) - track.distance(k - 1);
        let dy = smooth[k] - smooth[k + 1];
        let slope = match dx {
            0f64 => 0f64,
            _ => 100f64 * dy / dx,
        };
        ret.push(slope);
    }
    ret.push(0f64);
    debug_assert!(track.wgs84.len() == ret.len());
    elevation::smooth(track, 1000f64, |index: usize| -> f64 { ret[index] })
}

fn xticks(bbox: &ProfileBoundingBox) -> Vec<f64> {
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

fn xticks_dashed(bbox: &ProfileBoundingBox) -> Vec<f64> {
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

fn yticks(bbox: &ProfileBoundingBox) -> Vec<f64> {
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

fn yticks_dashed(bbox: &ProfileBoundingBox) -> Vec<f64> {
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

#[derive(Clone)]
pub struct ProfileView {
    W: f64,
    H: f64,
    Mleft: f64,
    Mbottom: f64,
    BG: Group,
    SL: Group,
    SB: Group,
    pub SD: Group,
    pub bbox: gpsdata::ProfileBoundingBox,
    render_device: RenderDevice,
    font_size_factor: f64,
    frame_stroke_width: f64,
}

impl ProfileView {
    pub fn set_render_device(&mut self, render_device: RenderDevice) {
        self.render_device = render_device;
        match self.render_device {
            RenderDevice::Native => {
                self.font_size_factor = 0.5f64;
            }
            _ => {
                self.font_size_factor = 0.6f64;
            }
        }
    }
    pub fn init(bbox: &gpsdata::ProfileBoundingBox) -> ProfileView {
        let W = 1400f64;
        let H = 400f64;
        let Mleft = ((W as f64) * 0.05f64).floor() as f64;
        let Mbottom = ((H as f64) / 10f64).floor() as f64;
        ProfileView {
            W,
            H,
            Mleft,
            Mbottom,
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
            render_device: RenderDevice::Unknown,
            font_size_factor: 1f64,
            frame_stroke_width: 3f64,
        }
    }

    pub fn reset_size(&mut self, W: f64, H: f64) {
        // TODO: code duplication with init()
        self.Mleft = ((W as f64) * 0.05f64).floor() as f64;
        self.Mbottom = ((H as f64) / 10f64).floor() as f64;
        self.W = W;
        self.H = H;
        if self.render_device != RenderDevice::PDF {
            // no "margin before 0"
            self.bbox.xmin = self.bbox.xmin.max(0f64);
            self.Mleft = 0f64;
        }
        self.BG = Group::new().set("id", "BG");
        self.SL = Group::new().set("id", "SL").set(
            "transform",
            transformSL(self.W, self.H, self.Mleft, self.Mbottom),
        );
        self.SB = Group::new().set("id", "SB").set(
            "transform",
            transformSB(self.W, self.H, self.Mleft, self.Mbottom),
        );
        self.SD = Group::new().set("id", "SD").set(
            "transform",
            transformSD(self.W, self.H, self.Mleft, self.Mbottom, self.WD()),
        );
    }

    fn toSD(&self, (x, y): (f64, f64)) -> (f64, f64) {
        assert!(self.bbox.xmin <= self.bbox.xmax);
        assert!(self.bbox.ymin <= self.bbox.ymax);
        let f = |x: f64| -> f64 {
            let a = self.WD() as f64 / (self.bbox.xmax - self.bbox.xmin);
            let b = -self.bbox.xmin * a;
            a * x + b
        };
        let g = |y: f64| -> f64 {
            let a = self.HD() as f64 / (self.bbox.ymin - self.bbox.ymax);
            let b = -self.bbox.ymax * a;
            a * y + b
        };
        (f(x), g(y))
    }

    fn toSL(&self, y: f64) -> f64 {
        assert!(self.bbox.xmin <= self.bbox.xmax);
        assert!(self.bbox.ymin <= self.bbox.ymax);
        let g = |y: f64| -> f64 {
            let a = self.HD() as f64 / (self.bbox.ymin - self.bbox.ymax);
            let b = -self.bbox.ymax * a;
            a * y + b
        };
        g(y)
    }

    pub fn WD(&self) -> f64 {
        self.W - self.Mleft - self.frame_stroke_width / 2f64
    }
    pub fn HD(&self) -> f64 {
        self.H - self.Mbottom - self.frame_stroke_width / 2f64
    }

    fn font_size(&self) -> f64 {
        if self.W < 750f64 {
            24f64 * self.font_size_factor
        } else {
            30f64 * self.font_size_factor
        }
    }

    pub fn render(&self) -> String {
        let font_size = self.font_size();
        let mut world = Group::new()
            .set("id", "world")
            .set("shape-rendering", "crispEdges")
            .set("font-family", "Libertinus Serif")
            .set("font-size", format!("{}", font_size));
        world.append(self.BG.clone());
        let mut Woutput = self.W;
        let C = self.SD.get_children();
        if C.is_some() && !C.unwrap().is_empty() {
            world.append(self.SB.clone());
            world.append(self.SD.clone());
            world.append(self.SL.clone());
        } else {
            // case render yaxis overlay
            debug_assert!(self.render_device != RenderDevice::PDF);
            world.append(self.SL.clone());
            Woutput = 50f64;
        }

        let document = svg::Document::new()
            .set("width", Woutput)
            .set("height", self.H)
            .add(world);

        document.to_string()
    }

    pub fn add_yaxis_labels_overlay(&mut self) {
        for ytick in yticks(&self.bbox) {
            let pos = self.toSD((self.bbox.xmin, ytick));
            let yd = pos.1;
            if yd > self.HD() {
                break;
            }
            self.SL.append(texty_overlay(
                format!("{}", ytick.floor()).as_str(),
                (30f64, yd + 1f64),
            ));
        }
    }

    pub fn add_canvas(&mut self) {
        let WD = self.WD();
        let HD = self.HD();
        if self.render_device != RenderDevice::PDF {
            self.SD
                .append(bbrect("bg", "lightgray", (0f64, 0f64), (WD, HD)));
        }
        let stroke_widths = format!("{}", self.frame_stroke_width);
        let stroke_width = stroke_widths.as_str();
        self.SD
            .append(stroke(stroke_width, (0f64, 0f64), (WD, 0f64)));
        self.SD
            .append(stroke(stroke_width, (0f64, 0f64), (0f64, HD)));
        self.SD.append(stroke(stroke_width, (0f64, HD), (WD, HD)));
        self.SD.append(stroke(stroke_width, (WD, 0f64), (WD, HD)));

        for xtick in xticks(&self.bbox) {
            let xg = self.toSD((xtick, 0f64)).0;
            if xg > WD {
                break;
            }
            if xtick < 0f64 {
                continue;
            }
            self.SD.append(stroke("1", (xg, 0f64), (xg, HD)));
            self.SB.append(textx(
                format!("{}", (xtick / 1000f64).floor() as f64).as_str(),
                (xg, 2f64 + 25f64 * self.font_size_factor),
            ));
        }

        for xtick in xticks_dashed(&self.bbox) {
            let xd = self.toSD((xtick, 0f64)).0;
            if xd > WD {
                break;
            }
            self.SD.append(dashed((xd, 0f64), (xd, HD)));
        }

        for ytick in yticks(&self.bbox) {
            let yd = self.toSL(ytick);
            if yd < 0f64 {
                break;
            }
            self.SL.append(ytick_text(
                format!("{}", ytick.floor() as f64).as_str(),
                (self.Mleft, yd + 5f64),
            ));
        }

        for ytick in yticks(&self.bbox) {
            let yd = self.toSD((self.bbox.xmin, ytick)).1;
            if yd > HD {
                break;
            }
            self.SD.append(stroke("1", (0f64, yd), (WD, yd)));
        }

        for ytick in yticks_dashed(&self.bbox) {
            let yd = self.toSD((self.bbox.xmin, ytick)).1;
            if yd > HD {
                break;
            }
            self.SD.append(dashed((0f64, yd), (WD, yd)));
        }
    }
}

fn generate_candidates_bboxes(point: &PointFeature) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let width = point.width();
    let height = point.height();
    let dtarget_min = 1f64;
    let dtarget_max = 20f64;
    let d0 = 2f64 * dtarget_max;
    let (cx, cy) = point.center();
    let xmin = cx;
    let ymin = cy - d0 - height;
    let xmax = cx + d0;
    let ymax = cy + d0;
    let dp = 5f64;
    let _countx = ((xmax - xmin) / dp).ceil() as i32;
    let county = ((ymax - ymin) / dp).ceil() as i32;
    let dx = dp;
    let dy = dp;
    let nx = 0;
    for ny in 0..county {
        let tl = (xmin + nx as f64 * dx, ymin + ny as f64 * dy);
        let bb = LabelBoundingBox::new_blwh(tl, width, height);
        if bb.contains((cx, cy)) {
            continue;
        }
        if bb.distance((cx, cy)) < dtarget_min {
            continue;
        }
        ret.push(bb);
    }
    ret
}

struct ProfileModel {
    polyline: Polyline,
    points: Vec<PointFeature>,
}

impl ProfileModel {
    pub fn make(
        backend: &backend::BackendData,
        segment: &segment::Segment,
        W: f64,
        H: f64,
        render_device: RenderDevice,
        _debug: bool,
    ) -> ProfileModel {
        let waypoints = backend.get_waypoints();
        let mut bbox = segment.bbox.clone();

        if render_device != RenderDevice::PDF {
            bbox.xmin = bbox.xmin.max(0f64);
        }

        let track = &backend.track;
        let range = &segment.range;
        let mut polyline = Polyline::new();
        // todo: path in the bbox, which more than the path in the range.
        let start = track.index_after(bbox.xmin);
        let end = track.index_before(bbox.xmax);
        for k in start..end {
            let e = track.wgs84[k].2;
            let (x, y) = (track.distance(k), e);
            let (xg, yg) = _toSD((x, y), W, H, &bbox);
            polyline.points.push((xg, yg));
        }

        let mut document = Attributes::new();
        set_attr(
            &mut document,
            "viewBox",
            format!("(0, 0, {W}, {H})").as_str(),
        );
        set_attr(&mut document, "width", format!("{W}").as_str());
        set_attr(&mut document, "height", format!("{H}").as_str());

        let V = waypoints_table::show_waypoints_in_table(&waypoints, &segment.bbox);
        let mut points = Vec::new();
        for k in 0..waypoints.len() {
            let w = &waypoints[k];
            let info = w.info.as_ref().unwrap();
            let index = w.track_index.unwrap();
            let x = info.distance;
            let y = info.elevation;
            if !bbox.contains(x, y) {
                continue;
            }
            if w.origin != WaypointOrigin::GPX {
                continue;
            }
            if !range.contains(&index) {
                continue;
            }
            let (xg, yg) = _toSD((x, y), W, H, &bbox);
            let mut circle = label_placement::Circle::new();
            circle.id = format!("wp-{}/circle", k);
            circle.cx = xg;
            circle.cy = yg;
            let id = format!("wp-{}", k);
            let mut label = label_placement::Label::new();
            if V.contains(&k) {
                label.set_text(w.info.as_ref().unwrap().profile_label().trim());
                label.id = format!("wp-{}/text", k);
            } else {
                circle.fill = Some(String::from_str("blue").unwrap());
            }
            points.push(PointFeature::new(id, circle, label));
        }

        for (kind, osmpoints) in &backend.osmwaypoints {
            for k in 0..osmpoints.len() {
                let w = &osmpoints[k];
                let index = w.track_index.unwrap();
                let trackpoint = &track.wgs84[index];
                let x = track.distance(index);
                let y = trackpoint.2;
                let delta = distance_wgs84(w.wgs84.0, w.wgs84.1, trackpoint.0, trackpoint.1);
                let maxdelta = 500f64;
                if delta > maxdelta {
                    continue;
                }
                if !bbox.contains(x, y) {
                    continue;
                }
                if w.name.is_none() {
                    continue;
                }
                if !range.contains(&w.track_index.unwrap()) {
                    continue;
                }
                let (xg, yg) = _toSD((x, y), W, H, &bbox);
                let n = points.len();
                let mut circle = label_placement::Circle::new();
                let mut label = label_placement::Label::new();
                circle.id = format!("wp-{}/circle", n);
                circle.cx = xg;
                circle.cy = yg;
                let id = format!("wp-{}", n);
                use super::osm::osmpoint::OSMType::*;
                match kind {
                    City => {
                        circle.r = 5f64;
                        circle.fill = Some("Gray".to_string());
                    }
                    Village => {
                        circle.r = 3f64;
                        circle.fill = Some("Gray".to_string());
                    }
                    MountainPass => {
                        circle.r = 3f64;
                        circle.fill = Some("Blue".to_string());
                    }
                }
                label.set_text(w.name.clone().unwrap().trim());
                label.id = format!("wp-{}/text", k);
                points.push(PointFeature::new(id, circle, label));
            }
        }
        let _ =
            label_placement::place_labels_gen(&mut points, generate_candidates_bboxes, &polyline);
        ProfileModel { polyline, points }
    }

    pub fn render_in_sd(self, SD: &mut svg::node::element::Group) {
        let mut svgpath = svg::node::element::Path::new();
        for (k, v) in self.polyline.to_attributes() {
            svgpath = svgpath.set(k, v);
        }
        SD.append(svgpath);
        let mut points_group = svg::node::element::Group::new();
        for point in self.points {
            point.render_in_group(&mut points_group);
        }
        SD.append(points_group);
    }
}

pub fn profile(
    backend: &backend::BackendData,
    segment: &segment::Segment,
    W: i32,
    H: i32,
    render_device: RenderDevice,
    debug: bool,
) -> String {
    let mut view = ProfileView::init(&segment.bbox);
    view.set_render_device(render_device.clone());
    view.reset_size(W as f64, H as f64);
    view.add_canvas();
    let model = ProfileModel::make(
        &backend,
        &segment,
        view.WD() as f64,
        view.HD() as f64,
        render_device.clone(),
        debug,
    );
    model.render_in_sd(&mut view.SD);
    view.render()
}
