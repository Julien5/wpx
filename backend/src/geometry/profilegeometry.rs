use geo::SimplifyIdx;

use crate::elevation;

#[derive(Clone)]
pub struct ProfileGeometry {
    distances: Vec<f64>,
    elevations: Vec<f64>,
    cumulative_gain: Vec<f64>,
    simplified_indices: Vec<usize>,
}

fn compute_cumulative_gain(elevations: &[f64]) -> Vec<f64> {
    let mut gains = Vec::with_capacity(elevations.len());
    let mut last_gain = 0f64;
    for i in 0..elevations.len() {
        let gain = if i == 0 {
            last_gain
        } else {
            let d = elevations[i] - elevations[i - 1];
            if d > 0.0 {
                last_gain + d
            } else {
                last_gain
            }
        };
        gains.push(gain);
        last_gain = gain;
    }
    gains
}

fn interpolate_index(values: &[f64], target: f64, k0: usize) -> f64 {
    assert!(!values.is_empty());
    if target <= 0.0 {
        return 0.0;
    }
    let last = values.last().unwrap();
    if target >= *last {
        return (values.len() - 1) as f64;
    }
    let slice = &values[k0..];
    let k_local = slice.partition_point(|&v| v < target);
    let k = k0 + k_local;
    let (prev, next) = if k_local == 0 {
        let prev = if k0 > 0 { values[k0 - 1] } else { 0.0 };
        (prev, values[k0])
    } else {
        (values[k - 1], values[k])
    };
    let base = if k_local == 0 {
        k0.saturating_sub(1)
    } else {
        k - 1
    };
    let t = (target - prev) / (next - prev);
    base as f64 + t
}

impl ProfileGeometry {
    pub fn new(distances: Vec<f64>, elevation: &impl Fn(usize) -> f64) -> Self {
        let smooth_elevation = elevation::smooth(
            200f64,
            distances.len(),
            &|i: usize| -> f64 { distances[i] },
            elevation,
        );
        let simplified_indices: Vec<usize> = {
            let coords: Vec<geo::Coord> = smooth_elevation
                .iter()
                .enumerate()
                .map(|(idx, elevation)| geo::coord!(x: distances[idx], y: *elevation))
                .collect();
            let line = geo::LineString::new(coords);
            let epsilon = 2f64;
            line.simplify_idx(epsilon)
        };

        let cumulative_gain = compute_cumulative_gain(&smooth_elevation);
        Self {
            distances,
            elevations: smooth_elevation,
            cumulative_gain,
            simplified_indices,
        }
    }

    pub fn distance(&self, index: usize) -> f64 {
        self.distances[index]
    }

    pub fn elevation(&self, index: usize) -> f64 {
        self.elevations[index]
    }

    pub fn elevation_gain(&self, index: usize) -> f64 {
        self.cumulative_gain[index]
    }

    pub fn total_distance(&self) -> f64 {
        self.distances.last().copied().unwrap_or(0.0)
    }

    pub fn len(&self) -> usize {
        self.distances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.distances.is_empty()
    }

    pub fn index_after(&self, distance: f64) -> usize {
        super::index_after(&self.distances, distance)
    }

    pub fn index_before(&self, distance: f64) -> usize {
        super::index_before(&self.distances, distance)
    }

    pub fn point_at_distance(&self, d: f64, k0: usize) -> f64 {
        interpolate_index(&self.distances, d, k0)
    }

    pub fn point_at_elevation_gain(&self, d: f64, k0: usize) -> f64 {
        interpolate_index(&self.cumulative_gain, d, k0)
    }

    pub fn simplified_indices(&self) -> &[usize] {
        &self.simplified_indices
    }

    pub fn gain_on_range(&self, range: &std::ops::Range<usize>) -> f64 {
        debug_assert!(range.end <= self.len());
        debug_assert!(range.start < self.len());
        self.elevation_gain(range.end - 1) - self.elevation_gain(range.start)
    }

    pub fn subrange(&self, d0: f64, d1: f64) -> std::ops::Range<usize> {
        debug_assert!(self.len() > 0);
        debug_assert!(d0 < d1);
        let startidx = self.index_after(d0);
        let endidx = self.index_before(d1) + 1;
        debug_assert!(endidx <= self.len());
        startidx..endidx
    }
}
