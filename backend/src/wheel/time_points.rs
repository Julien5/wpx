fn nice_interval(duration: &TimeDelta, n: usize) -> chrono::TimeDelta {
    // Define nice intervals in seconds
    const MINUTE: f64 = 60.0;
    const HOUR: f64 = 3600.0;
    const DAY: f64 = 86400.0;

    let nice_intervals_seconds = [
        1.0 * MINUTE,  // 1 minute
        2.0 * MINUTE,  // 2 minutes
        5.0 * MINUTE,  // 5 minutes
        10.0 * MINUTE, // 10 minutes
        15.0 * MINUTE, // 15 minutes
        30.0 * MINUTE, // 30 minutes
        1.0 * HOUR,    // 1 hour
        2.0 * HOUR,    // 2 hours
        3.0 * HOUR,    // 3 hours
        6.0 * HOUR,    // 6 hours
        12.0 * HOUR,   // 12 hours
        1.0 * DAY,     // 1 day
        2.0 * DAY,     // 2 days
        3.0 * DAY,     // 3 days
        7.0 * DAY,     // 1 week
    ];

    let target_interval_seconds = duration.as_seconds_f64() / n as f64;

    // Find the closest nice interval
    let mut best_interval_seconds = nice_intervals_seconds[0];
    let mut best_diff = (target_interval_seconds - best_interval_seconds).abs();

    for &interval_seconds in &nice_intervals_seconds {
        let diff = (target_interval_seconds - interval_seconds).abs();
        if diff < best_diff {
            best_diff = diff;
            best_interval_seconds = interval_seconds;
        }
    }

    TimeDelta::milliseconds((1000f64 * best_interval_seconds).round() as i64)
}

use std::collections::BTreeSet;

use chrono::{TimeDelta, Timelike};
use mercator::DateTime;

use crate::{
    mercator,
    speed::TimeParameters,
    wheel::model::{angle, CirclePoint},
};

pub fn format_time(time: &DateTime, force: bool) -> String {
    // per default, to not make text from "12:30"
    // unless force=true.
    if force {
        return time.format("%k:%M").to_string();
    }
    if time.hour() == 0 && time.minute() == 0 && time.second() == 0 {
        time.format("%a").to_string()
    } else if time.minute() == 0 && time.second() == 0 {
        // Same as %H but space-padded. Same as %_H.
        time.format("%k").to_string()
    } else {
        String::new()
    }
}

fn make(time_parameters: &TimeParameters, times: &Vec<DateTime>) -> Vec<CirclePoint> {
    let mut ret = Vec::new();
    let a_start = angle(0.0, time_parameters.track_distance);
    let a_end = 360.0 - super::constants::ARCANGLE / 2.0;
    for (index, time) in times.iter().enumerate() {
        let force = index == 0 || index == times.len() - 1;

        let x = time_parameters.distance(&(*time - time_parameters.start));
        let a = angle(x, time_parameters.track_distance);
        let margin = 10.0;
        // this condition is needed if we include the start time (or the end time)
        // to ensure no label overlap
        if index > 0 && (a - a_start).abs() < margin {
            continue;
        }
        if 0 < index && index < times.len() - 1 && (a - a_end).abs() < margin {
            continue;
        }
        let c = CirclePoint {
            angle: a,
            name: format_time(time, force),
        };
        ret.push(c);
    }
    ret
}

pub fn generate_circle_points(time_parameters: &TimeParameters) -> Vec<CirclePoint> {
    let duration = time_parameters.time(time_parameters.track_distance) - time_parameters.start;
    let duration_seconds = duration.as_seconds_f64();
    let times = generate_times(time_parameters, 0f64, time_parameters.track_distance, 12);
    make(&time_parameters, &times)
}

fn datetime_from_nanos(nanos: i64) -> DateTime {
    use chrono::TimeZone;
    chrono::Local
        .timestamp_opt(nanos / 1_000_000_000, (nanos % 1_000_000_000) as u32)
        .unwrap()
}

pub struct SnapResult {
    pub floor: DateTime,
    pub ceil: DateTime,
}

fn snap(time: &DateTime, duration: &TimeDelta) -> SnapResult {
    let duration_nanos = duration
        .num_nanoseconds()
        .expect("duration too large to represent in nanoseconds");

    let timestamp_nanos = time
        .timestamp_nanos_opt()
        .expect("datetime out of representable range");

    let floor_nanos = timestamp_nanos.div_euclid(duration_nanos) * duration_nanos;
    let ceil_nanos = floor_nanos
        + if timestamp_nanos == floor_nanos {
            0
        } else {
            duration_nanos
        };

    let floor = datetime_from_nanos(floor_nanos);
    let ceil = datetime_from_nanos(ceil_nanos);

    debug_assert!(floor <= ceil);
    debug_assert!((*time - floor).abs().num_milliseconds() <= duration.num_milliseconds());
    debug_assert!((*time - ceil).abs().num_milliseconds() <= duration.num_milliseconds());
    SnapResult { floor, ceil }
}

pub fn generate_times(
    time_parameters: &TimeParameters,
    start: f64,
    end: f64,
    n: usize,
) -> Vec<DateTime> {
    let mut points = BTreeSet::new();
    let interval_points = time_parameters.interpolation_points();
    for window in interval_points.windows(2) {
        let (prev, next) = (&window[0], &window[1]);
        let tprev = prev.unwrap_time();
        if next.distance < start {
            continue;
        }
        if prev.distance > end {
            continue;
        }
        let tnext = next.unwrap_time();
        let interval_duration = tnext - tprev;
        let interval_distance = next.distance - prev.distance;
        let interval_n =
            (n as f64 * interval_distance / time_parameters.track_distance).ceil() as usize;
        let interval_delta = nice_interval(&interval_duration, interval_n);
        debug_assert!(interval_delta.num_seconds() > 0);
        let mut t = snap(&prev.unwrap_time(), &interval_delta).floor;
        loop {
            log::debug!("t={} interval={}", t, interval_delta);
            if t > next.unwrap_time() {
                break;
            }
            let d = time_parameters.distance(&(t - time_parameters.start));
            if start <= d && d <= end {
                points.insert(t);
            }
            t += interval_delta;
        }
    }
    points.iter().cloned().collect()
}
