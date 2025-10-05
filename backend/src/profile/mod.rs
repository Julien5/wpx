#![allow(non_snake_case)]
mod elements;
mod ticks;

use ::svg::Node;

use crate::bbox::BoundingBox;
use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::inputpoint::InputPoint;
use crate::inputpoint::InputType;
use crate::label_placement;
use crate::label_placement::bbox::LabelBoundingBox;
use crate::label_placement::*;
use crate::segment;
use crate::track::Track;
use elements::*;

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
    let ticks = ticks::yticks(bbox, H);
    let mut ret = bbox.clone();
    ret._min.1 = ticks.first().unwrap().clone();
    ret._max.1 = ticks.last().unwrap().clone();
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
            let b = -self.bboxview._min.0 * a;
            a * x + b
        };
        let g = |y: &f64| -> f64 {
            let a = -self.HD() as f64 / self.bboxview.height();
            let b = -self.bboxview._max.1 * a;
            a * y + b
        };
        (f(x), g(y))
    }

    fn toSL(&self, y: &f64) -> f64 {
        let g = |y: &f64| -> f64 {
            let a = self.HD() as f64 / (self.bboxview._min.1 - self.bboxview._max.1);
            let b = -self.bboxview._max.1 * a;
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

        let document = ::svg::Document::new()
            .set("width", Woutput)
            .set("height", self.H)
            .add(world);

        document.to_string()
    }

    pub fn add_yaxis_labels_overlay(&mut self) {
        for ytick in ticks::yticks(&self.bboxdata, self.H) {
            let pos = self.toSD(&(self.bboxview._min.0, ytick));
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

        let _xticks = ticks::xticks(&self.bboxdata, self.W);
        let _xticks_dashed = ticks::xticks_dashed(&self.bboxdata, self.W);
        let _yticks = ticks::yticks(&self.bboxdata, self.H);
        let _yticks_dashed = ticks::yticks_dashed(&self.bboxdata, self.H);

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
            let yd = self.toSD(&(self.bboxview._min.0, *ytick)).1;
            self.SD.append(stroke("1", (0f64, yd), (WD, yd)));
        }

        for ytick in &_yticks_dashed {
            let yd = self.toSD(&(self.bboxview._min.0, *ytick)).1;
            self.SD.append(dashed((0f64, yd), (WD, yd)));
        }
    }

    pub fn render_model(&mut self) {
        let model = self.model.as_ref().unwrap();
        let mut svgpath = elements::Path::new();
        for (k, v) in model.polyline.to_attributes().clone() {
            svgpath = svgpath.set(k, v);
        }
        self.SD.append(svgpath);
        let mut points_group = elements::Group::new();
        for point in &model.points {
            point.render_in_group(&mut points_group);
        }
        self.SD.append(points_group);
    }

    pub fn add_track(
        &mut self,
        track: &Track,
        inputpoints: &Vec<InputPoint>,
        W: f64,
        H: f64,
        _debug: bool,
    ) {
        let bbox = &self.bboxview;

        /*if render_device != RenderDevice::PDF {
            bbox.min.0 = bbox.min.0.max(0f64);
        }*/

        let mut polyline = Polyline::new();
        let start = track.index_after(bbox._min.0);
        let end = track.index_before(bbox._max.0);
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
        for k in 0..inputpoints.len() {
            let w = &inputpoints[k];
            let index = w.round_track_index().unwrap();
            let trackpoint = &track.wgs84[index];
            // Note: It would be better to use the middle point with the float
            // track_index from track_projection.
            let x = track.distance(index);
            let y = trackpoint.z();
            let (xg, yg) = self.toSD(&(x, y));
            let n = points.len();
            let mut circle = label_placement::Circle::new();
            let mut label = label_placement::Label::new();
            circle.id = format!("wp-{}/circle", n);
            circle.cx = xg;
            circle.cy = yg;
            let id = format!("wp-{}", n);
            match w.kind() {
                InputType::City => {
                    circle.r = 5f64;
                    circle.fill = Some("Gray".to_string());
                }
                InputType::Village | InputType::Hamlet => {
                    circle.r = 2f64;
                    circle.fill = Some("Gray".to_string());
                }
                InputType::MountainPass => {
                    circle.r = 3f64;
                    circle.fill = Some("Blue".to_string());
                }
                InputType::Peak => {
                    circle.r = 3f64;
                    circle.fill = Some("Red".to_string());
                }
                InputType::GPX => {
                    circle.r = 4f64;
                    circle.fill = Some("Black".to_string());
                }
            }

            match inputpoints[k].short_name() {
                Some(name) => {
                    label.set_text(name.clone().trim());
                    label.id = format!("wp-{}/text", k);
                    points.push(PointFeature::new(
                        id,
                        circle,
                        label,
                        inputpoints[k].label_placement_order,
                    ));
                }
                None => {
                    log::error!("missing name for {:?}", inputpoints[k]);
                }
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
            if result.placed_indices.contains(&k) {
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
    let xmin = cx - width;
    let ymin = cy - d0 - height;
    let ymax = cy + d0;
    let dp = 5f64;
    let countx = (width / dp).ceil() as i32;
    let county = ((ymax - ymin) / dp).ceil() as i32;
    let dx = width / (countx as f64);
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
    track: &Track,
    inputpoints: &Vec<InputPoint>,
    segment: &segment::Segment,
    W: i32,
    H: i32,
    debug: bool,
) -> String {
    let mut view = ProfileView::init(&segment.profile_bbox, W, H);
    view.add_canvas();
    view.add_track(
        &track,
        inputpoints,
        view.WD() as f64,
        view.HD() as f64,
        debug,
    );
    view.render_model();
    view.render()
}
