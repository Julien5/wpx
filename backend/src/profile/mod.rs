#![allow(non_snake_case)]
mod elements;
mod ticks;

use svg::Node;

use crate::backend::Segment;
use crate::bbox::BoundingBox;
use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::inputpoint::{InputPoint, InputType};
use crate::label_placement;
use crate::label_placement::drawings::draw_for_profile;
use crate::label_placement::features::*;
use crate::label_placement::labelboundingbox::LabelBoundingBox;
use crate::label_placement::*;
use crate::math::{distance2, Point2D};
use crate::parameters::{ProfileIndication, ProfileOptions};
use crate::segment;
use crate::track::Track;
use elements::*;

pub struct ProfileModel {
    pub polylines: Vec<Polyline>,
    pub points: Vec<PointFeature>,
}

impl ProfileModel {
    pub fn input_points(&self) -> Vec<InputPoint> {
        self.points
            .iter()
            .map(|w| w.input_point.as_ref().unwrap().clone())
            .collect()
    }
}

pub struct ProfileView {
    W: f64,
    H: f64,
    Mleft: f64,
    Mbottom: f64,
    options: ProfileOptions,
    BG: Group, // bottom
    SL: Group, // left, with the y axis, the ticks and the labels
    SB: Group, // main group, with the diagram
    pub SD: Group,
    pub bboxdata: gpsdata::ProfileBoundingBox,
    pub bboxview: gpsdata::ProfileBoundingBox,
    frame_stroke_width: f64,
    model: Option<ProfileModel>,
}

fn fix_margins(bbox: &ProfileBoundingBox, options: &ProfileOptions) -> ProfileBoundingBox {
    let H = options.size.1;
    let ticks = ticks::yticks(bbox, H as f64);
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

    fn yticks_end(&self) -> f64 {
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
            bboxview: fix_margins(bbox, options),
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
        match kind {
            ProfileIndication::None => {}
            ProfileIndication::GainTicks => self.add_gain_ticks(track, range),
            ProfileIndication::NumericSlope => self.add_numeric_slope(track, range),
        }
    }

    pub fn render(&self) -> ProfileRenderResult {
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
        ProfileRenderResult {
            svg: document.to_string(),
            rendered: match self.model.as_ref() {
                Some(model) => model.input_points(),
                None => Vec::new(),
            },
        }
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

        let _xticks = ticks::xticks(&self.bboxview, self.W);
        let _xticks_dashed = ticks::xticks_dashed(&self.bboxview, self.W);
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
                Point2D::new(xg, self.yticks_end()),
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
                Point2D::new(xd, self.yticks_end()),
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
        for point in &model.points {
            point.render_in_group(&mut points_group);
        }
        self.SD.append(points_group);
    }

    pub fn add_segment(&mut self, segment: &Segment) {
        let bbox = &self.bboxview;

        /*if render_device != RenderDevice::PDF {
                bbox.min.0 = bbox.min.0.max(0f64);
        }*/
        let track = &segment.track;

        let mut polyline_points = PolylinePoints::new();
        let range = std::ops::Range {
            start: track.index_after(bbox.get_xmin()),
            end: track.index_before(bbox.get_xmax()),
        };
        for k in range.start..range.end {
            //let e = track.wgs84[k].z();
            let e = track.smooth_elevation[k];
            let p = self.toSD(&Point2D::new(track.distance(k), e));
            polyline_points.push(PolylinePoint(p));
        }
        let polyline = Polyline::new(polyline_points);

        /*let mut polyline_dp_points = PolylinePoints::new();
        for k in track.douglas_peucker(10.0, &range) {
            let e = track.wgs84[k].z();
            //let e = track.smooth_elevation[k];
            let p = self.toSD(&Point2D::new(track.distance(k), e));
            polyline_dp_points.push(PolylinePoint(p));
        }
        let polyline_dp = Polyline::new(polyline_dp_points);
        */

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
        let generator = Box::new(ProfileGenerator {
            _WD: self.WD(),
            _HD: self.HD(),
        }); // make features packets
        let packets = label_placement::prioritize::profile(&segment);
        let mut feature_packets = Vec::new();
        let mut counter = 0;
        for packet in packets {
            let mut feature_packet = Vec::new();
            for w in packet {
                let index = w.round_track_index().unwrap();
                let trackpoint = &track.wgs84[index];
                // Note: It would be better to use the middle point with the float
                // track_index from track_projection.
                let p = Point2D::new(track.distance(index), trackpoint.z());
                let g = self.toSD(&p);
                let k = counter;
                counter += 1;
                let id = format!("{}/wp", k);
                let circle = draw_for_profile(&g, id.as_str(), &w);
                let mut label = label_placement::features::Label::new();
                //assert!(label.unplaced());
                label.set_text(&drawings::make_label_text(&w, segment));
                label.id = format!("{}/wp/text", k);
                //assert!(label.unplaced());
                feature_packet.push(PointFeature {
                    circle,
                    label,
                    input_point: Some(w.clone()),
                    link: None,
                    xmlid: k,
                });
            }
            feature_packets.push(PointFeatures::make(feature_packet));
        }

        log::trace!("profile: place labels");
        let (results, obstacles) = label_placement::place_labels(
            &feature_packets,
            &*generator,
            &BoundingBox::minmax(
                Point2D::new(0f64, 0f64),
                Point2D::new(self.WD(), self.HD() - self.eticks_height()),
            ),
            &polyline,
            &self.options.max_area_ratio,
        );
        log::trace!("profile: apply placement");
        let features = PlacementResult::apply(&results, &obstacles, &mut feature_packets);
        self.model = Some(ProfileModel {
            polylines: vec![polyline], // , polyline_dp
            points: features,
        });
    }
}

