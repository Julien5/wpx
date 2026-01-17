pub mod model;
pub mod shorten;
mod time_points;

use svg::node::element::path::Data;
use svg::node::element::Text;
use svg::node::element::{Circle, Group, Path};
use svg::Document;

use crate::math::{IntegerSize2D, Point2D, ScreenSpace};
use crate::wheel::model::CirclePoint;

mod constants {
    pub const ARCANGLE: f64 = 40f64;
}

struct Page {
    pub total_size: IntegerSize2D,
    pub wheel_width: i32,
    pub margin: i32,
}

impl Page {
    fn size(&self) -> IntegerSize2D {
        IntegerSize2D::new(self.total_size.width, self.total_size.height)
    }
    fn wheel_outer_radius(&self) -> i32 {
        let size = self.size();
        size.width.min(size.height) / 2 - self.margin
    }
    fn wheel_inner_radius(&self) -> i32 {
        self.wheel_outer_radius() - self.wheel_width
    }
    fn center(&self) -> euclid::Vector2D<i32, ScreenSpace> {
        (self.size() / 2).to_vector()
    }
    fn make_centered_group(&self) -> Group {
        let center = self.center();
        Group::new().set(
            "transform",
            format!("translate({}, {})", center.x, center.y),
        )
    }
}

enum Region {
    Inner,
    Outer,
}

fn zone(angle: f64) -> usize {
    let mut ret = 1;
    if angle < 22.5 {
        return ret; // 1
    }
    ret += 1;
    if angle < 67.5 {
        return ret; // 2
    }
    ret += 1;
    if angle < 112.5 {
        return ret;
    }
    ret += 1;
    if angle < 157.5 {
        return ret;
    }
    ret += 1;
    if angle < 202.5 {
        return ret;
    }
    ret += 1;
    if angle < 247.5 {
        return ret;
    }
    ret += 1;
    if angle < 292.5 {
        return ret;
    }
    ret += 1;
    if angle < 337.5 {
        return ret; // 8
    }
    return 1;
}

fn anchor(angle: f64, region: Region) -> String {
    match region {
        Region::Inner => match zone(angle) {
            1 | 5 => "middle".into(),
            2 | 3 | 4 => "end".into(),
            _ => "start".into(),
        },
        Region::Outer => match zone(angle) {
            1 | 5 => "middle".into(),
            2 | 3 | 4 => "start".into(),
            _ => "end".into(),
        },
    }
}

fn label_position(angle: f64, radius: f64, text_height: f64, region: Region) -> Point2D {
    let mut ret = Point2D::new(angle.to_radians().sin(), -angle.to_radians().cos()) * radius;
    match region {
        Region::Inner => match zone(angle) {
            1 | 2 | 8 => {
                ret.y += text_height;
            }
            3 | 7 => {
                ret.y += text_height / 2f64;
            }
            4 | 5 | 6 => {}
            _ => {
                assert!(false);
            }
        },
        Region::Outer => match zone(angle) {
            1 | 2 | 8 => {}
            3 | 7 => {
                ret.y += text_height / 2f64;
            }
            4 | 5 | 6 => {
                ret.y += text_height;
            }
            _ => {
                assert!(false);
            }
        },
    }
    ret
}

fn text_height() -> f64 {
    10.0
}

fn add_control_point(
    page: &Page,
    point: &CirclePoint,
    mut ticks_group: Group,
    mut label_group: Group,
    hour_thick: i32,
    hour_tick_data: &Data,
) -> (Group, Group) {
    let angle = point.angle;

    let tick = Path::new()
        .set("d", hour_tick_data.clone())
        .set("stroke", "#333")
        .set("stroke-width", hour_thick);
    let tick_rotated = tick.set("transform", format!("rotate({})", angle));
    ticks_group = ticks_group.add(tick_rotated);

    let label_pos = label_position(
        angle,
        (page.wheel_outer_radius() + 7) as f64,
        text_height(),
        Region::Outer,
    );

    let name = point.name.clone();
    log::trace!("name={}", name);

    let label = Text::new(format!("{}", name))
        .set("text-anchor", anchor(angle, Region::Outer))
        .set("x", label_pos.x)
        .set("y", label_pos.y);
    label_group = label_group.add(label);
    (ticks_group, label_group)
}

