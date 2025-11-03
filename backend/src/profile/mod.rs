#![allow(non_snake_case)]
mod elements;
mod ticks;

use std::collections::BTreeSet;

use svg::Node;

use crate::bbox::BoundingBox;
use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::inputpoint::{InputPoint, InputType};
use crate::label_placement;
use crate::label_placement::bbox::LabelBoundingBox;
use crate::label_placement::drawings::draw_for_profile;
use crate::label_placement::*;
use crate::parameters::{ProfileIndication, ProfileOptions};
use crate::segment;
use crate::track::Track;
use elements::*;

struct ProfileModel {
    polylines: Vec<Polyline>,
    points: Vec<PointFeature>,
}

pub struct ProfileView {
    W: f64,
    H: f64,
    Mleft: f64,
    Mbottom: f64,
    options: ProfileOptions,
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
    ret.set_ymin(ticks.first().unwrap().clone());
    ret.set_ymax(ticks.last().unwrap().clone());
    ret
}

// -> (distance,gain as multiple from step_size)
fn elevation_gain_ticks(
    track: &Track,
    step_size: f64,
    range: &std::ops::Range<usize>,
) -> Vec<(f64, f64)> {
    let mut ret = Vec::new();
    for k in range.start + 1..range.end {
        let m0 = track.elevation_gain(k - 1);
        let m1 = track.elevation_gain(k);
        let f0 = m0 / step_size;
        let f1 = m1 / step_size;
        let d = track.distance(k);
        if f0.ceil() != f1.ceil() {
            ret.push((d, f1.floor() * step_size));
        }
    }
    ret
}

impl ProfileView {
    fn profile_indication(&self) -> ProfileIndication {
        let indicators = &self.options.elevation_indicators;
        for indicator in indicators {
            return indicator.clone();
        }
        ProfileIndication::None
    }

    fn xticks_end(&self) -> f64 {
        match self.profile_indication() {
            ProfileIndication::NumericSlope => {
                return self.HD();
            }
            _ => {}
        };
        self.HD() - self.eticks_height()
    }
    fn eticks_height(&self) -> f64 {
        let indicators = &self.options.elevation_indicators;
        if indicators.is_empty() {
            return 0.0;
        }
        let mut ret = 0.0;
        for indicator in indicators {
            let space = match indicator {
                ProfileIndication::None => 0.0,
                ProfileIndication::GainTicks => 7.0,
                ProfileIndication::NumericSlope => 15.0,
            };
            ret += space;
        }
        ret
    }
    pub fn init(
        bbox: &gpsdata::ProfileBoundingBox,
        options: &ProfileOptions,
        _W: i32,
        _H: i32,
    ) -> ProfileView {
        let W = _W as f64;
        let H = _H as f64;
        let Mleft = (W * 0.05f64).floor() as f64;
        let Mbottom = (H / 10f64).floor() as f64;

        ProfileView {
            W,
            H,
            Mleft,
            Mbottom,
            options: options.clone(),
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
            let b = -self.bboxview.get_xmin() * a;
            a * x + b
        };
        let g = |y: &f64| -> f64 {
            let a = -self.HD() as f64 / self.bboxview.height();
            let b = -self.bboxview.get_ymax() * a;
            a * y + b
        };
        (f(x), g(y))
    }