struct ProfileGenerator {
    pub _WD: f64,
    pub _HD: f64,
}

impl CandidatesGenerator for ProfileGenerator {
    fn gen(&self, feature: &PointFeature) -> Vec<LabelBoundingBox> {
        match feature.input_point.as_ref().unwrap().kind() {
            InputType::OSM => self.cardinal(feature),
            InputType::UserStep => self.extended_cardinal(feature),
            //InputType::UserStep => self.generate_column(feature),
            //InputType::UserStep => self.generate_header(feature, vec![25f64, self.HD - 20f64]),
            InputType::GPX | InputType::Control => self.header(feature, vec![5f64]),
        }
    }
}

impl ProfileGenerator {
    fn _generate_column(&self, feature: &PointFeature) -> Vec<LabelBoundingBox> {
        let target = feature.circle.center;
        let width = feature.width();
        let x = target.x - width / 2f64;
        let little = 5f64;

        let mut ret = Vec::new();
        let mut y = little;
        loop {
            let bbox = BoundingBox::minsize(Point2D::new(x, y), &width, &feature.height());
            if bbox.get_ymax() > self._HD {
                break;
            }
            y += little;
            ret.push(LabelBoundingBox::new_absolute(&bbox, &target));
        }
        ret.sort_by_key(|bbox| {
            let p = bbox.absolute().project_on_border(&target);
            (distance2(&target, &p) * 100f64).floor() as i64
        });
        ret
    }

    fn header(&self, feature: &PointFeature, ys: Vec<f64>) -> Vec<LabelBoundingBox> {
        let target = feature.circle.center;
        let width = feature.width();
        let x = target.x - width / 2f64;
        let mut ret = Vec::new();
        for y in ys {
            let bbox = BoundingBox::minsize(Point2D::new(x, y), &width, &feature.height());
            ret.push(LabelBoundingBox::new_absolute(&bbox, &target));
        }
        ret
    }

    fn cardinal(&self, feature: &PointFeature) -> Vec<LabelBoundingBox> {
        let mut ret = Vec::new();
        assert!(feature.input_point().is_some());

        ret.extend_from_slice(&label_placement::cardinal_boxes(
            &feature.center(),
            &feature.width(),
            &feature.height(),
        ));
        ret
    }

    fn extended_cardinal(&self, feature: &PointFeature) -> Vec<LabelBoundingBox> {
        let mut ret = Vec::new();
        assert!(feature.input_point().is_some());

        ret.extend_from_slice(&label_placement::cardinal_boxes(
            &feature.center(),
            &feature.width(),
            &feature.height(),
        ));

        let width = feature.width();
        let height = feature.height();
        let center = feature.center();
        // 20 px above the target
        let Btop = LabelBoundingBox::new_absolute(
            &BoundingBox::minsize(
                Point2D::new(center.x - width / 2.0, (center.y - 20.0).max(height)),
                &width,
                &height,
            ),
            &center,
        );
        ret.push(Btop);

        // 20 px below the target
        let Bbot = LabelBoundingBox::new_absolute(
            &BoundingBox::minsize(
                Point2D::new(center.x - width / 2.0, (center.y + 20.0).max(height)),
                &width,
                &height,
            ),
            &center,
        );
        ret.push(Bbot);

        // 5 boxes below the top border of the graph
        for n in [1, 3, 5, 7, 9] {
            let Btop2 = LabelBoundingBox::new_absolute(
                &BoundingBox::minsize(
                    Point2D::new(center.x - width / 2.0, (n as f64) * height),
                    &width,
                    &height,
                ),
                &center,
            );
            ret.push(Btop2);
        }

        /*ret.sort_by_key(|candidate| {
            let p = candidate.absolute().project_on_border(&point.center());
            (distance2(&point.center(), &p) * 100f64).floor() as i64
        });*/
        ret
    }
}

pub struct ProfileRenderResult {
    pub svg: String,
    pub rendered: Vec<InputPoint>,
}

pub fn profile(segment: &segment::Segment) -> ProfileRenderResult {
    let profile_bbox = ProfileBoundingBox::from_track(&segment.track, &segment.start, &segment.end);
    let mut view = ProfileView::init(&profile_bbox, &segment.parameters.profile_options);
    view.add_canvas();
    view.add_segment(&segment);
    view.render_model();
    view.render()
}
