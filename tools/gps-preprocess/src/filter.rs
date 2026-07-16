use crate::data::{interpolate_nans_slice, Input};

pub fn filter_outliers(input: &[Input], max_speed_kmh: f64, max_vert_speed: f64) -> Vec<Input> {
    let max_speed_ms = max_speed_kmh / 3.6;
    let mut result = input.to_vec();

    for item in result.iter_mut() {
        if item.speed.abs() > max_speed_ms || item.vertical_speed.abs() > max_vert_speed {
            item.distance = f64::NAN;
            item.elevation = f64::NAN;
            item.speed = f64::NAN;
            item.vertical_speed = f64::NAN;
        }
    }

    let mut distances: Vec<f64> = result.iter().map(|x| x.distance).collect();
    let mut elevations: Vec<f64> = result.iter().map(|x| x.elevation).collect();
    let mut speeds: Vec<f64> = result.iter().map(|x| x.speed).collect();
    let mut vert_speeds: Vec<f64> = result.iter().map(|x| x.vertical_speed).collect();

    interpolate_nans_slice(&mut distances);
    interpolate_nans_slice(&mut elevations);
    interpolate_nans_slice(&mut speeds);
    interpolate_nans_slice(&mut vert_speeds);

    for (i, item) in result.iter_mut().enumerate() {
        item.distance = distances[i];
        item.elevation = elevations[i];
        item.speed = speeds[i];
        item.vertical_speed = vert_speeds[i];
    }

    result
}
