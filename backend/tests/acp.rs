use tracks::{
    backend::Backend,
    parameters::{self, RenderFunction, RenderInput},
    point_collection::{self, Kind},
    waypoint::Waypoint,
};

static START_TIME: &'static str = "1985-04-12T06:05:00.00Z";
static BLACK_FOREST: &'static str = "data/blackforest.gpx";

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
        let info = p.info.as_ref().unwrap();
        let time = parameters::parse_time(&info.time).format("%H:%M");
        log::trace!(
            "[{:3}] | {} | {:16} | {:32} | {:?}",
            index,
            time,
            p.name,
            p.description,
            p.origin
        );
    }
}

#[tokio::test]
async fn table_acp() {
    let _ = env_logger::try_init();
    let mut backend = load_test_data(&"data/PBP-simple.gpx").await;
    let mut parameters = backend.get_parameters();
    parameters.start_time = START_TIME.to_string();
    backend.set_parameters(&parameters);
    let result = table(&backend);
    display_table(&result);

    assert_eq!(result.len(), 11);
    assert_eq!(result[5].name, "Brest");
    assert_eq!(result[5].origin, Kind::GPXWaypoints);

    backend.make_control_at_waypoint(&result[5], true);
    let result = table(&backend);
    display_table(&result);

    backend.set_control_time(&result[5], Some("1985-04-13T00:22:00.00Z".to_string()));
    let result = table(&backend);
    display_table(&result);

    assert_eq!(
        result[5].info.as_ref().unwrap().time,
        "1985-04-13T00:22:00.00Z"
    );
}
