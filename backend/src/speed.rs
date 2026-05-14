use chrono::TimeDelta;

use crate::{
    mercator::DateTime,
    parameters::{self, Parameters},
    point_collection::Kind,
    waypoint::Waypoint,
};

// from mps to kmh
pub fn _kmh(_mps: f64) -> f64 {
    // m/s => kmh
    _mps * 3.6f64
}

// from kmh to mps
pub fn mps(_kmh: f64) -> f64 {
    _kmh / 3.6f64
}

/*
The allocated time for RM sanctioned events of 1200 km is ninety (90) hours. For events of 1400 km and greater the total time will be based on an average global speed of twelve (12) km per hour. It is recommended that all RM sanctioned events be patterned after the PBP and use qualifying brevets to ensure participants are well prepared and non-finishers are minimize
=> 2200/12 at 2200km

https://en.wikipedia.org/wiki/Randonneuring#Time_limits => 10kmh at 2200km

https://web.archive.org/web/20210201125907/http://www.randonneursmondiaux.org/files/Rules_2019.pdf
=>
from 1200 to 1299 km: 13.33 kph
from 1300 to 1899 km: 12 kph
from 1900 to 2499 km: 10 kph
2500 km and above: 200 km per day.
 */

fn acp_distance_time_last() -> Vec<(f64, f64)> {
    vec![
        (0.0, 1.0),
        (60.0, 4.0),
        (200.0, 13.5), // 14.81 kmh
        (300.0, 20.0), // 15.00 kmh
        (400.0, 27.0), // 14.81 kmh
        (600.0, 40.0), // 15.00 kmh
        (1000.0, 75.0),
        (1200.0, 90.0),
        (1400.0, 1400.0 / 12.0),
        (2200.0, 220.0),
        (1_000_000.0, 100_000.0),
    ]
}

fn acp_distance_time_intermediate() -> Vec<(f64, f64)> {
    vec![
        (0.0, 1.0),
        (60.0, 4.0),
        (600.0, 40.0),
        (1000.0, 75.0),
        (1200.0, 90.0),
        (1400.0, 1400.0 / 12.0),
        (2200.0, 220.0),
        (1_000_000.0, 100_000.0),
    ]
}

fn duration_acp_last(distance: f64) -> f64 {
    let distance_km = distance / 1000.0;
    let distances = acp_distance_time_last();
    // Calculate time in hours based on ACP rules
    let closest = distances.iter().copied().min_by(|a, b| {
        (a.0 - distance_km)
            .abs()
            .partial_cmp(&(b.0 - distance_km).abs())
            .unwrap()
    });
    // beyond 2200km => 10kmh average
    if distance_km > 2250.0 {
        return 3600.0 * distance_km / 10f64;
    }
    match closest {
        Some(d) => {
            log::trace!("USING CONTROL ACP {} km {} hours", d.0, d.1);
            return d.1 * 3600.0;
        }
        _ => {}
    }
    0f64
}

// ACP (Audax Club Parisien) control closing time rules:
// Staggered minimum speeds based on distance segments:
//   - 0-600 km: 15.0 km/h
//   - 600-1000 km: 11.428 km/h (8/7 km/h)
//   - 1000-1300 km: 13.333 km/h (40/3 km/h)
// Special case for short distances (0-60 km): T = 1 + (D / 20)
fn duration_acp(distance: f64) -> f64 {
    let distance_km = distance / 1000.0;
    let time_hours = {
        let mut remain_km = distance_km;
        let mut prev = (0.0, 0.0);
        let mut time = 0f64;
        for (km, hours) in acp_distance_time_intermediate() {
            let (skm, shours) = (km - prev.0, hours - prev.1);
            if remain_km >= skm {
                time += shours;
                remain_km -= skm;
            } else {
                let speed = skm / shours;
                time += remain_km / speed;
                remain_km = 0f64;
                break;
            }
            prev = (km, hours);
        }
        debug_assert_eq!(remain_km, 0f64);
        time
    };

    /*else if distance_km <= 600.0 {
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
    };*/
    time_hours * 3600.0
}

