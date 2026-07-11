#![allow(dead_code)]
use core::f64;
use std::collections::{BTreeMap, BTreeSet};

use chrono::TimeDelta;
use geo::SimplifyIdx;

use crate::{
    elevation, geometry::power::PowerParameters, inputpoint::InputPoint, speed::InterpolationPoint,
};

pub type Table = BTreeMap<i32, Vec<TimeDelta>>;

#[derive(Clone, PartialEq, Eq)]
pub enum SolverMethod {
    Linear,
    Newton,
    Bisection,
}

#[derive(Clone)]
pub struct ConstantPowerGeometry {
    simplified_indices: Vec<usize>,
    simplified_distances: Vec<f64>,
    simplified_elevations: Vec<f64>,
    power_params: PowerParameters,
    table: Table,
    pub interpolation: Option<Vec<InterpolationPoint>>,
}

impl ConstantPowerGeometry {
    pub fn new(distances: &[f64], elevations: &[f64], waypoints: &[InputPoint]) -> Self {
        let smooth_elevation = elevation::smooth(
            200f64,
            distances.len(),
            &|i: usize| -> f64 { distances[i] },
            &|i: usize| -> f64 { elevations[i] },
        );

        let simplified_indices: Vec<usize> = {
            let coords: Vec<geo::Coord> = smooth_elevation
                .iter()
                .enumerate()
                .map(|(idx, elevation)| geo::coord!(x: distances[idx], y: *elevation))
                .collect();
            let line = geo::LineString::new(coords);
            let epsilon = 2f64;
            let mut dp_indices = line.simplify_idx(epsilon);
            let waypoint_indices = {
                let projections = InputPoint::flatten_projections(&waypoints);
                let indices: Vec<_> = projections
                    .iter()
                    .map(|proj| proj.1.track_index)
                    .collect::<BTreeSet<_>>()
                    .iter()
                    .cloned()
                    .collect();
                indices
            };
            dp_indices.extend_from_slice(&waypoint_indices);
            dp_indices.sort();
            dp_indices
        };

        let simplified_distances: Vec<_> =
            simplified_indices.iter().map(|i| distances[*i]).collect();

        let simplified_elevations: Vec<_> = simplified_indices
            .iter()
            .map(|i| smooth_elevation[*i])
            .collect();

        let power_params = PowerParameters::default();
        let table =
            Self::compute_table(&power_params, &simplified_distances, &simplified_elevations);

        Self {
            simplified_indices,
            simplified_distances,
            simplified_elevations,
            power_params,
            table,
            interpolation: None,
        }
    }

    pub fn with_power_params(mut self, params: PowerParameters) -> Self {
        self.power_params = params;
        self
    }

