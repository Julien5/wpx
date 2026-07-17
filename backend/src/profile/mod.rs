#![allow(non_snake_case)]
mod elements;
mod ticks;

use chrono::Timelike;
use svg::Node;

use crate::bbox::BoundingBox;
use crate::format::round_time;
use crate::geometry::profilegeometry::ProfileGeometry;
use crate::gpsdata;
use crate::gpsdata::ProfileBoundingBox;
use crate::inputpoint::InputPoint;
use crate::label_placement::candidate::Candidate;
use crate::label_placement::drawings::draw_for_profile;
use crate::label_placement::features::*;
use crate::label_placement::labelboundingbox::LabelBoundingBox;
use crate::label_placement::obstacle::Obstacles;
use crate::math::Point2D;
use crate::mercator::DateTime;
use crate::point_collection::{Kind, Packets, RenderInputParameters, RenderResult};
use crate::{label_placement, wheel};
use crate::{label_placement::*, parameters};
use elements::*;

pub struct TimeTicks {
    labels: Vec<PointFeature>,
    ticks: Vec<(DateTime, Point2D)>,
}

pub struct ProfileModel {
    pub polylines: Vec<Polyline>,
    pub points: Vec<PointFeature>,
}

impl ProfileModel {
    pub fn features(&self) -> Vec<PointFeature> {
        self.points.clone()
    }
}

pub struct ProfileView {
    W: f64,
    H: f64,
    Mleft: f64,
    Mbottom: f64,
    parameters: RenderInputParameters,
    BG: Group, // bottom
    SL: Group, // left, with the y axis, the ticks and the labels
    SB: Group, // main group, with the diagram
    pub SD: Group,
    pub bboxdata: gpsdata::ProfileBoundingBox,
    frame_stroke_width: f64,
    model: Option<ProfileModel>,
    debug_graphic_dir: Option<String>,
}

fn fix_margins(bbox: &ProfileBoundingBox, free_height: f64) -> ProfileBoundingBox {
    let ticks = ticks::yticks_full(bbox, free_height);
    let mut ret = bbox.clone();
    ret.set_ymin(ticks.first().unwrap().clone());
    ret.set_ymax(ticks.last().unwrap().clone());
    ret
}

impl ProfileView {
    fn free_height(&self) -> f64 {
        self.HD() - self.bottom_height() - self.header_height()
    }
    fn yticks_end(&self) -> f64 {
        self.HD()
    }

    fn header_height(&self) -> f64 {
        ProfileGenerator::header_height()
    }

    fn bottom_height(&self) -> f64 {
        15.0 + self.frame_stroke_width
    }

    pub fn bboxview(&self) -> ProfileBoundingBox {
        fix_margins(&self.bboxdata, self.free_height())
    }
    pub fn init(
        bbox: &gpsdata::ProfileBoundingBox,
        parameters: &RenderInputParameters,
        debug_graphic_dir: Option<String>,
    ) -> ProfileView {
        let W = parameters.screen_size.width as f64;
        let H = parameters.screen_size.height as f64;
        let Mleft = (W * 0.05f64).floor() as f64;
        let Mbottom = (H / 10f64).floor() as f64;
        ProfileView {
            W,
            H,
            Mleft,
            Mbottom,
            parameters: parameters.clone(),
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
            debug_graphic_dir,
        }
    }

    fn toSD(&self, p: &Point2D) -> Point2D {
        let f = |x: &f64| -> f64 {
            let a = self.WD() as f64 / (self.bboxview().width());
            let b = -self.bboxview().get_xmin() * a;
            a * x + b
        };
        let g = |y: &f64| -> f64 {
            let a = -self.free_height() as f64 / self.bboxview().height();
            let b = -self.bboxview().get_ymax() * a + self.bottom_height();
            a * y + b
        };
        Point2D::new(f(&p.x), g(&p.y))
    }