fn distance_acp(seconds: f64) -> f64 {
    let time_hours = seconds / 3600.0;

    // Precompute cumulative times at segment boundaries
    let t_60: f64 = 1.0 + (60.0 / 20.0); // = 4.0 h  (end of short-distance exception)
    let t_600: f64 = 600.0 / 15.0; // = 40.0 h (end of segment 1)
    let t_1000: f64 = t_600 + (400.0 / 11.428); // ≈ 75.0 h (end of segment 2)

    let distance_km = if time_hours <= t_60 {
        // T = 1 + D/20  =>  D = (T - 1) * 20
        (time_hours - 1.0) * 20.0
    } else if time_hours <= t_600 {
        // T = D/15  =>  D = T * 15
        time_hours * 15.0
    } else if time_hours <= t_1000 {
        // T = 40 + (D - 600) / 11.428  =>  D = 600 + (T - 40) * 11.428
        600.0 + (time_hours - t_600) * 11.428
    } else {
        // T = t_1000 + (D - 1000) / 13.333  =>  D = 1000 + (T - t_1000) * 13.333
        // Clamped at 1300 km per your original logic
        (1000.0 + (time_hours - t_1000) * 13.333).min(1300.0)
    };

    distance_km * 1000.0
}

fn duration(distance: f64, speed: &Speed) -> TimeDelta {
    let seconds = match speed {
        Speed::ACP => duration_acp(distance),
        Speed::MPS(mps) => distance / mps,
    };
    TimeDelta::nanoseconds((1000_000_000f64 * seconds).round() as i64)
}

fn time(distance: f64, start_time: &DateTime, speed: &Speed) -> DateTime {
    let delta = duration(distance, &speed);
    *start_time + delta
}

#[derive(Clone, Debug, Default)]
pub struct ControlSpeedData {
    pub track_index: usize,
    pub distance: f64,
    pub time: Option<DateTime>,
    pub last_control: bool,
}

impl ControlSpeedData {
    fn virtual_acp_controls(start: &DateTime) -> Vec<ControlSpeedData> {
        let mut ret = Vec::new();
        for (km, hours) in acp_distance_time_intermediate() {
            ret.push(ControlSpeedData {
                track_index: 0,
                distance: km * 1000f64,
                time: Some(*start + TimeDelta::seconds((hours * 3600f64).round() as i64)),
                last_control: false,
            });
        }
        ret
    }
}

fn time_at_control(control: &ControlSpeedData, start_time: &DateTime, speed: &Speed) -> DateTime {
    if control.last_control {
        match speed {
            Speed::ACP => {
                let delta = duration_acp_last(control.distance).round() as i64 * 1_000_000_000;
                let duration = TimeDelta::nanoseconds(delta);
                return *start_time + duration;
            }
            _ => {}
        }
    }
    control
        .time
        .unwrap_or(time(control.distance, start_time, speed))
}

fn filter_control_speed_data(data: Vec<ControlSpeedData>) -> Vec<ControlSpeedData> {
    fn time(control: &ControlSpeedData) -> DateTime {
        control.time.as_ref().unwrap().clone()
    }
    let keep: Vec<bool> = data
        .iter()
        .enumerate()
        .map(|(index, pl)| {
            !data
                .iter()
                .any(|pk| (pk.distance > pl.distance && time(pk) < time(pl)))
                || index == 0 // keep START
                || index == data.len() - 1 // keep END
        })
        .collect();

    data.into_iter()
        .zip(keep)
        .filter_map(|(item, keep)| if keep { Some(item) } else { None })
        .collect()
}

