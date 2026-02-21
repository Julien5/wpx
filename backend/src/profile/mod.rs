#![allow(non_snake_case)]
mod elements;
mod ticks;

use svg::Node;

use crate::bbox::BoundingBox;
use crate::gpsdata::ProfileBoundingBox;
use crate::inputpoint::InputPoint;
use crate::label_placement::drawings::draw_for_profile;
use crate::label_placement::features::*;
use crate::label_placement::labelboundingbox::LabelBoundingBox;
use crate::label_placement::obstacle::Obstacles;
use crate::label_placement::*;
use crate::math::{IntegerSize2D, Point2D};
use crate::parameters::{Parameters, ProfileIndication};
use crate::point_collection::{is_osm, Kind, Packets, RenderResult};
use crate::segment;
use crate::track::Track;
use crate::wheel::model::TimeParameters;
use crate::{gpsdata, speed};
use crate::{label_placement, wheel};
use elements::*;

pub struct ProfileModel {
    pub polylines: Vec<Polyline>,
    pub points: Vec<PointFeature>,
}

impl ProfileModel {
    pub fn input_points(&self) -> Vec<InputPoint> {
        self.points
            .iter()
            .filter(|feature| feature.input_point.is_some())
            .map(|feature| feature.input_point.as_ref().unwrap().clone())
            .collect()
    }
}

pub struct ProfileView {
    W: f64,
    H: f64,
    Mleft: f64,
    Mbottom: f64,
    parameters: Parameters,
    BG: Group, // bottom
    SL: Group, // left, with the y axis, the ticks and the labels
    SB: Group, // main group, with the diagram
    pub SD: Group,
    pub bboxdata: gpsdata::ProfileBoundingBox,
    frame_stroke_width: f64,
    model: Option<ProfileModel>,
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
    fn indications(&self) -> Vec<ProfileIndication> {
        self.parameters
            .profile_options
            .elevation_indicators
            .iter()
            .map(|x| x.clone())
            .collect()
    }

    fn yticks_end(&self) -> f64 {
        self.HD()
    }

    fn header_height(&self) -> f64 {
        ProfileGenerator::header_height()
    }

    fn indicator_height(indicator: &ProfileIndication) -> f64 {
        match indicator {
            ProfileIndication::None => 0.0,
            ProfileIndication::NumericSlope => 15.0,
        }
    }

    fn bottom_height(&self) -> f64 {
        let indicators = &self.parameters.profile_options.elevation_indicators;
        Self::indicators_height(indicators) + indicators.len() as f64 * self.frame_stroke_width
    }

    fn indicators_height(indicators: &Vec<ProfileIndication>) -> f64 {
        if indicators.is_empty() {
            return 0.0;
        }
        let mut ret = 0.0;
        for indicator in indicators {
            ret += Self::indicator_height(indicator);
        }
        ret
    }

    pub fn bboxview(&self) -> ProfileBoundingBox {
        fix_margins(&self.bboxdata, self.free_height())
    }
    pub fn init(
        bbox: &gpsdata::ProfileBoundingBox,
        size: &IntegerSize2D,
        parameters: &Parameters,
    ) -> ProfileView {
        let W = size.width as f64;
        let H = size.height as f64;
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

    fn add_time_ticks(
        &mut self,
        pacing_points: &Vec<InputPoint>,
    ) -> (Vec<PointFeature>, Vec<BoundingBox>) {
        let xstart = self.bboxview().get_xmin();
        let start = speed::time_at_distance(xstart, &self.parameters);
        let speed = self.parameters.speed;
        let total_distance = self.bboxview().width();
        let time_parameters = TimeParameters {
            start,
            speed,
            total_distance,
        };
        let times = wheel::time_points::generate_times(&time_parameters);
        let bottom = ProfileGenerator::header_bottom();
        let mut features = Vec::new();
        for (k, time) in times.iter().enumerate() {
            let duration = *time - start;
            let x = xstart + duration.as_seconds_f64() * speed;
            let xd = self.toSD(&Point2D::new(x, 0f64)).x;
            if xd > self.WD() {
                break;
            }
            let mut time_str = wheel::time_points::format_time(&time, false);
            // if it is "9", print "9h", otherwise ("09:30", "Fri") dont change.
            if time_str.trim().parse::<i32>().is_ok() {
                time_str = format!("{}h", time_str);
            }
            let label = Label::new(&time_str, FONTSIZE, &"normal", &"normal");
            let feature = PointFeature {
                circle: PointFeatureDrawing {
                    group: svg::node::element::Group::new(),
                    center: Point2D::new(xd - 10.0, bottom - self.frame_stroke_width),
                },
                label,
                input_point: None,
                link: None,
                xmlid: k,
            };
            features.push(feature);
        }

        let mut bboxes = Vec::new();

        let ceil = ProfileGenerator::header_ceil();
        for point in pacing_points {
            assert!(point.track_projections.len() == 1);
            let x = point
                .track_projections
                .first()
                .unwrap()
                .distance_on_track_to_projection;
            let xd = self.toSD(&Point2D::new(x, 0f64)).x;
            if xd > self.WD() {
                break;
            }
            // give the stroke some more width to avoid rendering
            // right at the edge of control point labels
            bboxes.push(BoundingBox::minmax(
                Point2D::new(xd - 2f64, ceil + 4f64),
                Point2D::new(xd + 2f64, bottom - 4f64),
            ));
        }

        (features, bboxes)
    }

    fn add_numeric_slope(
        &mut self,
        bottom: f64,
        track: &Track,
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
                start: track.index_after(x0),
                end: track.index_before(x1),
            };
            if range.start >= range.end {
                break;
            }
            assert!(range.start <= track.len());
            assert!(range.end < track.len());
            let elevation_gain = track.elevation_gain_on_range(&range);
            let slope_percent = 100.0 * elevation_gain / (x1 - x0);
            let mut text = elements::text(
                format!("{:.1}%", slope_percent).as_str(),
                Point2D::new(xg - 10.0, bottom - self.frame_stroke_width),
                "end",
            );
            text = text.set("font-size", (self.font_size() * 0.8).floor());
            self.SD.append(text);
        }

