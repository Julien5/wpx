use chrono::TimeDelta;

use crate::{mercator::DateTime, point_collection::Kind, waypoint::Waypoint};

// from mps to kmh
pub fn _kmh(_mps: f64) -> f64 {
    // m/s => kmh
    _mps * 3.6f64
}

// from kmh to mps
pub fn mps(_kmh: f64) -> f64 {
    _kmh / 3.6f64
}

// ACP (Audax Club Parisien) control closing time rules:
// Staggered minimum speeds based on distance segments:
//   - 0-600 km: 15.0 km/h
//   - 600-1000 km: 11.428 km/h (8/7 km/h)
//   - 1000-1300 km: 13.333 km/h (40/3 km/h)
// Special case for short distances (0-60 km): T = 1 + (D / 20)
fn duration_to_distance_acp(distance: f64) -> f64 {
    let distance_km = distance / 1000.0;

    // Calculate time in hours based on ACP rules
    let time_hours = if distance_km <= 60.0 {
        // Short distance exception: grace period
        1.0 + (distance_km / 20.0)
    } else if distance_km <= 600.0 {
        // Segment 1: 0-600 km at 15.0 km/h
        distance_km / 15.0
    } else if distance_km <= 1000.0 {
        // Segment 1: 0-600 km at 15.0 km/h
        // Segment 2: 600-1000 km at 11.428 km/h
        (600.0 / 15.0) + ((distance_km - 600.0) / 11.428)
    } else if distance_km <= 1300.0 {
        // Segment 1: 0-600 km at 15.0 km/h
        // Segment 2: 600-1000 km at 11.428 km/h
        // Segment 3: 1000-1300 km at 13.333 km/h
        (600.0 / 15.0) + (400.0 / 11.428) + ((distance_km - 1000.0) / 13.333)
    } else {
        // Beyond 1300 km: continue with 13.333 km/h
        (600.0 / 15.0) + (400.0 / 11.428) + ((distance_km - 1000.0) / 13.333)
    };

    // Apply event finish hard-caps for standard brevet distances
    // These are checked based on the control distance matching standard distances
    let capped_time_hours = {
        let rounded_km = distance_km.round();
        let hard_cap = if (rounded_km - 200.0).abs() < 1.0 {
            Some(13.5) // 200 km: 13h 30m
        } else if (rounded_km - 300.0).abs() < 1.0 {
            Some(20.0) // 300 km: 20h 00m
        } else if (rounded_km - 400.0).abs() < 1.0 {
            Some(27.0) // 400 km: 27h 00m
        } else if (rounded_km - 600.0).abs() < 1.0 {
            Some(40.0) // 600 km: 40h 00m
        } else if (rounded_km - 1000.0).abs() < 1.0 {
            Some(75.0) // 1000 km: 75h 00m
        } else {
            None // No hard cap for other distances
        };

        if let Some(cap) = hard_cap {
            time_hours.min(cap)
        } else {
            time_hours
        }
    };

    capped_time_hours * 3600.0 // convert hours to seconds
}

pub fn duration_distance(distance: f64, speed: &Speed) -> TimeDelta {
    let seconds = match speed {
        Speed::ACP => duration_to_distance_acp(distance),
        Speed::MPS(mps) => distance / mps,
    };
    TimeDelta::nanoseconds((1000_000_000f64 * seconds).round() as i64)
}

pub fn time_at_distance(distance: f64, start_time: &DateTime, speed: &Speed) -> DateTime {
    let delta = duration_distance(distance, &speed);
    *start_time + delta
}

#[derive(Clone, Debug, Default)]
pub struct ControlSpeedData {
    pub track_index: usize,
    pub distance: f64,
    pub time: Option<DateTime>,
}

#[derive(Clone, Default)]
pub struct TimeParameters {
    pub controls: Vec<ControlSpeedData>,
    pub start: DateTime,
    pub speed: Speed,
    pub total_distance: f64,
}