fn setup_interpolation_controls(
    start_time: &DateTime,
    realcontrols: &Vec<ControlSpeedData>,
) -> Vec<ControlSpeedData> {
    let mut all_controls = realcontrols.clone();
    let maxdist = realcontrols
        .iter()
        .find(|c| c.last_control)
        .unwrap()
        .distance;
    all_controls.retain(|c| c.last_control || c.time.is_some());
    let mut virtual_controls = ControlSpeedData::virtual_acp_controls(start_time);
    if !realcontrols.is_empty() {
        // remove virtual controls if there is a real one by less than 15km.
        virtual_controls.retain(|c| {
            let closest = all_controls.iter().min_by(|a, b| {
                (a.distance - c.distance)
                    .abs()
                    .partial_cmp(&(b.distance - c.distance).abs())
                    .unwrap()
            });
            let ret = (closest.unwrap().distance - c.distance).abs() > 15_000f64;
            ret
        });
    }
    all_controls.extend_from_slice(&virtual_controls);
    all_controls.sort_by(|a, b| a.distance.total_cmp(&b.distance));
    all_controls.retain(|c| c.distance <= maxdist);
    all_controls.iter_mut().enumerate().for_each(|(index, c)| {
        log::trace!("{} [{:?}]", index, c);
        debug_assert!(c.time.is_some() || c.last_control);
        if c.time.is_none() {
            let duration = duration_acp_last(c.distance);
            c.time = Some(*start_time + TimeDelta::seconds((3600f64 * duration).round() as i64));
        }
    });
    let ret = filter_control_speed_data(all_controls);
    ret.iter().enumerate().for_each(|(index, c)| {
        log::trace!("ret {} [{:?}]", index, c);
    });
    ret
}

