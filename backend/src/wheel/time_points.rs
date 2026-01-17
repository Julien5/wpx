use std::time::Duration;

fn nice_interval(total_duration: Duration, n: usize) -> Duration {
    let total_seconds = total_duration.as_secs_f64();

    // Define nice intervals in seconds
    const MINUTE: f64 = 60.0;
    const HOUR: f64 = 3600.0;
    const DAY: f64 = 86400.0;

    let nice_intervals = [
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

    let target_interval = total_seconds / n as f64;

    // Find the closest nice interval
    let mut best_interval = nice_intervals[0];
    let mut best_diff = (target_interval - best_interval).abs();

    for &interval in &nice_intervals {
        let diff = (target_interval - interval).abs();
        if diff < best_diff {
            best_diff = diff;
            best_interval = interval;
        }
    }

    Duration::from_secs_f64(best_interval)
}

use chrono::{Duration as ChronoDuration, Timelike};
use mercator::DateTime;

use crate::{
    mercator,
    wheel::model::{angle, CirclePoint, TimeParameters},
};

fn generate_time_intervals(
    start_time: DateTime,
    duration: Duration,
    interval: Duration,
) -> Vec<DateTime> {
    let mut times = Vec::new();

    // Add start time
    times.push(start_time);

    // Calculate end time
    let end_time = start_time + ChronoDuration::from_std(duration).unwrap();

    // Get midnight of the start day
    let midnight = start_time
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    let interval_chrono = ChronoDuration::from_std(interval).unwrap();

    // Find the first k such that midnight + k*interval > start_time
    let mut k = 0;
    loop {
        let candidate = midnight + interval_chrono * k;

        if candidate > start_time && candidate < end_time {
            times.push(candidate);
        }

        // Stop if we've passed the end time
        if candidate >= end_time {
            break;
        }

        k += 1;
    }

    // Add end time
    times.push(end_time);

    times
}

fn format_time(time: &DateTime, force: bool) -> String {
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

fn make(times: &Vec<DateTime>, start_time: &DateTime, duration_seconds: f64) -> Vec<CirclePoint> {
    let mut ret = Vec::new();
    let a_start = angle(0.0, duration_seconds);
    let a_end = 360.0 - super::constants::ARCANGLE / 2.0;
    for (index, time) in times.iter().enumerate() {
        let force = index == 0 || index == times.len() - 1;
        let x = time
            .signed_duration_since(start_time)
            .as_seconds_f64()
            .floor();
        let a = angle(x, duration_seconds);
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

pub fn generate(time_parameters: &TimeParameters) -> Vec<CirclePoint> {
    let duration_seconds = time_parameters.duration_seconds();
    let start_time: DateTime = time_parameters.start;
    let duration = std::time::Duration::from_secs_f64(duration_seconds);
    let interval = nice_interval(duration, 12);
    let times = generate_time_intervals(start_time, duration, interval);
    make(&times, &start_time, duration_seconds)
}
