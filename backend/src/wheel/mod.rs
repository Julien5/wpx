mod arc;
pub mod model;
pub mod shorten;
mod text;
mod time_points;

use svg::node::element::path::Data;
use svg::node::element::Text;
use svg::node::element::{Circle, Group, Path};
use svg::Document;

use crate::math::{IntegerSize2D, Point2D};
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
    fn center(&self) -> Point2D {
        Point2D::new(self.total_size.width as f64, self.total_size.height as f64) * 0.5
    }
    fn make_centered_group(&self) -> Group {
        let center = self.center();
        Group::new().set(
            "transform",
            format!("translate({}, {})", center.x, center.y),
        )
    }
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

    let label_pos = text::position(
        angle,
        (page.wheel_outer_radius() + 7) as f64,
        text::height(),
        text::Region::Outer,
    );

    let name = point.name.clone();
    log::trace!("name={}", name);

    let label = Text::new(format!("{}", name))
        .set("text-anchor", text::anchor(angle, text::Region::Outer))
        .set("x", label_pos.x)
        .set("y", label_pos.y);
    label_group = label_group.add(label);
    (ticks_group, label_group)
}

fn draw_arc1(page: &Page, m: &model::Arc) -> Group {
    let mut ret = Group::new();
    let start = m.start_angle;
    let end = m.end_angle;
    let dash_length = 5f64;

    let zero = Point2D::new(0f64, 0f64);
    let radius = page.wheel_inner_radius() as f64 / 2.0;

    let a1 = arc::Arc {
        center: zero,
        radius,
        angle1: start + 5f64,
        angle2: end - 5f64,
    };
    let dash_start = Path::new()
        .set(
            "d",
            a1.dash(radius - dash_length, radius + dash_length, start),
        )
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 4);
    ret = ret.add(dash_start);

    let dash_end = Path::new()
        .set(
            "d",
            a1.dash(radius - dash_length, radius + dash_length, end),
        )
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 4);
    ret = ret.add(dash_end);

    let arc1 = Path::new()
        .set("d", a1.open_path())
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3);
    ret = ret.add(arc1);
    ret
}

fn draw_arc2(page: &Page, m: &model::Arc) -> Group {
    let mut ret = Group::new();
    let start = m.start_angle;
    let mid = m.middle_angle.unwrap();
    let end = m.end_angle;

    let zero = Point2D::new(0f64, 0f64);
    let radius = page.wheel_inner_radius() as f64 / 2.0;
    let dash_length = 5f64;

    let a1 = arc::Arc {
        center: zero,
        radius,
        angle1: start + 5f64,
        angle2: mid - 2f64,
    };
    let a2 = arc::Arc {
        center: zero,
        radius,
        angle1: mid + 2f64,
        angle2: end - 5f64,
    };
    let dash_start = Path::new()
        .set(
            "d",
            a1.dash(radius - dash_length, radius + dash_length, start),
        )
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 4);
    ret = ret.add(dash_start);

    let dash_end = Path::new()
        .set(
            "d",
            a1.dash(radius - dash_length, radius + dash_length, end),
        )
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 4);
    ret = ret.add(dash_end);

    /*
    let mid_dash_length = 5f64;
    let dash_mid = Path::new()
            .set(
                "d",
                a1.dash(radius - mid_dash_length, radius + mid_dash_length, mid),
            )
            .set("fill", "none")
            .set("stroke", "blue")
            .set("stroke-width", 2);
        ret = ret.add(dash_mid);*/

    let arc1 = Path::new()
        .set("d", a1.open_path())
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3);
    ret = ret.add(arc1);

    let arc2 = Path::new()
        .set("d", a2.open_path())
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3);
    ret = ret.add(arc2);
    ret
}

fn draw_arc(page: &Page, m: &model::Arc) -> Group {
    match m.middle_angle {
        Some(_) => draw_arc2(page, m),
        None => draw_arc1(page, m),
    }
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

        let label_pos = text::position(
            angle,
            label_position_radius as f64,
            text::height(),
            text::Region::Inner,
        );

        let label = Text::new(format!("{}", point.name))
            .set("text-anchor", text::anchor(angle, text::Region::Inner))
            .set("x", label_pos.x)
            .set("y", label_pos.y);
        label_group = label_group.add(label);
    }

    let mut arc_group = page.make_centered_group();
    log::debug!("n={}", model.outer_arcs.len());
    for m in &model.outer_arcs {
        arc_group = arc_group.add(draw_arc(&page, m));
    }

    ret = ret.add(ticks_group);
    ret = ret.add(label_group);
    ret = ret.add(arc_group);
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
    let radius = page.wheel_outer_radius() as f64 + 3f64;
    let arc = arc::Arc {
        center,
        radius,
        angle1: -0.5 * constants::ARCANGLE,
        angle2: 0.5 * constants::ARCANGLE,
    };
    let darc = arc.closed_path();
    let middle_arc = Path::new()
        .set("d", darc)
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
    use super::model;
    use super::model::CirclePoint;
    use super::model::WheelModel;
    use super::render;
    use crate::{math::IntegerSize2D, mercator::DateTime};
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

        let arcs = vec![
            model::Arc {
                start_angle: 45f64,
                middle_angle: Some(55f64),
                end_angle: 180f64,
                label: String::new(),
            },
            model::Arc {
                start_angle: 180f64,
                middle_angle: None,
                end_angle: 255f64,
                label: String::new(),
            },
            model::Arc {
                start_angle: 65f64,
                middle_angle: Some(180f64),
                end_angle: 300f64,
                label: String::new(),
            },
        ];

        let time_parameters = model::TimeParameters {
            start: DateTime::from_timestamp_nanos(0),
            speed: 1f64,
            total_distance: 1f64,
        };

        WheelModel {
            time_parameters,
            control_points,
            mid_points,
            has_start_control: false,
            has_end_control: true,
            time_points,
            outer_arcs: arcs,
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