    fn _toSL(&self, y: &f64) -> f64 {
        let g = |y: &f64| -> f64 {
            let a = self.HD() as f64 / self.bboxview().height();
            let b = -self.bboxview().get_ymax() * a;
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

    fn cutoff_dot(&self, point: &InputPoint) -> Option<Point2D> {
        let bottom = ProfileGenerator::header_bottom();
        assert!(point.track_projections.len() == 1);
        let x = point
            .track_projections
            .first()
            .unwrap()
            .distance_on_track_to_projection;
        let xd = self.toSD(&Point2D::new(x, 0f64)).x;
        if xd > self.WD() {
            return None;
        }
        // give the stroke some more width to avoid rendering
        // right at the edge of control point labels
        Some(Point2D::new(xd, bottom + 8f64))
    }

    fn compute_time_ticks(&mut self) -> TimeTicks {
        let xstart = self.bboxview().get_xmin();
        let xend = self.bboxview().get_xmax();

        let start_time = parameters::parse_time(&self.parameters.parameters.start_time);
        let nkm = (0.02 * self.W).ceil() as usize;
        let times = wheel::time_points::generate_times_uniform_distance(
            &self.parameters.time_parameters,
            xstart,
            xend,
            nkm,
        );
        let bottom = ProfileGenerator::header_bottom();
        let mut time_features = Vec::new();
        let mut ticks = Vec::new();
        for (k, time) in times.iter().enumerate() {
            let duration = *time - start_time;
            let x = self.parameters.time_parameters.distance(&duration);
            let xd = self.toSD(&Point2D::new(x, 0f64)).x;
            if xd > self.WD() {
                break;
            }
            let mut time_str = wheel::time_points::format_time(&time, false);
            // if it is "9", print "9h", otherwise ("09:30", "Fri") dont change.
            if time_str.trim().parse::<i32>().is_ok() {
                time_str = format!("{}h", time_str);
            }
            ticks.push((time.clone(), Point2D::new(xd, bottom)));
            let label = Label::new(&time_str, FONTSIZE, &"normal", &"normal");
            let feature = PointFeature {
                circle: PointFeatureDrawing {
                    group: svg::node::element::Group::new(),
                    center: Point2D::new(xd, bottom - self.frame_stroke_width),
                },
                label,
                input_point: None,
                link: None,
                xmlid: k, // TODO: remove this
                hardness: 0,
            };
            time_features.push(feature);
        }

        TimeTicks {
            labels: time_features,
            ticks,
        }
    }

    fn add_numeric_slope(
        &mut self,
        bottom: f64,
        profile: &ProfileGeometry,
        _range: &std::ops::Range<usize>,
    ) -> f64 {
        let eticks = ticks::xticks_all(&self.bboxdata, self.W);
        for k in 1..eticks.len() {
            let x0 = eticks[k - 1];
            let x1 = eticks[k];
            let xg = self.toSD(&Point2D::new(x1, 0f64)).x;
            if xg > self.WD() {
                break;
            }
            let range = std::ops::Range {
                start: profile.index_after(x0),
                end: profile.index_before(x1),
            };
            if range.start >= range.end {
                break;
            }
            assert!(range.start <= profile.len());
            assert!(range.end < profile.len());
            let elevation_gain = profile.gain_on_range(&range);
            let slope_percent = 100.0 * elevation_gain / (x1 - x0);
            let mut text = elements::text(
                format!("{:.1}%", slope_percent).as_str(),
                Point2D::new(xg - 10.0, bottom - self.frame_stroke_width),
                "end",
            );
            text = text.set("font-size", (self.font_size() * 0.8).floor());
            self.SD.append(text);
        }

        let ceil = bottom - 15.0;

        for xtick in ticks::xticks_dashed(&self.bboxview(), self.W) {
            let xd = self.toSD(&Point2D::new(xtick, 0f64)).x;
            if xd > self.WD() {
                break;
            }
            self.SD
                .append(dashed(Point2D::new(xd, bottom), Point2D::new(xd, ceil)));
        }

        self.SD.append(stroke(
            &format!("{}", self.frame_stroke_width),
            Point2D::new(0f64, ceil),
            Point2D::new(self.WD(), ceil),
        ));
        ceil - self.frame_stroke_width
    }

    pub fn render_document(&self) -> RenderResult {
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
        RenderResult {
            svg: document.to_string(),
            rendered: match self.model.as_ref() {
                Some(model) => model.features(),
                None => Vec::new(),
            },
            parameters: self.parameters.clone(),
        }
    }

    pub fn add_canvas(&mut self) {
        let WD = self.WD();
        let HD = self.HD();
        let stroke_width = format!("{}", self.frame_stroke_width);
        self.SD.append(stroke(
            &stroke_width,
            Point2D::new(0f64, 0f64),
            Point2D::new(WD, 0f64),
        ));
        self.SD.append(stroke(
            &stroke_width,
            Point2D::new(0f64, 0f64),
            Point2D::new(0f64, HD),
        ));
        self.SD.append(stroke(
            &stroke_width,
            Point2D::new(0f64, HD),
            Point2D::new(WD, HD),
        ));
        self.SD.append(stroke(
            &stroke_width,
            Point2D::new(WD, 0f64),
            Point2D::new(WD, HD),
        ));

        self.SD
            .append(stroke("1", Point2D::new(0f64, HD), Point2D::new(WD, HD)));

        let _xticks = ticks::xticks(&self.bboxview(), self.W);
        let _xticks_dashed = ticks::xticks_dashed(&self.bboxview(), self.W);
        let _yticks_full = ticks::yticks_full(&self.bboxdata, self.free_height());
        let _yticks_dashed = ticks::yticks_dashed(&self.bboxdata, self.free_height());

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
                Point2D::new(xg, self.header_height()),
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
                Point2D::new(xd, self.header_height()),
                Point2D::new(xd, self.free_height()),
            ));
        }

