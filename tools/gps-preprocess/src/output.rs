use crate::data::{Input, Smooth};

fn align_to_grid(source: &[Input], grid_times: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut distances = vec![f64::NAN; grid_times.len()];
    let mut elevations = vec![f64::NAN; grid_times.len()];
    let mut src_idx = 0;
    for (i, t) in grid_times.iter().enumerate() {
        while src_idx < source.len() && source[src_idx].time < *t - 0.5 {
            src_idx += 1;
        }
        if src_idx < source.len() && (source[src_idx].time - *t).abs() <= 0.5 {
            distances[i] = source[src_idx].distance;
            elevations[i] = source[src_idx].elevation;
        }
    }
    (distances, elevations)
}

pub fn write_csv(
    smooth: &[Smooth],
    raw: &[Input],
    oversampled: &[Input],
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;

    wtr.write_record(&[
        "time",
        "distance",
        "elevation",
        "speed",
        "smooth_speed",
        "slope",
        "raw_distance",
        "raw_elevation",
        "oversample_distance",
        "oversample_elevation",
        "measured_power",
    ])?;

    let grid_times: Vec<f64> = smooth.iter().map(|s| s.time).collect();
    let (raw_dist, raw_elev) = align_to_grid(raw, &grid_times);
    let (over_dist, over_elev) = align_to_grid(oversampled, &grid_times);

    for (i, s) in smooth.iter().enumerate() {
        wtr.write_record(&[
            format!("{:.4}", s.time / 3600.0),
            format!("{:.6}", s.distance / 1000.0),
            format!("{:.2}", s.elevation),
            format!("{:.6}", s.speed * 3.6),
            format!("{:.6}", s.smooth_speed * 3.6),
            format!("{:.6}", s.slope * 100.0),
            format_opt(raw_dist[i] / 1000.0),
            format_opt(raw_elev[i]),
            format_opt(over_dist[i] / 1000.0),
            format_opt(over_elev[i]),
            format_opt(s.measured_power),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

fn format_opt(val: f64) -> String {
    if val.is_nan() {
        String::new()
    } else {
        format!("{:.6}", val)
    }
}
