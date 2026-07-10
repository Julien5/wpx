#![allow(non_snake_case)]

use crate::geometry::profilegeometry::ProfileGeometry;

/*
 * converted to rust from gpxstudio:
 * https://github.com/gpxstudio/gpx.studio/blob/main/gpx/src/gpx.ts#L1945
 */
pub fn smooth(
    W: f64,
    L: usize,
    distance: impl Fn(usize) -> f64,
    elevation: impl Fn(usize) -> f64,
) -> Vec<f64> {
    let mut ret = vec![0f64; L];
    let mut start = 0usize;
    let mut end = 0usize;
    let mut acc = 0f64;
    for i in 0..L {
        while start + 1 < i && (distance(i) - distance(start)) > W {
            acc -= elevation(start);
            start += 1;
        }
        while end < L && (distance(end) - distance(i)) <= W {
            acc += elevation(end);
            end += 1;
        }
        if start != end {
            ret[i] = acc / (end - start) as f64;
        } else {
            debug_assert!(false);
            ret[i] = elevation(i);
        }
    }
    ret
}

pub fn elevation_gain(smooth: &ProfileGeometry, from: usize, to: usize) -> f64 {
    debug_assert!(from <= to, "from:{}, to:{}", from, to);
    smooth.elevation_gain(to) - smooth.elevation_gain(from)
}

#[cfg(test)]
mod tests {
    use crate::testhelpers::load_backend_data_without_osm;

    #[test]
    fn ele() {
        let _ = env_logger::try_init();
        let backend = load_backend_data_without_osm("data/blackforest.gpx");
        let S = backend.segments();
        assert_eq!(S.len(), 3);
        let km = 1000f64;
        for s in &S {
            let stat = backend.segment_statistics(s);
            log::info!(
                "{0} {1:8.1} -> {2:8.1}:  {3:8.1}",
                s.id,
                stat.distance_start / km,
                stat.distance_end / km,
                stat.elevation_gain
            );
        }
    }
}