pub fn time_with_controls(
    realcontrols: &Vec<ControlSpeedData>,
    distance: f64,
    start_time: &DateTime,
    speed: &Speed,
) -> DateTime {
    let all_controls = match speed {
        Speed::ACP => setup_interpolation_controls(start_time, realcontrols),
        Speed::MPS(_) => realcontrols.clone(),
    };
    // controls has to be sorted by distance and time
    // controls must contains START and END.
    let maybe = all_controls
        .iter()
        .enumerate()
        .find(|(_, c)| c.distance >= distance);
    if maybe.is_none() {
        log::info!("could not find next control for distance={:.1}", distance);
        for c in all_controls {
            log::info!("control {:?}", c);
        }
        return time(distance, start_time, speed);
    }
    let (index_next, next) = maybe.unwrap();
    if index_next == 0 {
        return start_time.clone();
    }
    let previous = &all_controls[index_next - 1];

    let normal_previous_time = time(previous.distance, start_time, speed);
    let real_previous_time = time_at_control(previous, start_time, speed);
    let normal_next_time = time(next.distance, start_time, speed);
    let real_next_time = time_at_control(next, start_time, speed);

    let normal_time = time(distance, start_time, speed);
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

pub fn distance(duration: &TimeDelta, speed: &Speed) -> f64 {
    match speed {
        Speed::ACP => distance_acp(duration.as_seconds_f64()),
        Speed::MPS(mps) => duration.as_seconds_f64() * mps,
    }
}

pub fn distance_with_controls(
    realcontrols: &Vec<ControlSpeedData>,
    start_time: &DateTime,
    duration: &TimeDelta,
    speed: &Speed,
) -> f64 {
    let current_time = *start_time + *duration;
    let all_controls = match speed {
        Speed::ACP => setup_interpolation_controls(start_time, realcontrols),
        Speed::MPS(_) => realcontrols.clone(),
    };
    let maybe = all_controls
        .iter()
        .enumerate()
        .find(|(_, c)| time_at_control(c, start_time, speed) >= current_time);

    if maybe.is_none() {
        log::info!(
            "could not find next control for distance_with_controls at time: {:?}",
            current_time
        );
        return distance(duration, speed);
    }

    let (index_next, next) = maybe.unwrap();

    if index_next == 0 {
        return 0f64;
    }

    let previous = &all_controls[index_next - 1];

    let real_previous_time = time_at_control(previous, start_time, speed);
    let real_next_time = time_at_control(next, start_time, speed);

    let delta1 = current_time - real_previous_time;
    let delta2 = real_next_time - real_previous_time;

    // lambda: how far we are between previous and next control (in real time)
    let lambda = delta1.as_seconds_f64() / delta2.as_seconds_f64();

    debug_assert!((0.0..=1.0).contains(&lambda), "lambda={}", lambda);

    // Apply lambda to the normal (speed-model) distance span
    let normal_previous_dist = previous.distance;
    let normal_next_dist = next.distance;
    let ret = normal_previous_dist + lambda * (normal_next_dist - normal_previous_dist);
    ret
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
    debug_assert!(ok.is_some(), "data={}", data);
    let kmh: f64 = ok.unwrap();
    Speed::MPS(kmh * 1000.0 / 3600.0)
}

#[derive(Clone, Default)]
pub struct TimeParameters {
    pub controls: Vec<ControlSpeedData>,
    pub start: DateTime,
    pub speed: Speed,
}

impl TimeParameters {
    pub fn from_parameters(parameters: &Parameters) -> Self {
        Self {
            controls: Vec::new(),
            start: parameters::parse_time(&parameters.start_time),
            speed: parse_speed(&parameters.speed),
        }
    }
    pub fn time_at_control_speed_data(&self, data: &ControlSpeedData) -> DateTime {
        match data.time {
            Some(t) => t.clone(),
            None => self.time(data.distance),
        }
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
            _ => time_with_controls(&self.controls, distance, &self.start, &self.speed),
        }
    }
    pub fn time(&self, distance: f64) -> DateTime {
        time_with_controls(&self.controls, distance, &self.start, &self.speed)
    }
    pub fn distance(&self, duration: &TimeDelta) -> f64 {
        distance_with_controls(&self.controls, &self.start, duration, &self.speed)
    }
    pub fn duration(&self, distance_a: f64, distance_b: f64) -> TimeDelta {
        debug_assert!(distance_a <= distance_b);
        let ta = time_with_controls(&self.controls, distance_a, &self.start, &self.speed);
        let tb = time_with_controls(&self.controls, distance_b, &self.start, &self.speed);
        debug_assert!(ta <= tb);
        tb - ta
    }
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
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("{}", 15.0);
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 300 km at 15 km/h should take 20 hours
        let dist_300km = 300_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_300 = speed::time(dist_300km, &start, &speed);
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
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // Short distance (< 60 km): T = 1 + (D / 20)
        // 40 km should take: 1 + (40/20) = 3 hours
        let dist_40km = 40_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_40 = time(dist_40km, &start, &speed);
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
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 300 km at 15 km/h should take 20 hours
        let dist_300km = 300_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_300 = time(dist_300km, &start, &speed);
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
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 600 km: hard cap should be 40 hours
        let dist_600km = 600_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_600 = time(dist_600km, &start, &speed);
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
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);
        let speed = speed::parse_speed(&params.speed);
        // 800 km: 600/15 + (800-600)/11.428
        //       = 40 + 200/11.428 = 40 + 17.5 = 57.5 hours
        let dist_800km = 800_000.0;
        let time_800 = time(dist_800km, &start, &speed);
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
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 1000 km: hard cap should be 75 hours
        // Calculated: 600/15 + 400/11.428 = 40 + 35 = 75 hours
        let dist_1000km = 1_000_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_1000 = time(dist_1000km, &start, &speed);
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
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 1200 km: 600/15 + 400/11.428 + 200/13.333
        //        = 40 + 35 + 15 = 90 hours
        let dist_1200km = 1_200_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time_1200 = time(dist_1200km, &start, &speed);
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
    fn test_acp_above_2000km() {
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);

        // 2400 km at 10kmh => 240 hours => 10 days
        let dist = 2_400_000.0;
        let speed = speed::parse_speed(&params.speed);
        let time = time(dist, &start, &speed);
        let duration_sec = (time - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 240.0;

        assert!(
            (duration_hours - expected).abs() < 0.00001,
            "Expected ~{} hours for 2400km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_standard_brevets() {
        let _ = env_logger::try_init();
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
            let time = time(distance, &start, &speed);
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

    #[test]
    fn test_acp_revert() {
        let _ = env_logger::try_init();
        let start = ControlSpeedData {
            track_index: 0,
            distance: 0f64,
            time: None,
            last_control: false,
        };
        let end = ControlSpeedData {
            track_index: 10000,
            distance: 400_000f64,
            time: None,
            last_control: true,
        };
        let controls = vec![start, end];
        let distance = 20_000f64;
        let start_time = parameters::parse_time(&"2026-04-29T00:00:00");
        let expected_time = parameters::parse_time(&"2026-04-29T02:00:00");
        let time = time_with_controls(&controls, distance, &start_time, &Speed::ACP);
        assert_eq!(time, expected_time);
        let duration = time - start_time;
        let d = distance_with_controls(&controls, &start_time, &duration, &Speed::ACP);
        assert_eq!(distance, d);
    }
}