        for ytick in &_yticks_full {
            let yd = self
                .toSD(&Point2D::new(self.bboxview().get_xmin(), *ytick))
                .y;
            if *ytick < 0f64 {
                continue;
            }
            self.SL.append(text_end(
                format!("{}", ytick.floor() as f64).as_str(),
                Point2D::new(self.Mleft - 5f64, yd + 5f64),
            ));
        }

        for ytick in &_yticks_full {
            let yd = self
                .toSD(&Point2D::new(self.bboxview().get_xmin(), *ytick))
                .y;
            self.SD
                .append(stroke("1", Point2D::new(0f64, yd), Point2D::new(WD, yd)));
        }

        for ytick in &_yticks_dashed {
            let yd = self
                .toSD(&Point2D::new(self.bboxview().get_xmin(), *ytick))
                .y;
            self.SD
                .append(dashed(Point2D::new(0f64, yd), Point2D::new(WD, yd)));
        }

        self.SD.append(stroke(
            &format!("{}", self.frame_stroke_width),
            Point2D::new(0f64, ProfileGenerator::header_bottom()),
            Point2D::new(self.WD(), ProfileGenerator::header_bottom()),
        ));

        let time_ticks = self.compute_time_ticks();
        let mut points_group = elements::Group::new();
        // something we cannot do well here is hiding the ticks
        // that collide with control labels.
        for tick_stroke in time_ticks.ticks {
            let width = if tick_stroke.0.minute() == 0 {
                5
            } else if tick_stroke.0.minute() == 30 {
                1
            } else if tick_stroke.0.minute() % 15 == 0 {
                1
            } else {
                1
            };
            let length = if tick_stroke.0.minute() == 0 {
                4
            } else if tick_stroke.0.minute() == 30 {
                10
            } else if tick_stroke.0.minute() % 15 == 0 {
                7
            } else {
                2
            };
            points_group.append(stroke(
                &format!("{}", width),
                tick_stroke.1,
                tick_stroke.1 + Point2D::new(0f64, -length as f64),
            ));
        }
        self.SD.append(points_group);
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
        for point in model.points.iter().rev() {
            point.render_in_group(&mut points_group);
        }
        self.SD.append(points_group);
    }

    pub fn add_packets(&mut self, packets: &Packets, profile: &ProfileGeometry) {
        let features = self.place_packets(packets, profile);
        // the userstep-based time line and time points are rendered
        // in the foreground => do not render them here.
        self.model = Some(ProfileModel {
            points: features.points(),
            polylines: features.polylines,
        });
    }

    fn range(&self, profile: &ProfileGeometry) -> std::ops::Range<usize> {
        let bbox = &self.bboxview();
        profile.subrange(bbox.get_xmin(), bbox.get_xmax())
    }

    fn make_polyline(&self, profile: &ProfileGeometry) -> Polyline {
        let range = self.range(profile);
        let mut polyline_points = PolylinePoints::new();
        // make sure to cover the whole bounding box.
        for w in profile.simplified_indices().windows(2) {
            let (k1, k2) = (w[0], w[1]);
            if !range.contains(&k1) && range.contains(&k2) || (range.start == 0 && k1 == 0) {
                let e = profile.elevation(range.start);
                let p = self.toSD(&Point2D::new(profile.distance(range.start), e));
                polyline_points.push(PolylinePoint(p));
            } else if range.contains(&k1) && !range.contains(&k2) {
                let e = profile.elevation(range.end - 1);
                let p = self.toSD(&Point2D::new(profile.distance(range.end - 1), e));
                polyline_points.push(PolylinePoint(p));
            } else if range.contains(&k2) {
                let e = profile.elevation(k2);
                let p = self.toSD(&Point2D::new(profile.distance(k2), e));
                polyline_points.push(PolylinePoint(p));
            }
        }
        Polyline::new(polyline_points)
    }

    fn userstep_dot(box_center: &Point2D, w: &InputPoint) -> PointFeature {
        assert!(w.kind() == Kind::CutOff);
        let center = *box_center;
        let circle = draw_for_profile(&center, &format!("user-step"), w);
        let mut label = drawings::make_label_text(&w);
        label.id = format!("user-step/text");
        let proj = w.track_projections.first().unwrap().clone();
        PointFeature {
            circle,
            label,
            input_point: Some(w.clone_with_proj(&proj)),
            link: None,
            xmlid: w.index().unwrap_or(0),
            hardness: 0,
        }
    }

    fn place_packets(&mut self, packets: &Packets, profile: &ProfileGeometry) -> Features {
        // make sure to cover the whole bounding box.
        // => start before, end after
        let range = self.range(profile);
        let polyline = self.make_polyline(profile);
        let bottom = self.HD() - self.frame_stroke_width;
        self.add_numeric_slope(bottom, profile, &range);

        let generator = Box::new(ProfileGenerator {
            _WD: self.WD(),
            _HD: self.HD(),
        }); // make features packets
        let mut feature_packets = Vec::new();
        let mut feature_unlabeled = Vec::new();
        let mut counter = 0;

        let time_parameters = &self.parameters.time_parameters;

        for packet in packets {
            let mut feature_packet = Vec::new();
            for w in &packet.points {
                if w.kind() == Kind::CutOff {
                    let dot = self.cutoff_dot(&w);
                    if dot.is_none() {
                        continue;
                    }
                    feature_unlabeled.push(Self::userstep_dot(&dot.unwrap(), w));
                    continue;
                }
                for proj in &w.track_projections {
                    let index = proj.track_index;
                    if !range.contains(&index) {
                        continue;
                    }
                    // Note: It would be better to use the middle point with the float
                    // track_index from track_projection.
                    let p = Point2D::new(profile.distance(index), profile.elevation(index));
                    let g = self.toSD(&p);
                    let k = counter;
                    counter += 1;
                    let id = format!("{}/wp", k);
                    let circle = draw_for_profile(&g, id.as_str(), &w);

                    //assert!(label.unplaced());
                    let mut label = drawings::make_label_text(&w);
                    label.id = format!("{}/wp/text", k);
                    if w.kind() == Kind::Controls
                        && !w.data.as_control().unwrap().is_start()
                        && !w.data.as_control().unwrap().is_end()
                    {
                        let time = time_parameters.time(proj.distance_on_track_to_projection);
                        let text = format!("{} ({})", w.name(), round_time(&time).format("%H:%M"));
                        let format = drawings::format_for_kind(&w.kind());
                        label = Label::new(
                            &text,
                            format.fontsize,
                            &format.fontweight,
                            &format.fontstyle,
                        );
                    }
                    let empty = label.is_empty();
                    let feature = PointFeature {
                        circle,
                        label,
                        input_point: Some(w.clone_with_proj(&proj)),
                        link: None,
                        xmlid: k,
                        hardness: packet.hardness,
                    };
                    if empty {
                        feature_unlabeled.push(feature);
                    } else {
                        feature_packet.push(feature);
                    }
                }
            }
            feature_packets.push(PointFeatures::make(feature_packet));
        }

        // The time ticks are incorporated in the background because label placement is involved.
        // The background cache lookup is sensitive to the time parameters.
        // time parameters change => background is re-computed.
        let time_ticks = self.compute_time_ticks();
        feature_packets.push(PointFeatures::make(time_ticks.labels));

        let (results, obstacles) = label_placement::place_labels(
            &feature_packets,
            &*generator,
            &BoundingBox::minmax(
                Point2D::new(0f64, 0f64),
                Point2D::new(self.WD(), self.free_height()),
            ),
            &polyline,
            self.debug_graphic_dir.clone(),
        );

        let features = PlacementResult::apply(&results, &obstacles, &mut feature_packets);
        Features {
            labeled: features,
            unlabeled: feature_unlabeled,
            polylines: vec![polyline],
        }
    }
}