impl TimeParameters {
    pub fn total_duration(&self) -> TimeDelta {
        duration_distance(self.total_distance, &self.speed)
    }
    pub fn time_at_waypoint(&self, waypoint: &Waypoint, distance: f64) -> DateTime {
        let index = waypoint.track_index.unwrap();
        match waypoint.origin {
            Kind::Controls => {
                let control = self
                    .controls
                    .iter()
                    .find(|c| c.track_index == index)
                    .unwrap();
                time_at_control(control, &self.start, &self.speed)
            }
            _ => time_at_distance_with_controls(&self.controls, distance, &self.start, &self.speed),
        }
    }
}

pub fn time_at_control(
    control: &ControlSpeedData,
    start_time: &DateTime,
    speed: &Speed,
) -> DateTime {
    control
        .time
        .unwrap_or(time_at_distance(control.distance, start_time, speed))
}

pub fn time_at_distance_with_controls(
    controls: &Vec<ControlSpeedData>,
    distance: f64,
    start_time: &DateTime,
    speed: &Speed,
) -> DateTime {
    // controls has to be sorted by distance and time
    // controls must contains START and END.
    let maybe = controls
        .iter()
        .enumerate()
        .find(|(_, c)| c.distance >= distance);
    if maybe.is_none() {
        log::info!("could not find next control for distance={:.1}", distance);
        for c in controls {
            log::info!("control {:?}", c);
        }
        return time_at_distance(distance, start_time, speed);
    }
    let (index_next, next) = maybe.unwrap();
    if index_next == 0 {
        return start_time.clone();
    }
    let previous = &controls[index_next - 1];

    let normal_previous_time = time_at_distance(previous.distance, start_time, speed);
    let real_previous_time = time_at_control(previous, start_time, speed);
    let normal_next_time = time_at_distance(next.distance, start_time, speed);
    let real_next_time = time_at_control(next, start_time, speed);

    let normal_time = time_at_distance(distance, start_time, speed);
    debug_assert!(normal_time <= normal_next_time);

    let delta1 = normal_time - normal_previous_time;
    let delta2 = normal_next_time - normal_previous_time;
    let delta2real = real_next_time - real_previous_time;

    let lambda = delta1.as_seconds_f64() / delta2.as_seconds_f64();
    let seconds = lambda * delta2real.as_seconds_f64();
    // i64 + nanos => 290 years max.
    let ret = time_at_control(&previous, start_time, speed)
        + TimeDelta::nanoseconds((1_000_000_000f64 * seconds).round() as i64);
    debug_assert!(lambda <= 1f64);
    ret
}

pub fn distance_after_duration_with_controls(
    _controls: &Vec<ControlSpeedData>,
    duration: TimeDelta,
    speed: &Speed,
) -> f64 {
    match speed {
        Speed::ACP => duration.as_seconds_f64() * 15.0 * 1000.0 / 3600.0, // FIXME
        Speed::MPS(mps) => duration.as_seconds_f64() * mps,
    }
}

pub fn distance_after_duration(duration: TimeDelta, speed: &Speed) -> f64 {
    match speed {
        Speed::ACP => duration.as_seconds_f64() * 15.0 * 1000.0 / 3600.0, // FIXME
        Speed::MPS(mps) => duration.as_seconds_f64() * mps,
    }
}

#[derive(Clone, Debug)]
pub enum Speed {
    MPS(f64),
    ACP,
}

impl Default for Speed {
    fn default() -> Self {
        Speed::MPS(15.0 * 1000.0 / 3600.0)
    }
}

pub fn parse_speed(data: &str) -> Speed {
    if data == "ACP" {
        return Speed::ACP;
    }
    let ok = data.parse().ok();
    debug_assert!(ok.is_some());
    let kmh: f64 = ok.unwrap();
    Speed::MPS(kmh * 1000.0 / 3600.0)
}

#[cfg(test)]
mod tests {
    use crate::{
        parameters::{self, Parameters},
        speed,
    };

    use super::*;

