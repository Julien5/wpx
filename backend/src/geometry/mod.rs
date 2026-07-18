pub mod mapgeometry;
pub mod power;
pub mod powergeometry;
pub mod profilegeometry;

// the first index with distance[index] >= d
pub fn index_after(distances: &[f64], d: f64) -> usize {
    if d < 0.0 {
        return 0;
    }
    let maxdist = distances.last().copied().unwrap_or(0.0);
    if d > maxdist {
        return distances.len();
    }
    // TODO: make fast since the distance are sorted.
    distances.iter().position(|&x| x >= d).unwrap()
}

pub fn index_before(distances: &[f64], d: f64) -> usize {
    assert!(!distances.is_empty());
    assert!(d >= 0.0);
    let maxdist = distances.last().copied().unwrap_or(0.0);
    if d >= maxdist {
        return distances.len() - 1;
    }
    if d <= 0.0 {
        return 0;
    }
    // TODO: make fast since the distance are sorted.
    match distances.iter().rposition(|&x| x < d) {
        Some(idx) => idx,
        None => {
            log::error!("no index_before distance {}", d);
            0
        }
    }
}
