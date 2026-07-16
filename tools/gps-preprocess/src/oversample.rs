use crate::data::{interpolate_nans_slice, Input};

pub fn oversample(input: &[Input]) -> Vec<Input> {
    if input.is_empty() {
        return Vec::new();
    }

    let t_start = input[0].time.floor() as i64;
    let t_end = input[input.len() - 1].time.ceil() as i64;
    let n = (t_end - t_start + 1) as usize;

    let mut result: Vec<Input> = Vec::with_capacity(n);

    let mut idx = 0;
    for t in t_start..=t_end {
        let t_f = t as f64;
        if idx < input.len() && (input[idx].time - t_f).abs() < 0.5 {
            result.push(input[idx]);
            idx += 1;
        } else {
            result.push(Input {
                time: t_f,
                distance: f64::NAN,
                elevation: f64::NAN,
                speed: f64::NAN,
                vertical_speed: f64::NAN,
                measured_power: f64::NAN,
            });
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
