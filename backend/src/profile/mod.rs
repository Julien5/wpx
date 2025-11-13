#![allow(non_snake_case)]
mod elements;
mod ticks;

use std::collections::BTreeMap;

use svg::Node;

use crate::bbox::BoundingBox;
use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::inputpoint::InputPoint;
use crate::label_placement;
use crate::label_placement::bbox::LabelBoundingBox;
use crate::label_placement::candidate::*;
use crate::label_placement::drawings::draw_for_profile;
use crate::label_placement::*;
use crate::math::Point2D;
use crate::parameters::{ProfileIndication, ProfileOptions};
use crate::segment;
use crate::track::Track;
use elements::*;

struct ProfileModel {
    polylines: Vec<Polyline>,
    points: Vec<PointFeature>,
    placed_indices: Vec<usize>,
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
) -> Vec<Point2D> {
    let mut ret = Vec::new();
    for k in range.start + 1..range.end {
        let m0 = track.elevation_gain(k - 1);
        let m1 = track.elevation_gain(k);
        let f0 = m0 / step_size;
        let f1 = m1 / step_size;
        let d = track.distance(k);
        if f0.ceil() != f1.ceil() {
            ret.push(Point2D::new(d, f1.floor() * step_size));
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
    pub fn init(bbox: &gpsdata::ProfileBoundingBox, options: &ProfileOptions) -> ProfileView {
        let W = options.size.0 as f64;
        let H = options.size.1 as f64;
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

    fn toSD(&self, p: &Point2D) -> Point2D {
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
        Point2D::new(f(&p.x), g(&p.y))
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
            let x = etick.x;
            let xd = self.toSD(&Point2D::new(x, 0f64)).x;
            if xd > self.WD() {
                break;
            }
            let meter = etick.y.round() as i32;
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
                    Point2D::new(xd, self.HD()),
                    Point2D::new(xd, self.HD() - self.eticks_height()),
                );
                s = s.set(
                    "id",
                    format!("elevation-gain-{:.1}-{:.1}", etick.x, etick.y),
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
            let xg = self.toSD(&Point2D::new(x1, 0f64)).x;
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
                Point2D::new(xg - 10.0, self.HD() - 4.0),
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
            let pos = self.toSD(&Point2D::new(self.bboxview.get_xmin(), ytick));
            let yd = pos.y;
            if yd > self.HD() {
                break;
            }
            self.SL.append(texty_overlay(
                format!("{}", ytick.floor()).as_str(),
                Point2D::new(30f64, yd + 1f64),
            ));
        }
    }

    pub fn add_canvas(&mut self) {
        let WD = self.WD();
        let HD = self.HD();
        let stroke_widths = format!("{}", self.frame_stroke_width);
        let stroke_width = stroke_widths.as_str();
        self.SD.append(stroke(
            stroke_width,
            Point2D::new(0f64, 0f64),
            Point2D::new(WD, 0f64),
        ));
        self.SD.append(stroke(
            stroke_width,
            Point2D::new(0f64, 0f64),
            Point2D::new(0f64, HD),
        ));
        self.SD.append(stroke(
            stroke_width,
            Point2D::new(0f64, HD),
            Point2D::new(WD, HD),
        ));
        self.SD.append(stroke(
            stroke_width,
            Point2D::new(WD, 0f64),
            Point2D::new(WD, HD),
        ));

        self.SD.append(stroke(
            "1",
            Point2D::new(0f64, HD - self.eticks_height()),
            Point2D::new(WD, HD - self.eticks_height()),
        ));

        let _xticks = ticks::xticks(&self.bboxdata, self.W);
        let _xticks_dashed = ticks::xticks_dashed(&self.bboxdata, self.W);
        let _yticks = ticks::yticks(&self.bboxdata, self.H);
        let _yticks_dashed = ticks::yticks_dashed(&self.bboxdata, self.H);

        for xtick in _xticks {
            let xg = self.toSD(&Point2D::new(xtick, 0f64)).x;
            if xg > WD {
                break;
            }
            if xtick < 0f64 {
                continue;
            }
            self.SD.append(stroke(
                "1",
                Point2D::new(xg, 0f64),
                Point2D::new(xg, self.xticks_end()),
            ));
            self.SB.append(text_middle(
                format!("{}", (xtick / 1000f64).floor() as f64).as_str(),
                Point2D::new(xg, 2f64 + 15f64),
            ));
        }

        for xtick in _xticks_dashed {
            let xd = self.toSD(&Point2D::new(xtick, 0f64)).x;
            if xd > WD {
                break;
            }
            self.SD.append(dashed(
                Point2D::new(xd, self.eticks_height()),
                Point2D::new(xd, self.xticks_end()),
            ));
        }

        for ytick in &_yticks {
            let yd = self.toSL(ytick);
            self.SL.append(text_end(
                format!("{}", ytick.floor() as f64).as_str(),
                Point2D::new(self.Mleft - 5f64, yd + 5f64),
            ));
        }

        for ytick in &_yticks {
            let yd = self.toSD(&Point2D::new(self.bboxview.get_xmin(), *ytick)).y;
            self.SD
                .append(stroke("1", Point2D::new(0f64, yd), Point2D::new(WD, yd)));
        }

        for ytick in &_yticks_dashed {
            let yd = self.toSD(&Point2D::new(self.bboxview.get_xmin(), *ytick)).y;
            self.SD
                .append(dashed(Point2D::new(0f64, yd), Point2D::new(WD, yd)));
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
        let placed_points: Vec<PointFeature> = model
            .placed_indices
            .iter()
            .map(|k| model.points[*k].clone())
            .collect();
        for point in &placed_points {
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
            let p = self.toSD(&Point2D::new(track.distance(k), e));
            polyline.points.push(p);
        }

        let mut polyline_dp = Polyline::new();
        for k in track.douglas_peucker(10.0, &range) {
            let e = track.wgs84[k].z();
            //let e = track.smooth_elevation[k];
            let p = self.toSD(&Point2D::new(track.distance(k), e));
            polyline_dp.points.push(p);
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
            let p = Point2D::new(track.distance(index), trackpoint.z());
            let g = self.toSD(&p);
            let n = points.len();
            let id = format!("wp-{}", n);
            let circle = draw_for_profile(&g, id.as_str(), &w.kind());
            let mut label = label_placement::Label::new();
            match inputpoints[k].short_name() {
                Some(name) => {
                    label.set_text(name.clone().trim());
                    label.id = format!("wp-{}/text", k);
                }
                None => {
                    log::info!("missing name for {:?}", inputpoints[k]);
                }
            }
            points.push(PointFeature {
                id,
                circle,
                label,
                input_point: Some(w.clone()),
                link: None,
            });
        }
        assert_eq!(points.len(), inputpoints.len());

        let generator = Box::new(ProfileGenerator);
        log::trace!("profile: place labels");
        let result = label_placement::place_labels(
            &points,
            &*generator,
            &BoundingBox::init(
                Point2D::new(0f64, 0f64),
                Point2D::new(self.WD(), self.HD() - self.eticks_height()),
            ),
            &polyline,
            &self.options.max_area_ratio,
        );
        log::trace!("profile: apply placement");
        let placed_indices = result.apply(&mut points);
        self.model = Some(ProfileModel {
            polylines: vec![polyline], // , polyline_dp
            points: points.clone(),
            placed_indices,
        });
    }
}

struct ProfileGenerator;

impl ProfileGenerator {
    fn generate_one(point: &PointFeature) -> Vec<LabelBoundingBox> {
        assert!(point.input_point().is_some());
        let mut ret =
            label_placement::cardinal_boxes(&point.center(), &point.width(), &point.height());
        let width = point.width();
        let height = point.height();
        let center = point.center();
        let Btop = LabelBoundingBox::new_blwh(
            Point2D::new(center.x - width / 2.0, (center.y - 20.0).max(height)),
            width,
            height,
        );
        ret.push(Btop);
        for n in [1, 3, 5, 7, 9] {
            let Btop2 = LabelBoundingBox::new_blwh(
                Point2D::new(center.x - width / 2.0, (n as f64) * height),
                width,
                height,
            );
            ret.push(Btop2);
        }

        let Bbot = LabelBoundingBox::new_blwh(
            Point2D::new(center.x - width / 2.0, (center.y + 20.0).max(height)),
            width,
            height,
        );
        ret.push(Bbot);
        ret
    }
}

impl CandidatesGenerator for ProfileGenerator {
    fn generate(
        &self,
        points: &Vec<PointFeature>,
        subset: &Vec<usize>,
        obstacles: &Obstacles,
    ) -> BTreeMap<usize, Candidates> {
        label_placement::candidate::utils::generate(Self::generate_one, points, subset, obstacles)
    }

    fn prioritize(&self, points: &Vec<PointFeature>) -> Vec<Vec<usize>> {
        label_placement::prioritize::profile(points)
    }
}

pub struct ProfileRenderResult {
    pub svg: String,
    pub points_indices: Vec<usize>,
}

pub fn profile(segment: &segment::Segment) -> ProfileRenderResult {
    let profile_bbox = ProfileBoundingBox::from_track(&segment.track, &segment.range);
    let mut view = ProfileView::init(&profile_bbox, &segment.parameters.profile_options);
    view.add_canvas();
    view.add_track(&segment.track, &segment.profile_points());
    view.render_model();
    let svg = view.render();
    ProfileRenderResult {
        svg,
        points_indices: view.model.as_ref().unwrap().placed_indices.clone(),
    }
}
