pub mod spec;
mod test;

use chrono::TimeDelta;
use std::cmp::Ordering;

use crate::mercator::DateTime;

#[derive(Clone, Debug, Default)]
pub struct InterpolationPoint {
    pub distance: f64,
    //pub _time: Option<DateTime>,
    pub duration: Option<TimeDelta>,
    pub is_end: bool,
}

impl InterpolationPoint {
    pub fn unwrap_duration(&self) -> TimeDelta {
        self.duration.unwrap().clone()
    }
    pub fn unwrap_time(&self, start_time: &DateTime) -> DateTime {
        *start_time + self.unwrap_duration()
    }
}

impl PartialOrd for InterpolationPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InterpolationPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare by distance first (NaN values are sorted last)
        let dist_order = self
            .distance
            .partial_cmp(&other.distance)
            .unwrap_or_else(|| match (self.distance.is_nan(), other.distance.is_nan()) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => Ordering::Equal,
            });

        if dist_order != Ordering::Equal {
            return dist_order;
        }

        // Distances are equal — break tie by time (None sorts last)
        match (&self.duration, &other.duration) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl PartialEq for InterpolationPoint {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for InterpolationPoint {}

fn find_interval_at_distance(
    interpolation_points: &Vec<InterpolationPoint>,
    distance: f64,
) -> (&InterpolationPoint, &InterpolationPoint) {
    debug_assert!(interpolation_points.len() >= 2);
    debug_assert!(interpolation_points.is_sorted());

    let lo = interpolation_points.partition_point(|p| p.distance < distance);
    let hi = interpolation_points.partition_point(|p| p.distance <= distance);

    if lo < hi {
        return (&interpolation_points[lo], &interpolation_points[hi - 1]);
    }

    let next = match interpolation_points.get(lo) {
        Some(p) => p,
        None => interpolation_points.last().unwrap(),
    };

    let prev = match lo.checked_sub(1).and_then(|i| interpolation_points.get(i)) {
        Some(p) => p,
        None => next,
    };

    (prev, next)
}

fn interpolate_duration(
    interpolation_points: &Vec<InterpolationPoint>,
    distance: f64,
) -> TimeDelta {
    // controls must contains START and END.
    debug_assert!(interpolation_points.len() >= 2);
    // controls has to be sorted by distance and time
    debug_assert!(interpolation_points.is_sorted());
    let (previous, next) = find_interval_at_distance(interpolation_points, distance);
    let (t1, d1) = (previous.unwrap_duration(), previous.distance);
    let (t2, d2) = (next.unwrap_duration(), next.distance);

    // log::trace!("[TP ] distance:{:?}", distance); //
    // log::trace!("[TP1] previous:{:?}", previous); //
    // log::trace!("[TP1] 1:{:?},{:.1}", t1, d1); //
    // log::trace!("[TP1] next:{:?}", next); //
    // log::trace!("[TP1] 2:{:?},{:.1}", t2, d2); //

    // This is the "somwhat degenerate" case mentioned above.
    if d1 == d2 {
        debug_assert!(d1 == distance, "{}", &format!("{} / {}", d1, distance));
        // the *cutoff* corresponds to the maximum time for a given distance.
        // => t2.
        return t2;
    }
    debug_assert!(d1 < d2);
    debug_assert!(d1 <= distance && distance <= d2 || d2 < distance);
    let fraction = (distance - d1) / (d2 - d1);
    let span_ns = (t2 - t1)
        .num_nanoseconds()
        .expect("time span overflows i64 nanoseconds");
    let offset_ns = (fraction * span_ns as f64).round() as i64;
    let ret = t1 + TimeDelta::nanoseconds(offset_ns);
    //////////////////////////////////////////////////////////////////
    // log::info!("[TP1] distance:{:.1} time:{:?}", distance, ret); //
    //////////////////////////////////////////////////////////////////
    ret
}

fn find_interval_at_time<'a>(
    interpolation_points: &'a Vec<InterpolationPoint>,
    duration: &TimeDelta,
) -> (&'a InterpolationPoint, &'a InterpolationPoint) {
    let next_candidate = interpolation_points
        .iter()
        .find(|c| c.unwrap_duration() >= *duration);

    let next = match next_candidate {
        Some(point) => point,
        None => interpolation_points.iter().last().unwrap(),
    };

    if next.distance == 0f64 {
        return (
            interpolation_points.first().unwrap(),
            interpolation_points.first().unwrap(),
        );
    }

    let previous_candidate = interpolation_points
        .iter()
        .filter(|control| control.unwrap_duration() < next.duration.unwrap())
        .last();
    let previous = match previous_candidate {
        Some(point) => point,
        None => interpolation_points.first().unwrap(),
    };
    (previous, next)
}