    fn toSL(&self, y: &f64) -> f64 {
        let g = |y: &f64| -> f64 {
            let a = self.HD() as f64 / (self.bboxview.get_ymin() - self.bboxview.get_ymax());
            let b = -self.bboxview.get_ymax() * a;
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

    fn add_gain_ticks(&mut self, track: &Track, range: &std::ops::Range<usize>) {
        let step_size = 50f64;
        let eticks = elevation_gain_ticks(track, step_size, range);
        for etick in eticks {
            let x = etick.0;
            let xd = self.toSD(&(x, 0f64)).0;
            if xd > self.WD() {
                break;
            }
            let meter = etick.1.round() as i32;
            let width = if meter == 0 {
                0
            } else if meter % 1000 == 0 {
                6
            } else if meter % 500 == 0 {
                3
            } else if meter > 0 {
                assert!(meter % (step_size as i32) == 0);
                1
            } else {
                0
            };
            if width > 0 {
                let mut s = stroke(
                    format!("{}", width).as_str(),
                    (xd, self.HD()),
                    (xd, self.HD() - self.eticks_height()),
                );
                s = s.set(
                    "id",
                    format!("elevation-gain-{:.1}-{:.1}", etick.0, etick.1),
                );
                self.SD.append(s);
            }
        }
    }

    fn add_numeric_slope(&mut self, track: &Track, _range: &std::ops::Range<usize>) {
        let eticks = ticks::xticks_all(&self.bboxdata, self.W);
        for k in 1..eticks.len() {
            let x0 = eticks[k - 1];
            let x1 = eticks[k];
            let xg = self.toSD(&(x1, 0f64)).0;
            if xg > self.WD() {
                break;
            }
            let range = std::ops::Range {
                start: track.index_after(x0),
                end: track.index_before(x1),
            };
            let elevation_gain = track.elevation_gain_on_range(&range);
            let slope_percent = 100.0 * elevation_gain / (x1 - x0);
            //log::trace!("{} {} {}",elevation_gain,dx,slop);
            let mut text = elements::text(
                format!("{:.1}%", slope_percent).as_str(),
                (xg - 10.0, self.HD() - 4.0),
                "end",
            );
            text = text.set("font-size", (self.font_size() * 0.8).floor());
            self.SD.append(text);
        }
    }

    fn add_profile_indication(
        &mut self,
        track: &Track,
        range: &std::ops::Range<usize>,
        kind: &ProfileIndication,
    ) {
        println!("add profile indication: {:?}", kind);
        match kind {
            ProfileIndication::None => {}
            ProfileIndication::GainTicks => self.add_gain_ticks(track, range),
            ProfileIndication::NumericSlope => self.add_numeric_slope(track, range),
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
            let pos = self.toSD(&(self.bboxview.get_xmin(), ytick));
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

        self.SD.append(stroke(
            "1",
            (0f64, HD - self.eticks_height()),
            (WD, HD - self.eticks_height()),
        ));

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
            self.SD
                .append(stroke("1", (xg, 0f64), (xg, self.xticks_end())));
            self.SB.append(text_middle(
                format!("{}", (xtick / 1000f64).floor() as f64).as_str(),
                (xg, 2f64 + 15f64),
            ));
        }

        for xtick in _xticks_dashed {
            let xd = self.toSD(&(xtick, 0f64)).0;
            if xd > WD {
                break;
            }
            self.SD
                .append(dashed((xd, self.eticks_height()), (xd, self.xticks_end())));
        }

        for ytick in &_yticks {
            let yd = self.toSL(ytick);
            self.SL.append(text_end(
                format!("{}", ytick.floor() as f64).as_str(),
                (self.Mleft - 5f64, yd + 5f64),
            ));
        }

        for ytick in &_yticks {
            let yd = self.toSD(&(self.bboxview.get_xmin(), *ytick)).1;
            self.SD.append(stroke("1", (0f64, yd), (WD, yd)));
        }

        for ytick in &_yticks_dashed {
            let yd = self.toSD(&(self.bboxview.get_xmin(), *ytick)).1;
            self.SD.append(dashed((0f64, yd), (WD, yd)));
        }
    }

    pub fn render_model(&mut self) {
        let model = self.model.as_ref().unwrap();
        for polyline in &model.polylines {
            let mut svgpath = elements::Path::new();
            for (k, v) in polyline.to_attributes().clone() {
                svgpath = svgpath.set(k, v);
            }
            //svgpath = svgpath.set("stroke-width", "1");
            self.SD.append(svgpath);
        }
        let mut points_group = elements::Group::new();
        for point in &model.points {
            point.render_in_group(&mut points_group);
        }
        self.SD.append(points_group);
    }

    pub fn add_track(&mut self, track: &Track, inputpoints: &Vec<InputPoint>) {
        let bbox = &self.bboxview;

        /*if render_device != RenderDevice::PDF {
            bbox.min.0 = bbox.min.0.max(0f64);
        }*/

        let mut polyline = Polyline::new();
        let range = std::ops::Range {
            start: track.index_after(bbox.get_xmin()),
            end: track.index_before(bbox.get_xmax()),
        };
        for k in range.start..range.end {
            //let e = track.wgs84[k].z();
            let e = track.smooth_elevation[k];
            let (x, y) = (track.distance(k), e);
            let (xg, yg) = self.toSD(&(x, y));
            polyline.points.push((xg, yg));
        }

        let mut polyline_dp = Polyline::new();
        for k in track.douglas_peucker(10.0, &range) {
            let e = track.wgs84[k].z();
            //let e = track.smooth_elevation[k];
            let (x, y) = (track.distance(k), e);
            let (xg, yg) = self.toSD(&(x, y));
            polyline_dp.points.push((xg, yg));
        }

        let kind = self.profile_indication();
        self.add_profile_indication(&track, &range, &kind);

        let mut document = Attributes::new();
        set_attr(
            &mut document,
            "viewBox",
            format!("(0, 0, {}, {})", self.WD(), self.HD()).as_str(),
        );
        set_attr(&mut document, "width", format!("{}", self.WD()).as_str());
        set_attr(&mut document, "height", format!("{}", self.HD()).as_str());

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
            let id = format!("wp-{}", n);
            let circle = draw_for_profile(&(xg, yg), id.as_str(), &w.kind());
            let mut label = label_placement::Label::new();
            match inputpoints[k].short_name() {
                Some(name) => {
                    label.set_text(name.clone().trim());
                    label.id = format!("wp-{}/text", k);
                    points.push(PointFeature {
                        id,
                        circle,
                        label,
                        input_point: Some(w.clone()),
                    });
                }
                None => {
                    log::error!("missing name for {:?}", inputpoints[k]);
                }
            }
        }

        let generator = Box::new(ProfileGenerator);
        let force = false;
        let result = label_placement::place_labels_gen(
            &points,
            &*generator,
            &BoundingBox::init((0f64, 0f64), (self.WD(), self.HD() - self.eticks_height())),
            &polyline,
            force,
        );
        let placed_indices = result.apply(&mut points, force);
        let placed_points = placed_indices.iter().map(|k| points[*k].clone()).collect();
        self.model = Some(ProfileModel {
            polylines: vec![polyline], // , polyline_dp
            points: placed_points,
        });
    }
}

struct ProfileGenerator;

fn cardinal_boxes(center: &(f64, f64), width: &f64, height: &f64) -> Vec<LabelBoundingBox> {
    let mut ret = Vec::new();
    let epsilon = 3f64;
    let B1 = LabelBoundingBox::new_blwh((center.0 + epsilon, center.1 - epsilon), *width, *height);
    ret.push(B1);
    let B2 = LabelBoundingBox::new_brwh((center.0 - epsilon, center.1 - epsilon), *width, *height);
    ret.push(B2);
    let B3 = LabelBoundingBox::new_trwh((center.0 - epsilon, center.1 + epsilon), *width, *height);
    ret.push(B3);
    let B4 = LabelBoundingBox::new_tlwh((center.0 + epsilon, center.1 + epsilon), *width, *height);
    ret.push(B4);

    let B5 = LabelBoundingBox::new_blwh(
        (center.0 + epsilon, center.1 + height / 2.0),
        *width,
        *height,
    );
    ret.push(B5);
    let B6 = LabelBoundingBox::new_blwh(
        (center.0 - width / 2.0, center.1 - epsilon),
        *width,
        *height,
    );
    ret.push(B6);
    let B7 = LabelBoundingBox::new_brwh(
        (center.0 - epsilon, center.1 + height / 2.0),
        *width,
        *height,
    );
    ret.push(B7);

    let B8 = LabelBoundingBox::new_tlwh(
        (center.0 - width / 2.0, center.1 + epsilon),
        *width,
        *height,
    );
    ret.push(B8);

    ret
}

impl ProfileGenerator {
    fn generate_osm(&self, point: &PointFeature) -> Vec<LabelBoundingBox> {
        let mut ret = Vec::new();
        let width = point.width();
        let height = point.height();

        let center = point.center();
        ret.extend_from_slice(&cardinal_boxes(&center, &width, &height));

        let Btop = LabelBoundingBox::new_blwh(
            (center.0 - width / 2.0, (center.1 - 20.0).max(height)),
            width,
            height,
        );
        ret.push(Btop);
        for n in [1, 2, 3] {
            let Btop2 = LabelBoundingBox::new_blwh(
                (center.0 - width / 2.0, (n as f64) * height),
                width,
                height,
            );
            ret.push(Btop2);
        }

        let Bbot = LabelBoundingBox::new_blwh(
            (center.0 - width / 2.0, (center.1 + 20.0).max(height)),
            width,
            height,
        );
        ret.push(Bbot);
        ret
    }
    fn generate_user_step(&self, point: &PointFeature) -> Vec<LabelBoundingBox> {
        let mut ret = Vec::new();
        let width = point.width();
        let height = point.height();
        let dtarget_min = 1f64;
        let dtarget_max = 20f64;
        let d0 = 2f64 * dtarget_max;
        let (cx, cy) = point.center();

        let xmin = cx - width;
        let (ymin, ymax) = (cy - d0 - height, cy + d0);
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
}

impl CandidatesGenerator for ProfileGenerator {
    fn generate(&self, point: &PointFeature) -> Vec<LabelBoundingBox> {
        assert!(point.input_point().is_some());
        match point.input_point().unwrap().kind() {
            InputType::GPX => self.generate_user_step(point),
            InputType::OSM { kind: _ } => self.generate_osm(point),
            InputType::UserStep => self.generate_user_step(point),
        }
    }
    fn prioritize(&self, points: &Vec<PointFeature>) -> Vec<BTreeSet<usize>> {
        let mut user1 = BTreeSet::new();
        let mut user2 = BTreeSet::new();
        let mut osm = BTreeSet::new();
        let mut gpx = BTreeSet::new();
        for k in 0..points.len() {
            let w = &points[k];
            let wi = w.input_point().unwrap();
            match wi.kind() {
                InputType::GPX => {
                    gpx.insert(k);
                }
                InputType::OSM { kind: _ } => {
                    osm.insert(k);
                }
                InputType::UserStep => {
                    if wi.name().unwrap_or("".to_string()).ends_with("0") {
                        user1.insert(k);
                    } else {
                        user2.insert(k);
                    }
                }
            }
        }
        vec![gpx, user1, user2, osm]
    }
}

pub fn profile(
    track: &Track,
    inputpoints: &Vec<InputPoint>,
    segment: &segment::Segment,
    options: &ProfileOptions,
    W: i32,
    H: i32,
) -> String {
    let mut view = ProfileView::init(&segment.profile_bbox, options, W, H);
    view.add_canvas();
    view.add_track(&track, inputpoints);
    view.render_model();
    view.render()
}
