use crate::data::{haversine, Input};

fn parse_datetime_to_seconds(_date: &str, time: &str) -> f64 {
    let mut total = 0.0;
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() == 3 {
        let h: f64 = parts[0].parse().unwrap_or(0.0);
        let m: f64 = parts[1].parse().unwrap_or(0.0);
        let s: f64 = parts[2].parse().unwrap_or(0.0);
        total += h * 3600.0 + m * 60.0 + s;
    }
    total
}

pub fn read_csv(filename: &str) -> Result<Vec<Input>, Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_path(filename)?;
    let mut data = Vec::new();
    let mut prev_time: Option<f64> = None;
    let mut prev_lat: Option<f64> = None;
    let mut prev_lon: Option<f64> = None;
    let mut prev_elev: Option<f64> = None;
    let mut total_distance = 0.0;
    let mut start_time = 0.0;
    for result in rdr.records() {
        let record = result?;
        let lat: f64 = record[1].parse()?;
        let lon: f64 = record[2].parse()?;
        let elev: f64 = record[3].parse()?;
        let power_str = &record[8];
        let date = &record[9];
        let time_str = &record[10];
        let measured_power = if power_str.is_empty() {
            f64::NAN
        } else {
            power_str.parse().unwrap_or(f64::NAN)
        };

        let abs_seconds = parse_datetime_to_seconds(date, time_str);

        if let Some(prev_abs) = prev_time {
            let dt = abs_seconds - prev_abs;
            if dt == 0.0 {
                continue;
            }

            let d = haversine(prev_lat.unwrap(), prev_lon.unwrap(), lat, lon);
            total_distance += d;

            let speed = d / dt;
            let vert_speed = (elev - prev_elev.unwrap()) / dt;
            let elapsed = abs_seconds - start_time;

            data.push(Input {
                time: elapsed,
                distance: total_distance,
                elevation: elev,
                speed,
                vertical_speed: vert_speed,
                measured_power,
            });

            prev_time = Some(abs_seconds);
            prev_lat = Some(lat);
            prev_lon = Some(lon);
            prev_elev = Some(elev);
        } else {
            start_time = abs_seconds;
            data.push(Input {
                time: 0.0,
                distance: 0.0,
                elevation: elev,
                speed: 0.0,
                vertical_speed: 0.0,
                measured_power,
            });
            prev_time = Some(abs_seconds);
            prev_lat = Some(lat);
            prev_lon = Some(lon);
            prev_elev = Some(elev);
        }
    }

    Ok(data)
}