        let ceil = bottom - Self::indicator_height(&ProfileIndication::NumericSlope);

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

    pub fn render(&self) -> RenderResult {
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
                Some(model) => model.input_points(),
                None => Vec::new(),
            },
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
            if yd > self.free_height() {
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
            if yd > self.free_height() {
                continue;
            }
            self.SD
                .append(stroke("1", Point2D::new(0f64, yd), Point2D::new(WD, yd)));
        }

        for ytick in &_yticks_dashed {
            let yd = self
                .toSD(&Point2D::new(self.bboxview().get_xmin(), *ytick))
                .y;
            if yd > self.free_height() {
                continue;
            }
            self.SD
                .append(dashed(Point2D::new(0f64, yd), Point2D::new(WD, yd)));
        }

        self.SD.append(stroke(
            &format!("{}", self.frame_stroke_width),
            Point2D::new(0f64, ProfileGenerator::header_bottom()),
            Point2D::new(self.WD(), ProfileGenerator::header_bottom()),
        ));
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

    pub fn add_segment(&mut self, packets: &Packets, track: &Track) {
        let bbox = &self.bboxview();

        /*if render_device != RenderDevice::PDF {
                bbox.min.0 = bbox.min.0.max(0f64);
        }*/

        let mut polyline_points = PolylinePoints::new();
        // make sure to cover the whole bbounding box.
        // => start before, end after
        let range = std::ops::Range {
            start: track.index_before(bbox.get_xmin()),
            end: track.index_after(bbox.get_xmax()),
        };
        for k in &track.simplified.dz {
            if range.contains(&k) {
                //let e = track.wgs84[k].z();
                let e = track.smooth_elevation[*k];
                let p = self.toSD(&Point2D::new(track.distance(*k), e));
                polyline_points.push(PolylinePoint(p));
            }
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

        let mut bottom = self.HD() - self.frame_stroke_width;
        for indication in self.indications() {
            if indication == ProfileIndication::NumericSlope {
                bottom = self.add_numeric_slope(bottom, track, &range);
            }
        }

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
        let mut feature_packets = Vec::new();
        let mut feature_unlabeled = Vec::new();
        let mut counter = 0;
        for packet in packets {
            let mut feature_packet = Vec::new();
            for w in packet {
                if w.kind() == Kind::UserStep {
                    continue;
                }
                for proj in &w.track_projections {
                    let index = proj.track_index;
                    if !range.contains(&index) {
                        continue;
                    }
                    let trackpoint = &track.wgs84[index];
                    // Note: It would be better to use the middle point with the float
                    // track_index from track_projection.
                    let p = Point2D::new(track.distance(index), trackpoint.z());
                    let g = self.toSD(&p);
                    let k = counter;
                    counter += 1;
                    let id = format!("{}/wp", k);
                    let circle = draw_for_profile(&g, id.as_str(), &w);

                    //assert!(label.unplaced());
                    let mut label = drawings::make_label_text(&w);
                    label.id = format!("{}/wp/text", k);
                    let empty = label.is_empty();
                    let feature = PointFeature {
                        circle,
                        label,
                        input_point: Some(w.clone()),
                        link: None,
                        xmlid: k,
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

        let mut pacing_points = Vec::new();

        for packet in packets {
            if packet.is_empty() {
                continue;
            }
            if packet.first().unwrap().kind() == Kind::UserStep {
                pacing_points = packet.clone();
            }
        }

        let (time_packet, time_boxes) = self.add_time_ticks(&pacing_points);
        feature_packets.push(PointFeatures::make(time_packet));

        let (results, obstacles) = label_placement::place_labels(
            &feature_packets,
            &*generator,
            &BoundingBox::minmax(
                Point2D::new(0f64, 0f64),
                Point2D::new(self.WD(), self.free_height()),
            ),
            &polyline,
            &self.parameters.profile_options.max_area_ratio,
        );

        for time_box in time_boxes {
            let center = time_box.center() + Point2D::new(0f64, 17f64);
            let circle = {
                let mut ret = svg::node::element::Circle::new();
                ret = ret.set("id", format!("{}", "pacing-circle"));
                ret = ret.set("cx", format!("{}", center.x));
                ret = ret.set("cy", format!("{}", center.y));
                ret = ret.set("r", format!("{}", "2"));
                ret = ret.set("fill", format!("{}", "Gray"));
                ret = ret.set("stroke", format!("{}", "black"));
                ret = ret.set("stroke-width", format!("{}", "2"));
                ret
            };
            self.SD.append(circle);
        }

        let mut features = PlacementResult::apply(&results, &obstacles, &mut feature_packets);
        features.extend_from_slice(&feature_unlabeled);
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
    fn gen(&self, feature: &PointFeature, obstacles: &Obstacles) -> Vec<LabelBoundingBox> {
        if feature.input_point.is_none() {
            // [left mid right] => [mid]
            let mut ret = vec![Self::header(feature)[1].clone()];
            ret.retain(|bbox| !obstacles.hit(feature, &bbox.absolute()));
            return ret;
        }
        let kind = feature.input_point.as_ref().unwrap().kind();
        let mut ret = match kind {
            Kind::UserStep => self.extended_cardinal(feature),
            //OutputType::UserStep => self.generate_column(feature),
            //OutputType::UserStep => self.generate_header(feature, vec![25f64, self.HD - 20f64]),
            Kind::GPXWaypoints | Kind::Controls => Self::header(feature),
            _ => {
                assert!(is_osm(&kind));
                let mut ret = self.cardinal(feature);
                let search_width = 200f64;
                let search_area = BoundingBox::minsize(
                    feature.center() - Point2D::new(search_width * 0.5f64, search_width * 0.5f64),
                    search_width,
                    search_width,
                );
                if obstacles.occupied_area(&search_area) / search_area.area() >= 0.0f64 {
                    let a2 = self.generate_column(feature, 30f64);
                    let a4 = self.generate_column(feature, 55f64);
                    let a8 = self.generate_column(feature, 80f64);
                    ret.extend_from_slice(&a2);
                    ret.extend_from_slice(&a4);
                    ret.extend_from_slice(&a8);
                }
                ret
            }
        };
        ret.retain(|bbox| !obstacles.hit(feature, &bbox.absolute()));
        ret
    }
}

impl ProfileGenerator {
    fn generate_column(&self, feature: &PointFeature, distance: f64) -> Vec<LabelBoundingBox> {
        let target = feature.circle.center;
        let width = feature.width();
        let x = target.x - width / 2f64;
        let mut ret = Vec::new();

        let bbox = BoundingBox::minsize(
            Point2D::new(x, target.y + distance),
            width,
            feature.height(),
        );
        ret.push(LabelBoundingBox::new_absolute(&bbox, &target));

        let bbox = BoundingBox::minsize(
            Point2D::new(x, target.y - distance),
            width,
            feature.height(),
        );
        ret.push(LabelBoundingBox::new_absolute(&bbox, &target));

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

    fn header(feature: &PointFeature) -> Vec<LabelBoundingBox> {
        let target = feature.circle.center;
        let width = feature.width();
        let mut ret = Vec::new();
        for dx in [0.0, -0.5 * width, 0.5 * width] {
            let x = target.x + dx - width / 2f64;
            let bbox = BoundingBox::minsize(
                Point2D::new(x, Self::header_ceil()),
                width,
                feature.height(),
            );
            ret.push(LabelBoundingBox::new_absolute(&bbox, &target));
        }
        ret
    }

    fn cardinal(&self, feature: &PointFeature) -> Vec<LabelBoundingBox> {
        let mut ret = Vec::new();
        assert!(feature.input_point().is_some());

        ret.extend_from_slice(&label_placement::cardinal_boxes(
            &feature.center(),
            feature.width(),
            feature.height(),
        ));
        ret
    }

    fn extended_cardinal(&self, feature: &PointFeature) -> Vec<LabelBoundingBox> {
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
        ret
    }
}

pub fn profile(
    segment: &segment::Segment,
    size: &IntegerSize2D,
    track: &Track,
    packets: &Packets,
    parameters: &Parameters,
) -> RenderResult {
    log::info!(
        "compute profile for size {:?} and {} features",
        size,
        packets.iter().map(|p| p.len()).sum::<usize>()
    );
    let profile_bbox = ProfileBoundingBox::from_track(track, &segment.start, &segment.end);
    let mut view = ProfileView::init(&profile_bbox, size, &parameters);
    view.add_canvas();
    view.add_segment(packets, track);
    view.render_model();
    view.render()
}
