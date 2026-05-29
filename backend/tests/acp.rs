use chrono::TimeDelta;
use tracks::{
    backend::Backend,
    mercator::DateTime,
    parameters::{self, RenderFunction, RenderInput},
    point_collection::{self, Kind},
    waypoint::Waypoint,
};

static START_TIME: &'static str = "2026-04-12T00:00:00";

fn load_test_data(filename: &str) -> Backend {
    let mut backend = Backend::make();
    backend
        .load_filename(filename)
        .expect(&format!("failed to load {}", filename));
    backend.load_controls().unwrap();
    backend
}

fn format_delta(delta: &TimeDelta) -> String {
    let hours = delta.num_hours();
    let minutes = delta.num_minutes() % 60;
    format!("{:02}:{:02}", hours, minutes)
}

fn table(backend: &Backend) -> Vec<Waypoint> {
    let map = RenderInput {
        kinds: point_collection::allkinds(),
        function: RenderFunction::Map,
        size: (400, 400),
    };
    let profile = RenderInput {
        kinds: point_collection::allkinds(),
        function: RenderFunction::Profile,
        size: (1000, 300),
    };
    let segment = backend.trackSegment();
    let outputs = backend.render_segment(&segment, &vec![map, profile]);
    assert_eq!(outputs.len(), 2);
    let mut table = outputs[1].waypoints.clone();
    table.sort_by(|p1, p2| {
        let a1 = if let Some(i) = p1.info.as_ref() {
            i.distance
        } else {
            0f64
        };
        let a2 = if let Some(i) = p2.info.as_ref() {
            i.distance
        } else {
            0f64
        };
        a1.partial_cmp(&a2).unwrap()
    });
    table
}

fn display_table(result: &Vec<Waypoint>, start_time: &DateTime) {
    for (index, p) in result.iter().enumerate() {
        let info = p.info.as_ref().unwrap();
        let time = parameters::parse_time(&info.time);
        let duration = time - start_time;
        log::info!(
            "[{:3}] | {} | {} | {:6.1} | {:16} | {:32} | {:?}",
            index,
            time.format("%d-%H:%M"),
            format_delta(&duration),
            info.distance / 1000.0,
            p.name,
            p.description,
            p.origin
        );
    }
}

#[test]
fn test_constant_strech() {
    let _ = env_logger::try_init();
    let mut backend = load_test_data(&"data/PBP-simple.gpx");
    let mut parameters = backend.get_parameters();
    parameters.start_time = START_TIME.to_string();
    parameters.speed = format!("KMH-{}", 28.89);
    backend.set_parameters(&parameters);
    let start_time = parameters::parse_time(&parameters.start_time);
    let result = table(&backend);
    display_table(&result, &start_time);

    // Fougere at 288.9km => 10h @ 28.89km/h
    let control_index = 2;

    assert_eq!(result.len(), 11);
    assert_eq!(result[control_index].name, "Fougeres");
    assert_eq!(result[control_index].origin, Kind::GPXWaypoints);

    backend.make_control_at_waypoint(&result[control_index], true);
    let result = table(&backend);
    let mortagne_time = parameters::parse_time(&result[1].info.as_ref().unwrap().time);
    assert_eq!(format!("{}", mortagne_time.format("%H:%M")), "04:05");
    let loudeac_time = parameters::parse_time(&result[3].info.as_ref().unwrap().time);
    assert_eq!(format!("{}", loudeac_time.format("%H:%M")), "14:57");

    let ok = backend.set_control_time(
        &result[control_index],
        &Some(format!("2026-04-12T20:00:00")),
    );
    assert_eq!(ok, true);
    let result = table(&backend);
    display_table(&result, &start_time);

    // 04:05 => 08:10
    let control_time = parameters::parse_time(&result[control_index].info.as_ref().unwrap().time);
    assert_eq!(format!("{}", control_time.format("%H:%M")), "20:00");

    // 04:05 => 08:10
    let mortagne_time = parameters::parse_time(&result[1].info.as_ref().unwrap().time);
    assert_eq!(format!("{}", mortagne_time.format("%H:%M")), "08:10");

    // 14:57 =>
    let loudeac_time = parameters::parse_time(&result[3].info.as_ref().unwrap().time);
    assert!(loudeac_time > control_time);
}

#[test]
fn test_acp_strech() {
    let _ = env_logger::try_init();
    let mut backend = load_test_data(&"data/PBP-simple.gpx");
    let mut parameters = backend.get_parameters();
    parameters.start_time = START_TIME.to_string();
    parameters.speed = format!("ACP-1200-90.0");
    backend.set_parameters(&parameters);
    let result = table(&backend);
    let start_time = parameters::parse_time(&parameters.start_time);
    display_table(&result, &start_time);

    // Brest
    let control_index = 5;

    assert_eq!(result.len(), 11);
    assert_eq!(result[control_index].name, "Brest");
    assert_eq!(result[control_index].origin, Kind::GPXWaypoints);

    backend.make_control_at_waypoint(&result[control_index], true);
    let result = table(&backend);
    let control_time = parameters::parse_time(&result[control_index].info.as_ref().unwrap().time);
    assert_eq!(format!("{}", control_time.format("%d-%H:%M")), "13-15:59");
    let fougere_time = parameters::parse_time(&result[8].info.as_ref().unwrap().time);
    assert_eq!(format!("{}", fougere_time.format("%d-%H:%M")), "14-20:04");

    let ok = backend.set_control_time(
        &result[control_index],
        &Some(format!("2026-04-12T20:00:00")),
    );
    assert_eq!(ok, false);
    let result = table(&backend);
    display_table(&result, &start_time);

    // check that nothing has changed
    let control_time = parameters::parse_time(&result[control_index].info.as_ref().unwrap().time);
    assert_eq!(format!("{}", control_time.format("%d-%H:%M")), "13-15:59");

    let fougere_time = parameters::parse_time(&result[8].info.as_ref().unwrap().time);
    assert_eq!(format!("{}", fougere_time.format("%d-%H:%M")), "14-20:04");
}
