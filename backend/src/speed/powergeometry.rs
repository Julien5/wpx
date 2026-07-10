#![allow(dead_code)]
use core::f64;
use std::collections::BTreeMap;

use chrono::TimeDelta;

use crate::{
    speed::{power::PowerParameters, InterpolationPoint},
    track::Geometry,
};

pub type Table = BTreeMap<i32, Vec<TimeDelta>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SolverMethod {
    Linear,
    Newton,
    Bisection,
}

#[derive(Clone)]
pub struct ConstantPowerGeometry {
    geometry: Geometry,
    power_params: PowerParameters,
    table: Table,
    pub interpolation: Option<Vec<InterpolationPoint>>,
}

impl ConstantPowerGeometry {
    pub fn compute_table(power_params: &PowerParameters, geometry: &Geometry) -> Table {
        let mut table = Table::new();
        debug_assert!(geometry.len() > 0);
        for power in (10..=1000).step_by(50) {
            let mut durations = vec![TimeDelta::zero(); geometry.len()];
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
            interpolation: None,
        }
    }

    pub fn with_power_params(mut self, params: PowerParameters) -> Self {
        self.power_params = params;
        self
    }

    /// Common point generator used by all solvers
    fn generate_points(
        &self,
        power: f64,
        start: usize,
        end: usize,
        mut cumulative_duration: TimeDelta,
    ) -> Vec<InterpolationPoint> {
        let mut points = Vec::new();
        self.power_params.for_each_segment(
            power,
            &|i| self.geometry.distance(i),
            &|i| self.geometry.elevation(i),
            start,
            end,
            |i, duration| {
                cumulative_duration += duration;
                let new = InterpolationPoint {
                    distance: self.geometry.distance(i),
                    duration: Some(cumulative_duration),
                    is_end: false,
                };
                if let Some(last) = points.last() {
                    debug_assert!(*last <= new);
                }
                points.push(new);
            },
        );
        points
    }

    // -------------------------------------------------------------------------
    // ALGORITHM 1: Memory-Cache Table + Linear Interpolation
    // -------------------------------------------------------------------------
    fn solve_interval_linear(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> Vec<InterpolationPoint> {
        let start = self.geometry.index_after(prev.distance);
        let mut end = self.geometry.index_before(next.distance) + 1;
        end = end.min(self.geometry.len() - 1);
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

    // -------------------------------------------------------------------------
    // ALGORITHM 2: Table Seeding + Newton-Raphson Refinement
    // -------------------------------------------------------------------------
    fn solve_interval_newton(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> Vec<InterpolationPoint> {
        let start = self.geometry.index_after(prev.distance);
        let mut end = self.geometry.index_before(next.distance) + 1;
        end = end.min(self.geometry.len() - 1);
        if start >= end {
            return Vec::new();
        }

        let target_duration_secs =
            (next.unwrap_duration() - prev.unwrap_duration()).num_milliseconds() as f64 / 1000.0;

        // 1. Seed from the closest table value
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

        // 2. Newton-Raphson Iteration
        let mut power = best_p;
        for _ in 0..5 {
            // Cap at 5 to prevent infinite loops, usually converges in 1-2
            let (t, dt_dp) = self.power_params.duration_and_derivative_at_power(
                power,
                &|i| self.geometry.distance(i),
                &|i| self.geometry.elevation(i),
                start,
                end,
            );

            let err = t - target_duration_secs;
            if err.abs() < 0.1 {
                break;
            } // Converged to within 0.1 seconds

            if dt_dp.abs() < 1e-6 {
                break;
            } // Prevent division by zero

            power = power - err / dt_dp;
            power = power.clamp(0.0, 2000.0); // Keep physics sane
        }

        self.generate_points(power, start, end, prev.unwrap_duration())
    }

    // -------------------------------------------------------------------------
    // ALGORITHM 3: Original 60-iteration Bisection Approach (Baseline)
    // -------------------------------------------------------------------------
    fn solve_interval_60iterations(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> Vec<InterpolationPoint> {
        let start = self.geometry.index_after(prev.distance);
        let mut end = self.geometry.index_before(next.distance) + 1;
        end = end.min(self.geometry.len() - 1);
        if start >= end {
            return Vec::new();
        }

        let duration = next.unwrap_duration() - prev.unwrap_duration();

        let power = self.power_params.power_at_duration(
            &duration,
            |i| self.geometry.distance(i),
            |i| self.geometry.elevation(i),
            start,
            end,
        );

        self.generate_points(power, start, end, prev.unwrap_duration())
    }

    /// Builds a dense set of interpolation points from sparse control points.
    pub fn update_interpolation_points(
        &mut self,
        controls: &Vec<InterpolationPoint>,
        method: SolverMethod,
    ) {
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

            let new_points = match method {
                SolverMethod::Linear => self.solve_interval_linear(prev, next),
                SolverMethod::Newton => self.solve_interval_newton(prev, next),
                SolverMethod::Bisection => self.solve_interval_60iterations(prev, next),
            };

            all_points.extend(new_points);
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
        self.interpolation = Some(all_points);
    }
}