struct ProfileGenerator {
    pub _WD: f64,
    pub _HD: f64,
}

impl CandidatesGenerator for ProfileGenerator {
    fn gen(&self, feature: &PointFeature, obstacles: &Obstacles) -> Vec<Candidate> {
        if feature.input_point.is_none() {
            // [start mid end] => [mid]
            let mut ret = vec![Self::header(feature)[1].clone()];
            debug_assert!(!feature.force_rendering());
            ret.retain(|c| !obstacles.hit(feature, &c.bbox().absolute()));
            return ret;
        }
        let kind = feature.input_point().unwrap().kind();
        let drawing_width = obstacles.drawingbox.bbox.width();
        let mut ret = match kind {
            Kind::CutOff => {
                debug_assert!(false);
                Vec::new()
            }
            Kind::Controls => {
                let [start, middle, end]: [Candidate; 3] =
                    Self::header(feature).try_into().unwrap();
                vec![middle, start, end]
            }
            //Kind::GPXWaypoints => Self::header_offset(feature, 15f64),
            _ => {
                let mut ret = self.cardinal(feature);
                if feature.hardness > 2 {
                    let a = self.generate_column(feature, drawing_width, 30f64);
                    ret.extend_from_slice(&a);
                }
                if feature.hardness > 4 {
                    let a = self.generate_column(feature, drawing_width, 55f64);
                    ret.extend_from_slice(&a);
                }
                if feature.hardness > 6 {
                    let a = self.generate_column(feature, drawing_width, 80f64);
                    ret.extend_from_slice(&a);
                }
                ret
                // ret.iter().map(|lbbox| Candidate::new(lbbox)).collect()
            }
        };
        debug_assert!(!ret.is_empty());
        let last = ret.last().unwrap().clone();
        ret.retain(|c| !obstacles.hit(feature, &c.bbox().absolute()));
        if feature.force_rendering() && ret.is_empty() {
            ret = vec![last];
        }
        ret
    }
}