fn features(page: &Page, model: &model::WheelModel) -> Group {
    let mut ticks_group = page.make_centered_group();
    let mut label_group = page.make_centered_group();
    let mut ret = Group::new();

    let control_thick = page.wheel_width / 3;
    let control_tick_start = page.wheel_inner_radius();
    let control_tick_end = page.wheel_outer_radius();
    let control_tick_path =
        Data::parse(format!("M 0 -{} L 0 -{}", control_tick_start, control_tick_end).as_str())
            .unwrap();

    for i in 0..model.control_points.len() {
        let point = &model.control_points[i];
        (ticks_group, label_group) = add_control_point(
            page,
            point,
            ticks_group,
            label_group,
            control_thick,
            &control_tick_path,
        );
    }

    for (not_needed, a, name) in [
        (model.has_end_control, -constants::ARCANGLE / 2.0, "End"),
        (model.has_start_control, constants::ARCANGLE / 2.0, "Start"),
    ] {
        if not_needed {
            continue;
        }
        (ticks_group, label_group) = add_control_point(
            page,
            &CirclePoint {
                angle: a,
                name: name.to_string().clone(),
            },
            ticks_group,
            label_group,
            control_thick,
            &control_tick_path,
        );
    }

    let steps_thick = (control_thick / 4).max(1);
    let little_space = (control_thick / 4).max(1);
    let step_tick_start = control_tick_start + little_space;
    let step_tick_stop = control_tick_end - little_space;
    let step_tick_path =
        Data::parse(format!("M 0 -{} L 0 -{}", step_tick_start, step_tick_stop).as_str()).unwrap();

    for i in 0..model.mid_points.len() {
        let point = &model.mid_points[i];
        let angle = point.angle;
        let tick = Path::new()
            .set("d", step_tick_path.clone())
            .set("stroke", "#666")
            .set("stroke-width", steps_thick);
        let tick_rotated = tick.set("transform", format!("rotate({})", angle));
        ticks_group = ticks_group.add(tick_rotated);
    }

    let time_tick_path = Data::parse(
        format!(
            "M 0 -{} L 0 -{}",
            page.wheel_inner_radius() - 5,
            page.wheel_inner_radius()
        )
        .as_str(),
    )
    .unwrap();
    let label_position_radius = page.wheel_inner_radius() - 7;
    for i in 0..model.time_points.len() {
        let point = &model.time_points[i];
        let angle = point.angle;
        let tick = Path::new()
            .set("d", time_tick_path.clone())
            .set("stroke", "#666")
            .set("stroke-width", "2");
        let tick_rotated = tick.set("transform", format!("rotate({})", angle));
        ticks_group = ticks_group.add(tick_rotated);

        let label_pos = label_position(
            angle,
            label_position_radius as f64,
            text_height(),
            Region::Inner,
        );

        let label = Text::new(format!("{}", point.name))
            .set("text-anchor", anchor(angle, Region::Inner))
            .set("x", label_pos.x)
            .set("y", label_pos.y);
        label_group = label_group.add(label);
    }

    ret = ret.add(ticks_group);
    ret = ret.add(label_group);
    ret
}

/*
<path d="M 180 180 L 110 35 A 160 160 0 0 1 250 35 Z"
fill="white" stroke="white" stroke-width="3"/>
 */
