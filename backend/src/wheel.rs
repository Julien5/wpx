use svg::node::element::path::Data;
use svg::node::element::{Circle, Group, Path};
use svg::Document;

use crate::math::*;

pub fn render(size: IntegerSize2D) {
    let hour_tick = 15;
    let min_tick = 10;
    let wheel_width = 20;
    let margin = 10;

    let center = (size / 2).to_vector();
    let mut document = Document::new()
        .set("width", size.width)
        .set("height", size.height)
        .set("viewBox", (0, 0, size.width, size.height));

    // 2. Create the main group element, replicating the top-level <g>
    let main_group = Group::new();

    // 3. Add the outer circle (Dial)
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

    // 4. Create the nested group for the Ticks, translated to the center (200, 200)
    let mut ticks_group = Group::new().set(
        "transform",
        format!("translate({}, {})", center.x, center.y),
    );

    // Define the Path data strings
    let hour_tick_start = center.x - wheel_width;
    let hour_tick_stop = center.x - margin - 1;
    let hour_tick_data =
        Data::parse(format!("M 0 -{} L 0 -{}", hour_tick_start, hour_tick_stop).as_str()).unwrap();
    let min_tick_start = hour_tick_stop - min_tick;
    let min_tick_stop = hour_tick_stop;
    let minute_tick_data =
        Data::parse(format!("M 0 -{} L 0 -{}", min_tick_start, min_tick_stop).as_str()).unwrap();

    // 5. Generate Ticks (60 total)
    for i in 0..60 {
        let angle = i as f64 * 6.0; // 360 degrees / 60 minutes = 6 degrees per tick

        let tick = if i % 5 == 0 {
            // Hour Tick (every 5th minute mark)
            Path::new()
                .set("d", hour_tick_data.clone())
                .set("stroke", "#333")
                .set("stroke-width", 4.0)
                .set("stroke-linecap", "round")
        } else {
            // Minute Tick
            Path::new()
                .set("d", minute_tick_data.clone())
                .set("stroke", "#666")
                .set("stroke-width", 1.5)
                .set("stroke-linecap", "round")
        };

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

    // Write the document to stdout (or a file)
    // To write to a file, uncomment the line below and comment the one after it:
    // svg::save("watch_dial.svg", &document).unwrap();
    println!("{}", document);
}
