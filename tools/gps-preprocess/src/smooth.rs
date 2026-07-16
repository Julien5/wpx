use crate::data::Input;

pub fn smooth_speed(input: &[Input], window_sec: f64) -> Vec<f64> {
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    let half = (window_sec / 2.0).round() as usize;
    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let start = i.saturating_sub(half);
        let end = (i + half).min(n - 1);
        let count = end - start + 1;
        let sum: f64 = input[start..=end].iter().map(|x| x.speed).sum();
        result.push(sum / count as f64);
    }

    result
}

pub fn slope(input: &[Input], window_sec: f64) -> Vec<f64> {
    let n = input.len();
    if n == 0 {
        return Vec::new();
    }
    let half = (window_sec / 2.0).round() as usize;
    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let start = i.saturating_sub(half);
        let end = (i + half).min(n - 1);
        let count = (end - start + 1) as f64;

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for j in start..=end {
            let x = input[j].distance;
            let y = input[j].elevation;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let denom = count * sum_xx - sum_x * sum_x;
        if denom.abs() < 1e-12 {
            result.push(0.0);
        } else {
            let s = (count * sum_xy - sum_x * sum_y) / denom;
            result.push(s);
        }
    }

    result
}
