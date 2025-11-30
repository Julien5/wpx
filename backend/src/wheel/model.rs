pub struct ControlPoint {
    pub angle: f64,
    pub name: String,
}

pub struct MidPoint {
    pub angle: f64,
    pub name: String,
}

pub struct WheelModel {
    pub control_points: Vec<ControlPoint>,
    pub mid_points: Vec<MidPoint>,
}
