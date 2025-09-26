#![allow(non_snake_case)]

use std::str::FromStr;

use crate::backend;
use crate::bbox::BoundingBox;
use crate::gpsdata::distance_wgs84;
use crate::gpsdata::ProfileBoundingBox;
use crate::label_placement;
use crate::label_placement::bbox::LabelBoundingBox;
use crate::label_placement::*;
use crate::segment;
use crate::waypoint::WaypointOrigin;
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

fn snap_ceil(x: f64, step: f64) -> f64 {
    (x / step).ceil() * step
}

fn snap_floor(x: f64, step: f64) -> f64 {
    (x / step).floor() * step
}

fn xtick_delta(bbox: &ProfileBoundingBox, W: f64) -> f64 {
    let min = 50f64 * bbox.width() / W;
    for candidate in [1, 2, 10, 20, 50, 100, 250] {
        let ret = 1000f64 * candidate as f64;
        if ret > min {
            return ret;
        }
    }
    100f64
}

fn xticks_all(bbox: &ProfileBoundingBox, W: f64) -> Vec<f64> {
    let mut ret = Vec::new();
    let _D = bbox.width();
    let delta = xtick_delta(bbox, W);
    let mut start = snap_floor(bbox.min.0, delta);
    start = start.max(0f64);
    let stop = snap_ceil(bbox.max.0, delta);
    let mut p = start;
    while p <= stop {
        ret.push(p);
        p = p + delta;
    }
    ret
}

fn xticks_dashed(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = xticks_all(bbox, H);
    let mut k = 0;
    ret.retain(|_x| {
        k += 1;
        (k % 2) == 0
    });
    ret
}

fn xticks(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = xticks_all(bbox, H);
    let mut k = 0;
    ret.retain(|_y| {
        k += 1;
        (k % 2) != 0
    });
    ret
}

/* ** */

fn ytick_delta(height: &f64, H: f64) -> f64 {
    let min = 20f64 * height / H;
    for candidate in [10, 20, 50, 100, 200, 250, 500, 1000] {
        let ret = candidate as f64;
        if ret > min {
            return ret;
        }
    }
    100f64
}

fn yticks_all(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = Vec::new();
    let delta = ytick_delta(&bbox.height().max(750f64), H);
    let mut start = snap_floor(bbox.min.1, delta) - delta;
    start = start.max(0f64);
    let mut stop = snap_ceil(bbox.max.1, delta) + 2f64 * delta;
    while stop - start < 750f64 {
        start -= delta;
        start = start.max(0f64);
        stop += delta;
    }

    let mut p = start;
    while p <= stop {
        ret.push(p);
        p = p + delta;
    }
    ret
}

fn yticks_dashed(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = yticks_all(bbox, H);
    let mut k = 0;
    ret.retain(|_y| {
        k += 1;
        (k % 2) == 0
    });
    ret
}

fn yticks(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = yticks_all(bbox, H);
    let mut k = 0;
    ret.retain(|_y| {
        k += 1;
        (k % 2) != 0
    });
    ret
}

struct ProfileModel {
    polyline: Polyline,
    points: Vec<PointFeature>,
}

pub struct ProfileView {
    W: f64,
    H: f64,
    Mleft: f64,
    Mbottom: f64,
    BG: Group,
    SL: Group,
    SB: Group,
    pub SD: Group,
    pub bboxdata: gpsdata::ProfileBoundingBox,
    pub bboxview: gpsdata::ProfileBoundingBox,
    frame_stroke_width: f64,
    model: Option<ProfileModel>,
}

fn fix_ymargins(bbox: &ProfileBoundingBox, H: f64) -> ProfileBoundingBox {
    let ticks = yticks(bbox, H);
    let mut ret = bbox.clone();
    ret.min.1 = ticks.first().unwrap().clone();
    ret.max.1 = ticks.last().unwrap().clone();
    ret
}

impl ProfileView {
    pub fn init(bbox: &gpsdata::ProfileBoundingBox, _W: i32, _H: i32) -> ProfileView {
        let W = _W as f64;
        let H = _H as f64;
        let Mleft = (W * 0.05f64).floor() as f64;
        let Mbottom = (H / 10f64).floor() as f64;

        ProfileView {
            W,
            H,
            Mleft,
            Mbottom,
            bboxview: fix_ymargins(bbox, H),
            bboxdata: bbox.clone(),
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
            frame_stroke_width: 3f64,
            model: None,
        }
    }

    fn toSD(&self, (x, y): &(f64, f64)) -> (f64, f64) {
        let f = |x: &f64| -> f64 {
            let a = self.WD() as f64 / (self.bboxview.width());
            let b = -self.bboxview.min.0 * a;
            a * x + b
        };
        let g = |y: &f64| -> f64 {
            let a = -self.HD() as f64 / self.bboxview.height();
            let b = -self.bboxview.max.1 * a;
            a * y + b
        };
        (f(x), g(y))
    }

    fn toSL(&self, y: &f64) -> f64 {
        let g = |y: &f64| -> f64 {
            let a = self.HD() as f64 / (self.bboxview.min.1 - self.bboxview.max.1);
            let b = -self.bboxview.max.1 * a;
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
            12f64
        } else {
            18f64
        }
    }