impl ProfileGenerator {
    fn generate_column(
        &self,
        feature: &PointFeature,
        drawing_width: f64,
        distance: f64,
    ) -> Vec<Candidate> {
        let cx = feature.circle.center.x;
        let width = feature.label.bbox.width();
        let delta = 0.6 * width;
        if cx < delta {
            return self.generate_x_column(feature, cx + 5f64, distance);
        }
        if (cx + delta) > drawing_width {
            return self.generate_x_column(feature, drawing_width - width - delta, distance);
        }
        self.generate_middle_column(feature, distance)
    }

    fn generate_middle_column(&self, feature: &PointFeature, distance: f64) -> Vec<Candidate> {
        let target = feature.circle.center;
        let width = feature.width();
        let x = target.x - width / 2f64;
        self.generate_x_column(feature, x, distance)
    }

    fn generate_x_column(&self, feature: &PointFeature, x: f64, distance: f64) -> Vec<Candidate> {
        let target = feature.circle.center;
        let width = feature.width();
        let mut ret = Vec::new();

        let bbox = BoundingBox::minsize(
            Point2D::new(x, target.y + distance),
            width,
            feature.height(),
        );
        ret.push(Candidate::new(&LabelBoundingBox::new_absolute(
            &bbox, &target,
        )));

        let bbox = BoundingBox::minsize(
            Point2D::new(x, target.y - distance),
            width,
            feature.height(),
        );
        ret.push(Candidate::new(&LabelBoundingBox::new_absolute(
            &bbox, &target,
        )));

        ret
    }

