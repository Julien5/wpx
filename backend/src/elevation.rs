#![allow(non_snake_case)]

use crate::gpsdata;

pub fn smooth(track: &gpsdata::Track) -> Vec<f64> {
    let L = track.wgs84.len();
    let mut ret = vec![0f64; L];
    let mut start = 0usize;
    let mut end = 0usize;
    let W = 200f64;
    let mut acc = 0f64;
    for i in 0..track.wgs84.len() {
        while start + 1 < i && (track.distance(i) - track.distance(start)) > W {
            acc -= track.elevation(start);
            start += 1;
        }
        while end < L && (track.distance(end) - track.distance(i)) <= W {
            acc += track.elevation(end);
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

#[cfg(test)]
mod tests {
    use crate::backend;

    #[test]
    fn ele() {
        let backend = backend::Backend::new("data/blackforest.gpx");
        let S = backend.segments();
        let km = 1000f64;
        for s in &S {
            let stat = backend.segment_statistics(s);
            println!(
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
