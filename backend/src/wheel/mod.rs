pub mod model;

use svg::node::element::path::Data;
use svg::node::element::Text;
use svg::node::element::{Circle, Group, Path};
use svg::Document;

use crate::math::*;

pub fn render(total_size: &IntegerSize2D, model: &model::WheelModel) -> String {
    let margin = 20;
    let size = IntegerSize2D::new(
        total_size.width - 2 * margin,
        total_size.height - 2 * margin,
    );
    let min_tick = 10;
    let wheel_width = 20;

    let center = (size / 2).to_vector();
    let mut document = Document::new()
        .set("width", size.width)
        .set("height", size.height)
        .set("viewBox", (0, 0, size.width, size.height));

    let main_group = Group::new();

    let outer_circle = Circle::new()
        .set("cx", center.x)
        .set("cy", center.y)
        .set("r", size.width / 2 - margin)
        .set("fill", "#f0f0f0")
        .set("stroke", "#333")
        .set("stroke-width", 3);

    let inner_circle = Circle::new()
        .set("cx", center.x)
        .set("cy", center.y)
        .set("r", size.width / 2 - margin - wheel_width)
        .set("fill", "#f0f0f0")
        .set("stroke", "#333")
        .set("stroke-width", 2);

    let mut ticks_group = Group::new().set(
        "transform",
        format!("translate({}, {})", center.x, center.y),
    );

    let hour_tick_start = center.x - wheel_width;
    let hour_tick_stop = center.x - margin - 1;
    let hour_tick_data =
        Data::parse(format!("M 0 -{} L 0 -{}", hour_tick_start, hour_tick_stop).as_str()).unwrap();
    let min_tick_start = hour_tick_stop - min_tick;
    let min_tick_stop = hour_tick_stop;
    let minute_tick_data =
        Data::parse(format!("M 0 -{} L 0 -{}", min_tick_start, min_tick_stop).as_str()).unwrap();

    for i in 0..model.control_points.len() {
        let point = &model.control_points[i];
        let angle = point.angle;

        let tick = Path::new()
            .set("d", hour_tick_data.clone())
            .set("stroke", "#333")
            .set("stroke-width", 5.0)
            .set("stroke-linecap", "round");
        let tick_rotated = tick.set("transform", format!("rotate({})", angle));
        ticks_group = ticks_group.add(tick_rotated);
        //let Kname = format!("K{}", i + 1);
        let mut Kname = point.name.clone();
        Kname.truncate(1);
        let label = Text::new(Kname)
            .set("text-anchor", "middle")
            .set("x", 0)
            .set("y", -hour_tick_stop - 10)
            .set("transform", format!("rotate({})", angle));
        ticks_group = ticks_group.add(label);
    }

    for i in 0..model.mid_points.len() {
        let point = &model.mid_points[i];
        let angle = point.angle;
        let tick = Path::new()
            .set("d", minute_tick_data.clone())
            .set("stroke", "#666")
            .set("stroke-width", 1.0)
            .set("stroke-linecap", "round");
        let tick_rotated = tick.set("transform", format!("rotate({})", angle));
        ticks_group = ticks_group.add(tick_rotated);
    }

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
        .add(ticks_group)
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
            ControlPoint {
                angle: 0.0,
                name: String::from("Start/End"),
            },
            ControlPoint {
                angle: 60.0,
                name: String::from("Angers"),
            },
            ControlPoint {
                angle: 180.0,
                name: String::from("Winterstettenstadt"),
            },
            ControlPoint {
                angle: 250.0,
                name: String::from("Noirmoutiers"),
            },
        ];

        let mut mid_points = Vec::new();
        let step_angle = 360.0 / (nmid as f64);

        for i in 0..nmid {
            mid_points.push(MidPoint {
                angle: step_angle * (i as f64),
                name: format!("I{}", i + 1),
            });
        }

        WheelModel {
            control_points,
            mid_points,
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
