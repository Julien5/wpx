#![allow(dead_code)]
use core::f64;
use std::collections::{BTreeMap, BTreeSet};

use chrono::TimeDelta;
use geo::SimplifyIdx;

use crate::{
    elevation, geometry::power::PowerModel, inputpoint::InputPoint, parameters::PowerParameters,
    speed::InterpolationPoint,
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
    power_model: PowerModel,
    table: Table,
    pub interpolation: Option<Vec<InterpolationPoint>>,
}

impl ConstantPowerGeometry {
    pub fn new(
        power_parameters: &PowerParameters,
        distances: &[f64],
        elevations: &[f64],
        waypoints: &[InputPoint],
    ) -> Self {
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

        let power_model = PowerModel {
            parameters: power_parameters.clone(),
        };
        let table =
            Self::compute_table(&power_model, &simplified_distances, &simplified_elevations);

        log::trace!("build power model: weight: {} kg", power_parameters.W);

        Self {
            simplified_indices,
            simplified_distances,
            simplified_elevations,
            power_model,
            table,
            interpolation: None,
        }
    }

    pub fn with_power_params(mut self, params: PowerParameters) -> Self {
        self.power_model.parameters = params;
        self
    }

    pub fn compute_table(
        power_params: &PowerModel,
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
    ) -> (f64, Vec<InterpolationPoint>) {
        debug_assert!(start < end);
        let mut points = Vec::new();
        self.power_model.for_each_segment(
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
        let distance_start = self.simplified_distances[start];
        let distance_end = self.simplified_distances[end];
        // exclude the borders to avoid conflicts between intervals
        points.retain(|p| distance_start < p.distance && p.distance < distance_end);
        (power, points)
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
        debug_assert!(start < end);
        (start, end)
    }

    fn solve_interval_linear(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> (f64, Vec<InterpolationPoint>) {
        let (start, end) = self.find_start_end(prev, next);

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
    ) -> (f64, Vec<InterpolationPoint>) {
        let (start, end) = self.find_start_end(prev, next);

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
            let (t, dt_dp) = self.power_model.duration_and_derivative_at_power(
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
            power = power.max(0.0);
        }

        self.generate_points(power, start, end, prev.unwrap_duration())
    }

    fn solve_interval_bisection(
        &self,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> (f64, Vec<InterpolationPoint>) {
        let (start, end) = self.find_start_end(prev, next);

        let duration = next.unwrap_duration() - prev.unwrap_duration();

        let power = self.power_model.power_at_duraction_bisect(
            &duration,
            |i| self.simplified_distances[i],
            |i| self.simplified_elevations[i],
            start,
            end,
        );

        self.generate_points(power, start, end, prev.unwrap_duration())
    }

    pub fn solve_interval(
        &self,
        method: &SolverMethod,
        prev: &InterpolationPoint,
        next: &InterpolationPoint,
    ) -> (f64, Vec<InterpolationPoint>) {
        match method {
            SolverMethod::Linear => self.solve_interval_linear(prev, next),
            SolverMethod::Newton => self.solve_interval_newton(prev, next),
            SolverMethod::Bisection => self.solve_interval_bisection(prev, next),
        }
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
            let distance = next.distance - prev.distance;
            let time = next.unwrap_duration() - prev.unwrap_duration();
            debug_assert!(time >= TimeDelta::zero());
            if time.as_seconds_f64() < 60f64 {
                // skip it
                continue;
            }
            let speed_kmh = (distance / time.as_seconds_f64()) * (3600.0 / 1000.0);
            if speed_kmh > 1000f64 {
                // skip it
                continue;
            }
            let (power, mut new_points) = self.solve_interval(&method, prev, next);
            log::trace!(
                "[{:.1}-{:.1}] in {} hours => power: {:.1}W",
                prev.distance / 1000f64,
                next.distance / 1000f64,
                (next.unwrap_duration() - prev.unwrap_duration()).num_hours(),
                power
            );
            // Post solver cleaning:
            // Because the numerical solver may fail to find an exact solution, a power
            // that is a bit too high, which results in intermediate points that may be
            // reached before the next control time. We remove those intermediate points.
            // Note: The new points do not include the controls.
            new_points.retain(|p| {
                debug_assert!(prev.distance < p.distance && p.distance < next.distance);
                prev.unwrap_duration() <= p.unwrap_duration()
                    && p.unwrap_duration() <= next.unwrap_duration()
            });
            all_points.extend(new_points);
        }
        all_points.sort();
        debug_assert!(all_points.is_sorted_by_key(|a| a.distance));
        debug_assert!(all_points.is_sorted_by_key(|a| a.unwrap_duration()));
        self.interpolation = Some(all_points);
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use crate::{
        geometry::powergeometry::{ConstantPowerGeometry, SolverMethod},
        parameters::PowerParameters,
        speed::InterpolationPoint,
    };

    fn synthetic_geometry() -> ConstantPowerGeometry {
        let _ = env_logger::try_init();
        let step = 100.0;
        let n = (100_000.0 / step) as usize + 1;
        let mut distances = Vec::with_capacity(n);
        let mut elevations = Vec::with_capacity(n);
        let mut state: u64 = 42;
        let mut rng = move || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            state as f64 / u64::MAX as f64
        };
        let mut e = 500.0;
        for km in 0..n {
            let d = km as f64 * step;
            e = (e + (rng() - 0.5) * 20.0).clamp(0.0, 1000.0);
            distances.push(d);
            elevations.push(e);
        }
        ConstantPowerGeometry::new(
            &PowerParameters::default(),
            &distances,
            &elevations,
            &Vec::new(),
        )
    }

    fn compare_at_duration(hours: i64) {
        let geometry = synthetic_geometry();
        let start = InterpolationPoint {
            distance: 0f64,
            duration: Some(TimeDelta::zero()),
            is_end: false,
        };
        let end = InterpolationPoint {
            distance: 100_000f64,
            duration: Some(TimeDelta::hours(hours)),
            is_end: true,
        };
        let (p_lin, _) = geometry.solve_interval(&SolverMethod::Linear, &start, &end);
        let (p_new, _) = geometry.solve_interval(&SolverMethod::Newton, &start, &end);
        let (p_bis, _) = geometry.solve_interval(&SolverMethod::Bisection, &start, &end);
        if 1 < hours && hours < 10 {
            log::trace!("hours={hours:5} lin={p_lin:8.3} new={p_new:8.3}");
            assert!((p_bis - p_lin).abs() < 5.0,);
        }
        log::trace!("hours={hours:5} bis={p_bis:8.3} new={p_new:8.3}");
        let tol = if hours < 100 { 0.1 } else { 5.0 };
        assert!((p_bis - p_new).abs() < tol);
    }

    #[test]
    fn compare_solvers() {
        let _ = env_logger::try_init();
        compare_at_duration(10);
        compare_at_duration(5);
        compare_at_duration(4);
        compare_at_duration(3);
        compare_at_duration(2);
        compare_at_duration(1);
        compare_at_duration(100);
        compare_at_duration(1000);
    }
}
