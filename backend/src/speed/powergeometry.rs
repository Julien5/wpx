#![allow(dead_code)]
use chrono::TimeDelta;

use crate::{
    speed::{power::PowerParameters, InterpolationPoint},
    track::Geometry,
};

#[derive(Clone)]
pub struct ConstantPowerGeometry {
    geometry: Geometry,
    power_params: PowerParameters,
}

impl ConstantPowerGeometry {
    pub fn new(geometry: &Geometry) -> Self {
        Self {
            geometry: geometry.clone(),
            power_params: PowerParameters::default(),
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

        let duration_ns = (next.unwrap_duration() - prev.unwrap_duration())
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
        let mut cum_time = 0.0f64;
        let duration = prev.duration.unwrap()
            + TimeDelta::nanoseconds((cum_time * 1_000_000_000.0).round() as i64);

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
                    duration: Some(duration),
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
    pub fn interpolation_points(
        &self,
        controls: &Vec<InterpolationPoint>,
    ) -> Vec<InterpolationPoint> {
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
                    duration: prev.duration.clone(),
                    is_end: false,
                });
            }

            all_points.extend(self.solve_interval(prev, next));
            debug_assert!(all_points.is_sorted());
        }

        if let Some(last) = controls.last() {
            all_points.push(InterpolationPoint {
                distance: last.distance,
                duration: last.duration.clone(),
                is_end: true,
            });
            all_points.retain(|p| p <= last);
        }
        debug_assert!(all_points.is_sorted());
        all_points
    }
}
