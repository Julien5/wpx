use tracks::{
    backend::Backend,
    parameters::{RenderFunction, RenderInput},
    point_collection::{self, Kind},
    waypoint::Waypoint,
};

async fn load_test_data(filename: &str) -> Backend {
    let mut backend = Backend::make();
    backend
        .load_filename(filename)
        .expect(&format!("failed to load {}", filename));
    backend.load_controls().unwrap();
    backend
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

fn display_table(result: &Vec<Waypoint>) {
    for (index, p) in result.iter().enumerate() {
        log::trace!(
            "[{:3}] | {:16} | {:32} | {:?}",
            index,
            p.name,
            p.description,
            p.origin
        );
    }
}

static START_TIME: &'static str = "1985-04-12T06:05:00.00Z";

#[tokio::test]
async fn table_1() {
    let _ = env_logger::try_init();
    let mut backend = load_test_data(&"data/PBP-simple.gpx").await;
    let mut parameters = backend.get_parameters();
    parameters.start_time = START_TIME.to_string();
    backend.set_parameters(&parameters);
    let result = table(&backend);
    display_table(&result);

    assert_eq!(result.len(), 11);
    assert_eq!(result[1].name, "Mortagne");
    assert_eq!(result[1].origin, Kind::GPXWaypoints);

    assert_eq!(result[9].name, "Mortagne");
    assert_eq!(result[9].origin, Kind::GPXWaypoints);

    backend.make_control_at_waypoint(&result[1], true);
    let result = table(&backend);
    display_table(&result);
    assert_eq!(result.len(), 11);
    assert_eq!(result[1].name, "CP-1");
    assert_eq!(result[1].origin, Kind::Controls);
    assert_eq!(result[9].name, "Mortagne");
    assert_eq!(result[9].origin, Kind::GPXWaypoints);

    backend.make_control_at_waypoint(&result[9], true);
    let result = table(&backend);
    display_table(&result);
    assert_eq!(result.len(), 11);
    assert_eq!(result[1].name, "CP-1");
    assert_eq!(result[1].origin, Kind::Controls);
    assert_eq!(result[9].name, "CP-2");
    assert_eq!(result[9].origin, Kind::Controls);
}
