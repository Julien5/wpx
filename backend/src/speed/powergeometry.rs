#![allow(dead_code)]
use std::collections::BTreeMap;

use chrono::TimeDelta;

use crate::{
    speed::{power::PowerParameters, InterpolationPoint},
    track::Geometry,
};

pub type Table = BTreeMap<i32, Vec<TimeDelta>>;

#[derive(Clone)]
pub struct ConstantPowerGeometry {
    geometry: Geometry,
    power_params: PowerParameters,
    table: Table,
}

impl ConstantPowerGeometry {
    pub fn compute_table(power_params: &PowerParameters, geometry: &Geometry) -> Table {
        let mut table = Table::new();
        debug_assert!(geometry.len() > 0);
        for power in (10..=1000).step_by(10) {
            let mut durations = vec![TimeDelta::seconds(0); geometry.len()];

            power_params.for_each_segment(
                power as f64,
                &|i| geometry.distance(i),
                &|i| geometry.elevation(i),
                0,
                geometry.len() - 1,
                |i, duration| {
                    durations[i] = if i > 0 {
                        durations[i - 1] + duration
                    } else {
                        duration
                    };
                },
            );
            table.insert(power, durations);
        }
        table
    }
    pub fn new(geometry: &Geometry) -> Self {
        let params = PowerParameters::default();
        let table = Self::compute_table(&params, geometry);
        Self {
            geometry: geometry.clone(),
            power_params: params,
            table,
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

        let duration = next.unwrap_duration() - prev.unwrap_duration();

        let power = self.power_params.power_at_duration(
            &duration,
            |i| self.geometry.distance(i),
            |i| self.geometry.elevation(i),
            start,
            end.min(self.geometry.len() - 1),
        );
        log::trace!("{} -> {}, power = {}", start, end, power);
        let mut points = Vec::new();
        let mut cumulative_duration = TimeDelta::seconds(0);

        self.power_params.for_each_segment(
            power,
            &|i| self.geometry.distance(i),
            &|i| self.geometry.elevation(i),
            start,
            end.min(self.geometry.len() - 1),
            |i, duration| {
                cumulative_duration += duration;
                let new = InterpolationPoint {
                    distance: self.geometry.distance(i),
                    duration: Some(cumulative_duration),
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
        // std::backtrace::Backtrace::force_capture()
        log::warn!("solve called",);
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