    pub fn compute_table(
        power_params: &PowerParameters,
        distances: &[f64],
        elevations: &[f64],
    ) -> Table {
        let mut table = Table::new();
        debug_assert!(distances.len() > 0);
        for power in (10..=1000).step_by(50) {
            let mut durations = vec![TimeDelta::zero(); distances.len()];
            power_params.for_each_segment(
                power as f64,
                &|i| distances[i],
                &|i| elevations[i],
                0,
                distances.len() - 1,
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

    fn generate_points(
        &self,
        power: f64,
        start: usize,
        end: usize,
        mut cumulative_duration: TimeDelta,
    ) -> Vec<InterpolationPoint> {
        debug_assert!(start < end);
        let mut points = Vec::new();
        self.power_params.for_each_segment(
            power,
            &|i| self.simplified_distances[i],
            &|i| self.simplified_elevations[i],
            start,
            end,
            |i, duration| {
                cumulative_duration += duration;
                let new = InterpolationPoint {
                    distance: self.simplified_distances[i],
                    duration: Some(cumulative_duration),
                    is_end: false,
                };
                if let Some(last) = points.last() {
                    debug_assert!(*last <= new);
                }
                points.push(new);
            },
        );
        let d_start = self.simplified_distances[start];
        let d_end = self.simplified_distances[end];
        // exclude the borders to avoid conflicts between intervals
        points.retain(|p| d_start < p.distance && p.distance < d_end);
        points
    }

    fn len(&self) -> usize {
        self.simplified_distances.len()
    }

    fn find_start_end(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> (usize, usize) {
        // In this function, we decide about what error we are going to make.
        // In KMH mode, there should be no error (we should have exact matches
        // since the waypoints indices where added to the DP indices).
        // In ACP/LRM mode there we will be an error. Either we take an interval
        // that is too large, or too narrow. The power will be evaluated with
        // this mismatch.
        use super::{index_after, index_before};
        let start = index_after(&self.simplified_distances, prev.distance);
        // iterations are run in start..=end => bound check.
        let end = (index_before(&self.simplified_distances, next.distance) + 1).min(self.len() - 1);
        let start_error = (self.simplified_distances[start] - prev.distance).abs();
        let end_error = (self.simplified_distances[end] - next.distance).abs();
        log::trace!("error: {:.1} {:.1}", start_error, end_error);
        (start, end)
    }

    fn solve_interval_linear(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> Vec<InterpolationPoint> {
        let (start, end) = self.find_start_end(prev, next);
        if start >= end {
            return Vec::new();
        }

        let target_duration = next.unwrap_duration() - prev.unwrap_duration();

        let mut p_low = 0i32;
        let mut d_low = TimeDelta::seconds(i32::MAX.into());
        let mut p_high = 0i32;
        let mut d_high = TimeDelta::zero();
        let mut bracket_found = false;

        for (&p, durations) in &self.table {
            let d = durations[end] - durations[start];
            if d >= target_duration {
                p_low = p;
                d_low = d;
            } else {
                p_high = p;
                d_high = d;
                bracket_found = true;
                break;
            }
        }

        let power = if !bracket_found {
            p_low as f64
        } else if p_low == 0 {
            p_high as f64
        } else {
            let total_time_gap = (d_low - d_high).num_milliseconds() as f64;
            let target_time_gap = (d_low - target_duration).num_milliseconds() as f64;
            let ratio = if total_time_gap > 0.0 {
                target_time_gap / total_time_gap
            } else {
                0.0
            };
            p_low as f64 + ratio * (p_high - p_low) as f64
        };

        self.generate_points(power, start, end, prev.unwrap_duration())
    }

    fn solve_interval_newton(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> Vec<InterpolationPoint> {
        let (start, end) = self.find_start_end(prev, next);
        if start >= end {
            return Vec::new();
        }

        let target_duration_secs =
            (next.unwrap_duration() - prev.unwrap_duration()).num_milliseconds() as f64 / 1000.0;

        let mut best_p = 200.0;
        let mut min_err = f64::MAX;
        for (&p, durations) in &self.table {
            let d_secs = (durations[end] - durations[start]).num_milliseconds() as f64 / 1000.0;
            let err = (d_secs - target_duration_secs).abs();
            if err < min_err {
                min_err = err;
                best_p = p as f64;
            }
        }

        let mut power = best_p;
        for _ in 0..5 {
            let (t, dt_dp) = self.power_params.duration_and_derivative_at_power(
                power,
                &|i| self.simplified_distances[i],
                &|i| self.simplified_elevations[i],
                start,
                end,
            );

            let err = t - target_duration_secs;
            if err.abs() < 0.1 {
                break;
            }

            if dt_dp.abs() < 1e-6 {
                break;
            }

            power = power - err / dt_dp;
            power = power.clamp(0.0, 2000.0);
        }

        self.generate_points(power, start, end, prev.unwrap_duration())
    }

    fn solve_interval_bisection(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> Vec<InterpolationPoint> {
        let (start, end) = self.find_start_end(prev, next);
        if start >= end {
            return Vec::new();
        }

        let duration = next.unwrap_duration() - prev.unwrap_duration();

        let power = self.power_params.power_at_duration(
            &duration,
            |i| self.simplified_distances[i],
            |i| self.simplified_elevations[i],
            start,
            end,
        );

        self.generate_points(power, start, end, prev.unwrap_duration())
    }

    pub fn update_interpolation_points(
        &mut self,
        controls: &Vec<InterpolationPoint>,
        method: SolverMethod,
    ) {
        debug_assert!(!controls.is_empty());
        debug_assert!(controls.is_sorted());
        let mut all_points = controls.clone();
        for window in controls.windows(2) {
            let prev = &window[0];
            let next = &window[1];
            let new_points = match method {
                SolverMethod::Linear => self.solve_interval_linear(prev, next),
                SolverMethod::Newton => self.solve_interval_newton(prev, next),
                SolverMethod::Bisection => self.solve_interval_bisection(prev, next),
            };
            // the new points should not include the controls.
            new_points.iter().for_each(|p| {
                debug_assert!(prev.distance < p.distance && p.distance < next.distance);
            });
            all_points.extend(new_points);
        }
        all_points.sort();
        self.interpolation = Some(all_points);
    }
}