pub fn interpolate_distance(
    interpolation_points: &Vec<InterpolationPoint>,
    start_time: &DateTime,
    duration: &TimeDelta,
) -> f64 {
    debug_assert!(
        interpolation_points.len() > 1,
        "len={}",
        interpolation_points.len()
    );
    let (previous, next) = find_interval_at_time(interpolation_points, duration);
    let (t1, d1) = (previous.unwrap_time(start_time), previous.distance);
    let (t2, d2) = (next.unwrap_time(start_time), next.distance);
    debug_assert!(t1 <= t2);
    if t1 == t2 {
        return d2;
    }
    let span_ns = (t2 - t1)
        .num_nanoseconds()
        .expect("time span overflows i64 nanoseconds");
    let time = *start_time + *duration;
    debug_assert!(interpolation_points.is_sorted());
    debug_assert!(interpolation_points.is_sorted_by_key(|a| a.unwrap_duration()));
    debug_assert!(
        t1 <= time && time <= t2 || time > t2,
        "t1={} time={} t2={} | len={}",
        t1,
        time,
        t2,
        interpolation_points.len()
    );

    let offset_ns = (time - t1)
        .num_nanoseconds()
        .expect("time offset overflows i64 nanoseconds");

    let fraction = offset_ns as f64 / span_ns as f64;
    debug_assert!(fraction >= 0f64);
    let ret = d1 + fraction * (d2 - d1);
    /*log::trace!(
        "[TP2] distance:{:.1} time:{:?}",
        ret / 1000f64,
        current_time
    );*/
    ret
}

#[derive(Clone, Debug)]
pub struct ACPSpec {
    pub km: f64,    // km
    pub hours: f64, // hours
}

#[derive(Clone, Debug)]
pub struct LRMSpec {
    pub kmh: f64, // kmh
}

#[derive(Clone, Debug)]
pub struct KMHSpec {
    pub kmh: f64, // kmh
}

#[derive(Clone, Debug)]
pub enum Speed {
    KMH(KMHSpec),
    ACP(ACPSpec),
    LRM(LRMSpec),
}

impl Default for Speed {
    fn default() -> Self {
        Speed::KMH(KMHSpec { kmh: 15.0 })
    }
}

pub fn parse_speed(data: &str) -> Speed {
    if data.contains("ACP") {
        let parts: Vec<&str> = data.split('-').collect();
        if parts.len() != 3 {
            log::trace!("parts:{:?}", parts);
        }
        debug_assert!(parts.len() == 3);
        let distance: f64 = parts[1].parse().expect("Failed to parse distance");
        let time: f64 = parts[2].parse().expect("Failed to parse hours");
        return Speed::ACP(ACPSpec {
            km: distance,
            hours: time,
        });
    }
    if data.contains("LRM") {
        let parts: Vec<&str> = data.split('-').collect();
        if parts.len() != 2 {
            log::trace!("parts:{:?}", parts);
        }
        debug_assert!(parts.len() == 2);
        let kmh: f64 = parts[1]
            .parse()
            .expect(&format!("Failed to parse LRM kmh: {}", data));
        return Speed::LRM(LRMSpec { kmh });
    }
    if data.contains("KMH") {
        let parts: Vec<&str> = data.split('-').collect();
        if parts.len() != 2 {
            log::trace!("parts:{:?}", parts);
        }
        debug_assert!(parts.len() == 2);
        let kmh: f64 = parts[1]
            .parse()
            .expect(&format!("Failed to parse KMH kmh: {}", data));
        return Speed::KMH(KMHSpec { kmh });
    }
    panic!("invalid speed string {}", data)
}

pub fn format_kmh(kmh: f64) -> String {
    format!("KMH-{:.3}", kmh)
}

pub fn allowed_speeds(end_distance: f64) -> Vec<String> {
    let mut ret = Vec::new();
    ret.push(format!("KMH-*"));
    if let Some(spec) = spec::ACP::guess_spec(end_distance) {
        ret.push(format!("ACP-{:.0}-{:.1}", spec.km, spec.hours));
    }
    if let Some(spec) = spec::LRM::guess_spec(end_distance) {
        // five digits to cover 13.33...
        ret.push(format!("LRM-{:.2}", spec.kmh));
    }
    ret
}

#[derive(Clone, Default)]
pub struct TimeParameters {
    pub controls: Vec<InterpolationPoint>,
    pub start: DateTime,
    pub speed: Speed,
    pub track_distance: f64,
    pub power: Option<Vec<InterpolationPoint>>,
}

impl TimeParameters {
    pub fn control_interpolation_points(&self) -> Vec<InterpolationPoint> {
        let ret = match &self.speed {
            Speed::ACP(spec) => {
                spec::ACP::interpolation_controls(self.track_distance, &self.controls, &spec)
            }
            Speed::LRM(spec) => spec::LRM::interpolation_points(self.track_distance, &spec),
            Speed::KMH(kmh) => {
                let mps = kmh.kmh * 1000.0 / 3600.0;
                spec::MPS::interpolation_points(&self.controls, self.track_distance, mps)
            }
        };
        debug_assert!(ret.is_sorted());
        debug_assert!(ret.len() >= 2);
        ret.iter().for_each(|c| debug_assert!(c.duration.is_some()));
        ret
    }

    pub fn time(&self, distance: f64) -> DateTime {
        let interpolation_points = match &self.power {
            Some(points) => &points,
            None => &self.control_interpolation_points(),
        };
        let duration = interpolate_duration(interpolation_points, distance);
        self.start + duration
    }

    pub fn distance(&self, duration: &TimeDelta) -> f64 {
        let interpolation_points = match &self.power {
            Some(points) => &points,
            None => &self.control_interpolation_points(),
        };
        interpolate_distance(interpolation_points, &self.start, duration)
    }
}