    pub fn render(&self) -> String {
        let font_size = self.font_size();
        let mut world = Group::new()
            .set("id", "world")
            .set("shape-rendering", "crispEdges")
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
        for ytick in yticks(&self.bboxdata, self.H) {
            let pos = self.toSD(&(self.bboxview.min.0, ytick));
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
        let stroke_widths = format!("{}", self.frame_stroke_width);
        let stroke_width = stroke_widths.as_str();
        self.SD
            .append(stroke(stroke_width, (0f64, 0f64), (WD, 0f64)));
        self.SD
            .append(stroke(stroke_width, (0f64, 0f64), (0f64, HD)));
        self.SD.append(stroke(stroke_width, (0f64, HD), (WD, HD)));
        self.SD.append(stroke(stroke_width, (WD, 0f64), (WD, HD)));

        let _xticks = xticks(&self.bboxdata, self.W);
        let _xticks_dashed = xticks_dashed(&self.bboxdata, self.W);
        let _yticks = yticks(&self.bboxdata, self.H);
        let _yticks_dashed = yticks_dashed(&self.bboxdata, self.H);

        log::debug!(" x={:?}", _xticks);
        log::debug!("xd={:?}", _xticks_dashed);

        for xtick in _xticks {
            let xg = self.toSD(&(xtick, 0f64)).0;
            if xg > WD {
                break;
            }
            if xtick < 0f64 {
                continue;
            }
            self.SD.append(stroke("1", (xg, 0f64), (xg, HD)));
            self.SB.append(textx(
                format!("{}", (xtick / 1000f64).floor() as f64).as_str(),
                (xg, 2f64 + 15f64),
            ));
        }

        for xtick in _xticks_dashed {
            let xd = self.toSD(&(xtick, 0f64)).0;
            if xd > WD {
                break;
            }
            self.SD.append(dashed((xd, 0f64), (xd, HD)));
        }

        for ytick in &_yticks {
            let yd = self.toSL(ytick);
            self.SL.append(ytick_text(
                format!("{}", ytick.floor() as f64).as_str(),
                (self.Mleft - 5f64, yd + 5f64),
            ));
        }

        for ytick in &_yticks {
            let yd = self.toSD(&(self.bboxview.min.0, *ytick)).1;
            self.SD.append(stroke("1", (0f64, yd), (WD, yd)));
        }

        for ytick in &_yticks_dashed {
            let yd = self.toSD(&(self.bboxview.min.0, *ytick)).1;
            self.SD.append(dashed((0f64, yd), (WD, yd)));
        }
    }

    pub fn render_model(&mut self) {
        let model = self.model.as_ref().unwrap();
        let mut svgpath = svg::node::element::Path::new();
        for (k, v) in model.polyline.to_attributes().clone() {
            svgpath = svgpath.set(k, v);
        }
        self.SD.append(svgpath);
        let mut points_group = svg::node::element::Group::new();
        for point in &model.points {
            point.render_in_group(&mut points_group);
        }
        self.SD.append(points_group);
    }

    pub fn add_track(
        &mut self,
        backend: &backend::BackendData,
        segment: &segment::Segment,
        W: f64,
        H: f64,
        _debug: bool,
    ) {
        let waypoints = backend.get_waypoints();
        let bbox = &self.bboxview;

        /*if render_device != RenderDevice::PDF {
            bbox.min.0 = bbox.min.0.max(0f64);
        }*/

        let track = &backend.track;
        let _range = &segment.range;
        let mut polyline = Polyline::new();
        let start = track.index_after(bbox.min.0);
        let end = track.index_before(bbox.max.0);
        for k in start..end {
            let e = track.wgs84[k].z();
            let (x, y) = (track.distance(k), e);
            let (xg, yg) = self.toSD(&(x, y));
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

        let mut points = Vec::new();
        for k in 0..waypoints.len() {
            let w = &waypoints[k];
            let info = w.info.as_ref().unwrap();
            let _index = w.track_index.unwrap();
            let x = info.distance;
            let y = info.elevation;
            if !bbox.contains(&(x, y)) {
                continue;
            }
            if w.origin != WaypointOrigin::GPX {
                continue;
            }
            /*if !range.contains(&index) {
                continue;
            }*/
            let (xg, yg) = self.toSD(&(x, y));
            let mut circle = label_placement::Circle::new();
            circle.id = format!("wp-{}/circle", k);
            circle.cx = xg;
            circle.cy = yg;
            let id = format!("wp-{}", k);
            let mut label = label_placement::Label::new();
            if segment.shows_waypoint(&w) {
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
                let delta = distance_wgs84(&w.wgs84, &trackpoint);
                let y = trackpoint.z();
                let maxdelta = 500f64;
                if delta > maxdelta {
                    continue;
                }
                if !bbox.contains(&(x, y)) {
                    continue;
                }
                if w.name.is_none() {
                    continue;
                }
                /*if !range.contains(&w.track_index.unwrap()) {
                    continue;
                }*/
                let (xg, yg) = self.toSD(&(x, y));
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
        let result = label_placement::place_labels_gen(
            &mut points,
            generate_candidates_bboxes,
            &BoundingBox::init((0f64, 0f64), (W as f64, H as f64)),
            &polyline,
        );
        let mut placed_points = Vec::new();
        for k in 0..points.len() {
            if !result.failed_indices.contains(&k) {
                placed_points.push(points[k].clone());
            }
        }
        self.model = Some(ProfileModel {
            polyline,
            points: placed_points,
        });
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
    let countx = ((xmax - xmin) / dp).ceil() as i32;
    let county = ((ymax - ymin) / dp).ceil() as i32;
    let dx = dp;
    let dy = dp;
    for nx in 0..countx {
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
    }
    ret
}

pub fn profile(
    backend: &backend::BackendData,
    segment: &segment::Segment,
    W: i32,
    H: i32,
    debug: bool,
) -> String {
    let mut view = ProfileView::init(&segment.bbox, W, H);
    view.add_canvas();
    view.add_track(
        &backend,
        &segment,
        view.WD() as f64,
        view.HD() as f64,
        debug,
    );
    view.render_model();
    view.render()
}
