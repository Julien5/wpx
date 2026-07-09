pub mod spec;
mod test;

use chrono::TimeDelta;
use std::cmp::Ordering;

use crate::{mercator::DateTime, power::PowerParameters, track::Geometry};

#[derive(Clone, Debug, Default)]
pub struct InterpolationPoint {
    pub distance: f64,
    pub time: Option<DateTime>,
    pub is_end: bool,
}

impl InterpolationPoint {
    pub fn unwrap_time(&self) -> DateTime {
        self.time.unwrap().clone()
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
        match (&self.time, &other.time) {
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
    // controls must contains START and END.
    debug_assert!(interpolation_points.len() >= 2);
    // controls has to be sorted by distance and time
    debug_assert!(interpolation_points.is_sorted());

    /* There is a somewhat degenerate case in the case of ACP at
     * start. The interpolations points have two elements at d==0:
     * 0h and 1h.
     */
    let equals: Vec<_> = interpolation_points
        .iter()
        .filter(|c| c.distance == distance)
        .collect();

    if equals.len() >= 2 {
        return (equals.first().unwrap(), equals.last().unwrap());
    }

    let next = {
        let larger = interpolation_points.iter().find(|c| c.distance >= distance);
        match larger {
            Some(point) => point,
            None => interpolation_points.last().unwrap(),
        }
    };

    ////////////////////////////////////////////////////////////////////
    // log::trace!("next: {:?}", next);								  //
    // for c in interpolation_points {								  //
    //     log::trace!("[TP1] inter:{:?},{:.1}", c.time, c.distance); //
    // }															  //
    ////////////////////////////////////////////////////////////////////
    if next.distance == 0f64 {
        return (
            interpolation_points.first().unwrap(),
            interpolation_points.first().unwrap(),
        );
    }
    let previous_candidate = interpolation_points
        .iter()
        .filter(|control| control.distance < next.distance)
        .last();

    let previous = match previous_candidate {
        Some(point) => point,
        None => interpolation_points.first().unwrap(),
    };

    (previous, next)
}

fn interpolate_time(interpolation_points: &Vec<InterpolationPoint>, distance: f64) -> DateTime {
    // controls must contains START and END.
    debug_assert!(interpolation_points.len() >= 2);
    // controls has to be sorted by distance and time
    debug_assert!(interpolation_points.is_sorted());
    let (previous, next) = find_interval_at_distance(interpolation_points, distance);
    let (t1, d1) = (previous.unwrap_time(), previous.distance);
    let (t2, d2) = (next.unwrap_time(), next.distance);

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
    time: &DateTime,
) -> (&'a InterpolationPoint, &'a InterpolationPoint) {
    let next_candidate = interpolation_points
        .iter()
        .find(|c| c.unwrap_time() >= *time);

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
        .filter(|control| control.unwrap_time() < next.time.unwrap())
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
    let time = *start_time + *duration;
    let (previous, next) = find_interval_at_time(interpolation_points, &time);
    let (t1, d1) = (previous.unwrap_time(), previous.distance);
    let (t2, d2) = (next.unwrap_time(), next.distance);
    debug_assert!(t1 <= t2);
    if t1 == t2 {
        return d2;
    }
    let span_ns = (t2 - t1)
        .num_nanoseconds()
        .expect("time span overflows i64 nanoseconds");
    // log::trace!("t1={} time={} t2={}", t1, time, t2);
    debug_assert!(t1 <= time && time <= t2 || time > t2);

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

#[derive(Clone)]
pub struct ConstantPowerGeometry {
    geometry: Geometry,
    power_params: PowerParameters,
    points: Option<Vec<InterpolationPoint>>,
}

impl ConstantPowerGeometry {
    pub fn new(geometry: &Geometry) -> Self {
        Self {
            geometry: geometry.clone(),
            power_params: PowerParameters::default(),
            points: None,
        }
    }

    pub fn with_power_params(mut self, params: PowerParameters) -> Self {
        self.power_params = params;
        self
    }

    /// Computes interior interpolation points for one control interval.
    /// Returns points for geometry indices strictly between `prev` and `next`,
    /// so the caller can stitch intervals together without duplicates.
    fn solve_interval(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> Vec<InterpolationPoint> {
        let start = self.geometry.index_after(prev.distance);
        let end = self.geometry.index_before(next.distance) + 1;
        if start >= end {
            return Vec::new();
        }

        let duration_ns = (next.unwrap_time() - prev.unwrap_time())
            .num_nanoseconds()
            .unwrap() as f64;
        let duration_secs = duration_ns / 1_000_000_000.0;

        let power = self.power_params.power_at_duration(
            duration_secs,
            |i| self.geometry.distance(i),
            |i| self.geometry.elevation(i),
            start,
            end,
        );
        log::trace!("{} -> {}, power = {}", start, end, power);
        let mut points = Vec::new();
        let mut cum_time = 0.0;
        let start_time = prev.unwrap_time();

        self.power_params.for_each_segment(
            power,
            &|i| self.geometry.distance(i),
            &|i| self.geometry.elevation(i),
            start,
            end,
            |i, seg_time| {
                cum_time += seg_time;
                let new = InterpolationPoint {
                    distance: self.geometry.distance(i + 1),
                    time: Some(
                        start_time
                            + TimeDelta::nanoseconds((cum_time * 1_000_000_000.0).round() as i64),
                    ),
                    is_end: false,
                };
                if points.last().is_some() {
                    let last: &InterpolationPoint = points.last().unwrap();
                    debug_assert!(*last < new);
                }
                points.push(new);
            },
        );

        points
    }

    /// Builds a dense set of interpolation points from sparse control points.
    /// Each adjacent pair `(prev, next)` defines a time window; the constant
    /// power needed to cover that distance in that time is computed, and the
    /// segment-by-segment cumulative times become the new points.
    pub fn solve(&mut self, controls: &Vec<InterpolationPoint>) {
        log::warn!(
            "solve called\n{:?}",
            std::backtrace::Backtrace::force_capture()
        );
        let mut all_points = Vec::new();

        for window in controls.windows(2) {
            let prev = &window[0];
            let next = &window[1];

            if all_points.is_empty() {
                all_points.push(InterpolationPoint {
                    distance: prev.distance,
                    time: Some(prev.unwrap_time()),
                    is_end: false,
                });
            }

            all_points.extend(self.solve_interval(prev, next));
            debug_assert!(all_points.is_sorted());
        }

        if let Some(last) = controls.last() {
            all_points.push(InterpolationPoint {
                distance: last.distance,
                time: Some(last.unwrap_time()),
                is_end: true,
            });
            all_points.retain(|p| p <= last);
        }
        debug_assert!(all_points.is_sorted());
        self.points = Some(all_points);
    }
}

#[derive(Clone, Default)]
pub struct TimeParameters {
    pub controls: Vec<InterpolationPoint>,
    pub start: DateTime,
    pub speed: Speed,
    pub track_distance: f64,
    pub geometry: Option<std::sync::Arc<ConstantPowerGeometry>>,
}

impl TimeParameters {
    pub fn control_interpolation_points(&self) -> Vec<InterpolationPoint> {
        let ret = match &self.speed {
            Speed::ACP(spec) => spec::ACP::interpolation_controls(
                &self.start,
                self.track_distance,
                &self.controls,
                &spec,
            ),
            Speed::LRM(spec) => {
                spec::LRM::interpolation_points(self.track_distance, &self.start, &spec)
            }
            Speed::KMH(kmh) => {
                let mps = kmh.kmh * 1000.0 / 3600.0;
                spec::MPS::interpolation_points(
                    &self.controls,
                    self.track_distance,
                    &self.start,
                    mps,
                )
            }
        };
        debug_assert!(ret.is_sorted());
        debug_assert!(ret.len() >= 2);
        ret.iter().for_each(|c| debug_assert!(c.time.is_some()));
        ret
    }

    pub fn time(&self, distance: f64) -> DateTime {
        match &self.geometry {
            Some(g) => match &g.points {
                Some(interpolation_points) => {
                    return interpolate_time(&interpolation_points, distance)
                }
                None => {}
            },
            None => {}
        }
        interpolate_time(&self.control_interpolation_points(), distance)
    }

    pub fn distance(&self, duration: &TimeDelta) -> f64 {
        match &self.geometry {
            Some(g) => match &g.points {
                Some(interpolation_points) => {
                    return interpolate_distance(&interpolation_points, &self.start, duration);
                }
                None => {}
            },
            None => {}
        }
        interpolate_distance(&self.control_interpolation_points(), &self.start, duration)
    }

    pub fn duration(&self, distance_a: f64, distance_b: f64) -> TimeDelta {
        debug_assert!(distance_a <= distance_b);
        let ta = interpolate_time(&self.control_interpolation_points(), distance_a);
        let tb = interpolate_time(&self.control_interpolation_points(), distance_b);
        debug_assert!(ta <= tb);
        tb - ta
    }
}