    fn header_height() -> f64 {
        20f64
    }

    fn header_ceil() -> f64 {
        2f64
    }

    fn header_bottom() -> f64 {
        Self::header_height()
    }

    fn header(feature: &PointFeature) -> Vec<Candidate> {
        Self::header_offset(feature, 0f64)
    }

    fn header_offset(feature: &PointFeature, yoffset: f64) -> Vec<Candidate> {
        let target = feature.circle.center;
        let width = feature.width();
        let mut ret = Vec::new();
        for anchor in ["start", "middle", "end"] {
            let dx = if anchor == "start" {
                0.0
            } else if anchor == "middle" {
                -0.5 * width
            } else {
                -width
            };
            let bbox = BoundingBox::minsize(
                Point2D::new(target.x + dx, Self::header_ceil() + yoffset),
                width,
                feature.height(),
            );
            let lbbox = LabelBoundingBox::new_absolute(&bbox, &target).with_text_anchor(anchor);
            ret.push(Candidate::make_external(&lbbox));
        }
        ret
    }

    fn cardinal(&self, feature: &PointFeature) -> Vec<Candidate> {
        let mut ret = Vec::new();
        assert!(feature.input_point().is_some());

        ret.extend_from_slice(&label_placement::cardinal_boxes_profile(
            &feature.center(),
            feature.width(),
            feature.height(),
        ));
        ret.iter().map(|lbbox| Candidate::new(lbbox)).collect()
    }

    #[allow(dead_code)]
    fn extended_cardinal(&self, feature: &PointFeature) -> Vec<Candidate> {
        let mut ret = Vec::new();
        assert!(feature.input_point().is_some());

        ret.extend_from_slice(&label_placement::cardinal_boxes(
            &feature.center(),
            feature.width(),
            feature.height(),
        ));

        let width = feature.width();
        let height = feature.height();
        let center = feature.center();
        // 20 px above the target
        let Btop = LabelBoundingBox::new_absolute(
            &BoundingBox::minsize(
                Point2D::new(center.x - width / 2.0, (center.y - 20.0).max(height)),
                width,
                height,
            ),
            &center,
        );
        ret.push(Btop);

        // 20 px below the target
        let Bbot = LabelBoundingBox::new_absolute(
            &BoundingBox::minsize(
                Point2D::new(center.x - width / 2.0, (center.y + 20.0).max(height)),
                width,
                height,
            ),
            &center,
        );
        ret.push(Bbot);

        // 5 boxes below the top border of the graph
        for n in [1, 3, 5, 7, 9] {
            let Btop2 = LabelBoundingBox::new_absolute(
                &BoundingBox::minsize(
                    Point2D::new(center.x - width / 2.0, (n as f64) * height),
                    width,
                    height,
                ),
                &center,
            );
            ret.push(Btop2);
        }

        /*ret.sort_by_key(|candidate| {
            let p = candidate.absolute().project_on_border(&point.center());
            (distance2(&point.center(), &p) * 100f64).floor() as i64
        });*/
        ret.iter().map(|lbbox| Candidate::new(lbbox)).collect()
    }
}

pub fn render_profile(
    profile: &ProfileGeometry,
    parameters: &RenderInputParameters,
    packets: &Packets,
    debug_dir: Option<String>,
) -> RenderResult {
    log::info!("compute profile foreground for parameters {:?}", parameters);
    let profile_bbox =
        ProfileBoundingBox::from_track(profile, &parameters.drange.start, &parameters.drange.end);
    let mut view = ProfileView::init(&profile_bbox, parameters, debug_dir);
    view.add_canvas();
    view.add_packets(packets, profile);
    view.render_model();
    view.render_document()
}
