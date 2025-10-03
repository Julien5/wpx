use crate::gpsdata::ProfileBoundingBox;

fn snap_ceil(x: f64, step: f64) -> f64 {
    (x / step).ceil() * step
}

fn snap_floor(x: f64, step: f64) -> f64 {
    (x / step).floor() * step
}

fn xtick_delta(bbox: &ProfileBoundingBox, W: f64) -> f64 {
    let min = 50f64 * bbox.width() / W;
    for candidate in [1, 2, 10, 20, 50, 100, 250] {
        let ret = 1000f64 * candidate as f64;
        if ret > min {
            return ret;
        }
    }
    100f64
}

fn xticks_all(bbox: &ProfileBoundingBox, W: f64) -> Vec<f64> {
    let mut ret = Vec::new();
    let _D = bbox.width();
    let delta = xtick_delta(bbox, W);
    let mut start = snap_floor(bbox._min.0, delta);
    start = start.max(0f64);
    let stop = snap_ceil(bbox._max.0, delta);
    let mut p = start;
    while p <= stop {
        ret.push(p);
        p = p + delta;
    }
    ret
}

pub fn xticks_dashed(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = xticks_all(bbox, H);
    let mut k = 0;
    ret.retain(|_x| {
        k += 1;
        (k % 2) == 0
    });
    ret
}

pub fn xticks(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = xticks_all(bbox, H);
    let mut k = 0;
    ret.retain(|_y| {
        k += 1;
        (k % 2) != 0
    });
    ret
}

/* ** */

fn ytick_delta(height: &f64, H: f64) -> f64 {
    let min = 20f64 * height / H;
    for candidate in [10, 20, 50, 100, 200, 250, 500, 1000] {
        let ret = candidate as f64;
        if ret > min {
            return ret;
        }
    }
    100f64
}

fn yticks_all(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = Vec::new();
    let delta = ytick_delta(&bbox.height().max(750f64), H);
    let mut start = snap_floor(bbox._min.1, delta) - delta;
    start = start.max(0f64);
    let mut stop = snap_ceil(bbox._max.1, delta) + 2f64 * delta;
    while stop - start < 750f64 {
        start -= delta;
        start = start.max(0f64);
        stop += delta;
    }

    let mut p = start;
    while p <= stop {
        ret.push(p);
        p = p + delta;
    }
    ret
}

pub fn yticks_dashed(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = yticks_all(bbox, H);
    let mut k = 0;
    ret.retain(|_y| {
        k += 1;
        (k % 2) == 0
    });
    ret
}

pub fn yticks(bbox: &ProfileBoundingBox, H: f64) -> Vec<f64> {
    let mut ret = yticks_all(bbox, H);
    let mut k = 0;
    ret.retain(|_y| {
        k += 1;
        (k % 2) != 0
    });
    ret
}
