#![allow(non_snake_case)]

use crate::track;

/*
 * converted to rust from gpxstudio:
 * https://github.com/gpxstudio/gpx.studio/blob/main/gpx/src/gpx.ts#L1945
 */
pub fn smooth(track: &track::Track, W: f64, signal: impl Fn(usize) -> f64) -> Vec<f64> {
    let L = track.wgs84.len();
    let mut ret = vec![0f64; L];
    let mut start = 0usize;
    let mut end = 0usize;
    let mut acc = 0f64;
    for i in 0..track.wgs84.len() {
        while start + 1 < i && (track.distance(i) - track.distance(start)) > W {
            acc -= signal(start);
            start += 1;
        }
        while end < L && (track.distance(end) - track.distance(i)) <= W {
            acc += signal(end);
            end += 1;
        }
        if start != end {
            ret[i] = acc / (end - start) as f64;
        } else {
            assert!(false);
            ret[i] = track.elevation(i);
        }
    }
    ret
}

pub fn smooth_elevation(track: &track::Track, W: f64) -> Vec<f64> {
    smooth(track, W, |index: usize| -> f64 { track.elevation(index) })
}

pub fn elevation_gain(smooth: &Vec<f64>, from: usize, to: usize) -> f64 {
    // log::trace!("{} - {} / {}", from, to, smooth.len());
    debug_assert!(from <= to);
    let mut ret = 0f64;
    for k in from..to {
        if k == 0 {
            continue;
        }
        let d = smooth[k] - smooth[k - 1];
        if d > 0f64 {
            ret += d;
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use crate::backend;

    #[tokio::test]
    async fn ele() {
        let mut backend = backend::Backend::make();
        backend
            .load_filename("data/blackforest.gpx")
            .await
            .expect("fail");
        let S = backend.segments();
        let km = 1000f64;
        for s in &S {
            let stat = backend.d().segment_statistics(s);
            log::info!(
                "{0} {1:8.1} -> {2:8.1}:  {3:8.1}",
                s.id,
                stat.distance_start / km,
                stat.distance_end / km,
                stat.elevation_gain
            );
        }
        assert_eq!(S.len(), 3);
    }
}
