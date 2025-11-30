use svg::node::element::path::Data;
use svg::node::element::{Circle, Group, Path};
use svg::Document;

const SIZE: f64 = 400.0;
const CENTER: f64 = SIZE / 2.0;

pub fn render() {
    // 1. Define the main SVG document with the correct dimensions
    let mut document = Document::new()
        .set("width", SIZE)
        .set("height", SIZE)
        .set("viewBox", (0, 0, SIZE, SIZE))
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("title", "Minimalist Clock Face with Hour and Minute Ticks");

    // 2. Create the main group element, replicating the top-level <g>
    let main_group = Group::new();

    // 3. Add the outer circle (Dial)
    let dial_circle = Circle::new()
        .set("cx", CENTER)
        .set("cy", CENTER)
        .set("r", 190)
        .set("fill", "#f0f0f0")
        .set("stroke", "#333")
        .set("stroke-width", 3);

    // 4. Create the nested group for the Ticks, translated to the center (200, 200)
    let mut ticks_group =
        Group::new().set("transform", format!("translate({}, {})", CENTER, CENTER));

    // Define the Path data strings
    let hour_tick_data = Data::parse("M 0 -170 L 0 -185").unwrap();
    let minute_tick_data = Data::parse("M 0 -177 L 0 -185").unwrap();

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
        .set("cx", CENTER)
        .set("cy", CENTER)
        .set("r", 5)
        .set("fill", "#333");

    // 7. Assemble the final SVG
    let assembled_group = main_group.add(dial_circle).add(ticks_group).add(center_dot);

    document = document.add(assembled_group);

    // Write the document to stdout (or a file)
    // To write to a file, uncomment the line below and comment the one after it:
    // svg::save("watch_dial.svg", &document).unwrap();
    println!("{}", document);
}