    #[test]
    fn test_constant_speed_mode() {
        let mut params = Parameters::default();
        params.speed = format!("{}", 15.0);
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 300 km at 15 km/h should take 20 hours
        let dist_300km = 300_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_300 = speed::time_at_distance(dist_300km, &start, &speed);
        let duration_sec = (time_300 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;

        assert!(
            (duration_hours - 20.0).abs() < 0.01,
            "Expected ~20 hours, got {}",
            duration_hours
        );
    }

    #[test]
    fn test_acp_short_distance() {
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // Short distance (< 60 km): T = 1 + (D / 20)
        // 40 km should take: 1 + (40/20) = 3 hours
        let dist_40km = 40_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_40 = time_at_distance(dist_40km, &start, &speed);
        let duration_sec = (time_40 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 1.0 + (40.0 / 20.0); // 3 hours

        assert!(
            (duration_hours - expected).abs() < 0.01,
            "Expected ~{} hours for 40km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_below_600km() {
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 300 km at 15 km/h should take 20 hours
        let dist_300km = 300_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_300 = time_at_distance(dist_300km, &start, &speed);
        let duration_sec = (time_300 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;

        assert!(
            (duration_hours - 20.0).abs() < 0.01,
            "Expected ~20 hours for 300km, got {}",
            duration_hours
        );
    }

    #[test]
    fn test_acp_at_600km() {
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 600 km: hard cap should be 40 hours
        let dist_600km = 600_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_600 = time_at_distance(dist_600km, &start, &speed);
        let duration_sec = (time_600 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;

        assert!(
            (duration_hours - 40.0).abs() < 0.01,
            "Expected 40 hours for 600km (hard cap), got {}",
            duration_hours
        );
    }

    #[test]
    fn test_acp_between_600_and_1000km() {
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);
        let speed = speed::parse_speed(&params.speed);
        // 800 km: 600/15 + (800-600)/11.428
        //       = 40 + 200/11.428 = 40 + 17.5 = 57.5 hours
        let dist_800km = 800_000.0;
        let time_800 = time_at_distance(dist_800km, &start, &speed);
        let duration_sec = (time_800 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 40.0 + (200.0 / 11.428); // ~57.5 hours

        assert!(
            (duration_hours - expected).abs() < 0.1,
            "Expected ~{} hours for 800km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_at_1000km() {
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 1000 km: hard cap should be 75 hours
        // Calculated: 600/15 + 400/11.428 = 40 + 35 = 75 hours
        let dist_1000km = 1_000_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_1000 = time_at_distance(dist_1000km, &start, &speed);
        let duration_sec = (time_1000 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;

        assert!(
            (duration_hours - 75.0).abs() < 0.1,
            "Expected 75 hours for 1000km (hard cap), got {}",
            duration_hours
        );
    }

    #[test]
    fn test_acp_above_1000km() {
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 1200 km: 600/15 + 400/11.428 + 200/13.333
        //        = 40 + 35 + 15 = 90 hours
        let dist_1200km = 1_200_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_1200 = time_at_distance(dist_1200km, &start, &speed);
        let duration_sec = (time_1200 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 40.0 + (400.0 / 11.428) + (200.0 / 13.333);

        assert!(
            (duration_hours - expected).abs() < 0.1,
            "Expected ~{} hours for 1200km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_standard_brevets() {
        // Test the standard brevet distances with their hard caps
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        // Test each standard distance
        let test_cases = vec![
            (200_000.0, 13.5),   // 200 km: 13h 30m
            (300_000.0, 20.0),   // 300 km: 20h 00m
            (400_000.0, 27.0),   // 400 km: 27h 00m
            (600_000.0, 40.0),   // 600 km: 40h 00m
            (1_000_000.0, 75.0), // 1000 km: 75h 00m
        ];

        let start = parameters::parse_time(&params.start_time);
        let speed = speed::parse_speed(&params.speed);
        for (distance, expected_hours) in test_cases {
            let time = time_at_distance(distance, &start, &speed);
            let duration_hours = (time - start).num_seconds() as f64 / 3600.0;
            println!("{} km: {:.2} hours", distance / 1000.0, duration_hours);
            assert!(
                duration_hours <= expected_hours + 0.1,
                "{} km should not exceed {} hours, got {}",
                distance / 1000.0,
                expected_hours,
                duration_hours
            );
        }
    }
}