pub fn render(total_size: &IntegerSize2D, model: &model::WheelModel) -> String {
    // TODO: remove hardcoded values.
    // crash with height=60.
    let wheel_width = 10;
    let margin = 20;
    let page = Page {
        total_size: total_size.clone(),
        wheel_width,
        margin,
    };

    let size = page.size();

    let mut document = Document::new()
        .set("width", size.width)
        .set("height", size.height)
        .set("viewBox", (0, 0, size.width, size.height));

    let main_group = Group::new()
        .set("id", "world")
        .set("shape-rendering", "geometricPrecision")
        .set("font-size", format!("{}", 12f64));

    let center = page.center();

    let outer_circle = Circle::new()
        .set("cx", center.x)
        .set("cy", center.y)
        .set("r", page.wheel_outer_radius())
        .set("fill", "#f0f0f0")
        .set("stroke", "#333")
        .set("stroke-width", 3);
    let arcradius = page.wheel_outer_radius() as f64 + 3f64;
    let yarc = center.y as f64 - arcradius * (0.5 * constants::ARCANGLE.to_radians()).cos();
    let x1arc = center.x as f64 - arcradius * (0.5 * constants::ARCANGLE.to_radians()).sin();
    let x2arc = center.x as f64 + arcradius * (0.5 * constants::ARCANGLE.to_radians()).sin();
    let d = format!(
        "M {} {} L {} {} A {} {} 0 0 1 {} {} Z",
        center.x,
        center.y,
        x1arc,
        yarc,
        page.wheel_outer_radius(),
        page.wheel_outer_radius(),
        x2arc,
        yarc
    );
    let middle_arc = Path::new()
        .set("d", d)
        .set("fill", "white")
        .set("stroke", "white")
        .set("stroke-width", 0);

    let inner_circle = Circle::new()
        .set("cx", center.x)
        .set("cy", center.y)
        .set("r", page.wheel_inner_radius())
        .set("fill", "#f0f0f0")
        .set("stroke", "#555")
        .set("stroke-width", 2);

    let features = features(&page, &model);

    // 6. Add the central hub circle
    let center_dot = Circle::new()
        .set("cx", center.x)
        .set("cy", center.y)
        .set("r", 5)
        .set("fill", "#333");

    // 7. Assemble the final SVG
    let assembled_group = main_group
        .add(outer_circle)
        .add(inner_circle)
        .add(middle_arc)
        .add(features)
        .add(center_dot);

    document = document.add(assembled_group);
    document.to_string()
}

#[cfg(test)]
mod tests {
    use crate::{math::IntegerSize2D, wheel::model::*, wheel::*};

    fn create_wheel_model(nmid: usize) -> WheelModel {
        // 1. Define the Control Points
        let control_points = vec![
            CirclePoint {
                angle: 360.0 - 20.0,
                name: String::from("End"),
            },
            CirclePoint {
                angle: 60.0,
                name: String::from("Angers"),
            },
            CirclePoint {
                angle: 180.0,
                name: String::from("Winterstettenstadt"),
            },
            CirclePoint {
                angle: 250.0,
                name: String::from("Noirmoutiers"),
            },
        ];

        let mut mid_points = Vec::new();
        let step_angle = 360.0 / (nmid as f64);

        for i in 0..nmid {
            mid_points.push(CirclePoint {
                angle: step_angle * (i as f64),
                name: format!("I{}", i + 1),
            });
        }

        let time_points = vec![
            CirclePoint {
                angle: 60.0,
                name: String::from("3"),
            },
            CirclePoint {
                angle: 120.0,
                name: String::from("6"),
            },
            CirclePoint {
                angle: 180.0,
                name: String::from("12"),
            },
            CirclePoint {
                angle: 240.0,
                name: String::from("Wed"),
            },
        ];

        WheelModel {
            control_points,
            mid_points,
            has_start_control: false,
            has_end_control: true,
            time_points,
            outer_arcs: Vec::new(),
        }
    }

    #[tokio::test]
    async fn svg_wheel() {
        let _ = env_logger::try_init();
        for n in [50] {
            let reffilename = std::format!("data/ref/wheel-{}.svg", n);
            let data = if std::fs::exists(&reffilename).unwrap() {
                std::fs::read_to_string(&reffilename).unwrap()
            } else {
                String::new()
            };
            let model = create_wheel_model(n);
            let svg = render(&IntegerSize2D::new(400, 400), &model);

            let tmpfilename = std::format!("/tmp/wheel-{}.svg", n);
            std::fs::write(&tmpfilename, svg.clone()).unwrap();
            if data != svg {
                println!("test failed: {} {}", tmpfilename, reffilename);
                assert!(false);
            }
        }
    }
}
