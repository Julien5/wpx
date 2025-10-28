use crate::{
    backend::Segment,
    inputpoint::{InputPoint, InputType},
    parameters::Parameters,
};

fn _placement_order_profile(point: &InputPoint) -> usize {
    let delta = point.distance_to_track();
    let kind = point.kind();
    let population = match point.population() {
        Some(p) => p,
        None => 0,
    };
    let mut ret = 1;
    if kind == InputType::GPX && delta < 1000f64 {
        return ret;
    }
    ret += 1;
    if kind == InputType::City && delta < 1000f64 {
        return ret;
    }
    ret += 1;
    if kind == InputType::Village && delta < 1000f64 && population > 1000 {
        return ret;
    }
    ret += 1;
    if (kind == InputType::MountainPass || kind == InputType::Peak) && delta < 500f64 {
        return ret;
    }
    ret += 1;
    if kind == InputType::Village && delta < 200f64 {
        return ret;
    }
    ret += 10;
    ret
}
/*
fn _important(p: &InputPoint) -> bool {
    let pop = match p.population() {
        Some(n) => n,
        None => {
            if p.kind() == InputType::City {
                1000
            } else {
                0
            }
        }
    };
    let dist = p.distance_to_track();
    if pop > 100000 && dist < 5000f64 {
        return true;
    }
    if pop > 10000 && dist < 1000f64 {
        return true;
    }
    if pop >= 500 && dist < 500f64 {
        return true;
    }
    /*if dist < 2000f64 {
        log::trace!(
            "too far for the profile:{:?} {:?} {:?} d={:.1}",
            p.kind(),
            p.population(),
            p.name(),
            dist
        );
    }*/
    false
}

type Interval = std::ops::Range<usize>;
type Points = Vec<InputPoint>;

fn contains(interval: &Interval, point: &InputPoint) -> bool {
    let index = point.track_projection.as_ref().unwrap().track_index;
    interval.start <= index && index < interval.end
}

fn tight(interval: &mut Interval, track: &Track) {
    let dstart = track.distance(interval.start);
    let dend = track.distance(interval.end - 1);
    let margin = (dend - dstart) / 8f64;
    interval.start = track.index_after(dstart + margin);
    interval.end = track.index_before(dend - margin);
}

fn largest_interval(segment: &Segment, points: &Points) -> Interval {
    let mut indices: Vec<_> = points
        .iter()
        .map(|p| p.track_projection.as_ref().unwrap().track_index)
        .collect();
    indices.sort();
    let mut prev = 0usize;
    let mut intervals = Vec::new();
    for i in indices {
        intervals.push(Interval {
            start: prev,
            end: i,
        });
        prev = i;
    }
    intervals.push(Interval {
        start: prev,
        end: segment.range.end,
    });
    intervals.sort_by_key(|i| i.len());
    intervals.last().unwrap().clone()
}
 */

fn profile_points_elevation_gain(segment: &Segment, d: &f64) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    let mut prev = 0;
    let mut index = prev + 1;
    let length = segment.track.len();
    let mut count = 1;
    loop {
        if index >= length {
            break;
        }
        let g = segment.track.elevation_gain_on_range(&std::ops::Range {
            start: prev,
            end: index,
        });
        if g >= *d {
            let mut w = segment
                .track
                .create_point_on_track(index, &format!("P{}", count));
            w.label_placement_order = 2;
            if segment.range.contains(&index) {
                ret.push(w);
            }
            count += 1;
            prev = index;
        }
        index += 1;
    }
    ret.retain(|w| segment.map_bbox.contains(&w.euclidian.xy()));

    ret
}

fn profile_points_distance(segment: &Segment, d: &f64) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    let mut prev = 0;
    let mut index = prev + 1;
    let mut count = 1;
    loop {
        if index >= segment.range.end {
            break;
        }
        if segment.track.distance(index) - segment.track.distance(prev) >= *d {
            let mut w = segment
                .track
                .create_point_on_track(index, &format!("P{}", count));
            w.label_placement_order = 2;
            if segment.range.contains(&index) {
                ret.push(w);
            }
            count += 1;
            prev = index;
        }
        index += 1;
    }
    ret.retain(|w| segment.map_bbox.contains(&w.euclidian.xy()));
    ret
}

pub fn profile_points(segment: &Segment, parameters: &Parameters) -> Vec<InputPoint> {
    match parameters.profile_options.step_distance {
        None => {}
        Some(d) => {
            return profile_points_distance(segment, &d);
        }
    }

    match parameters.profile_options.step_elevation_gain {
        None => {}
        Some(d) => {
            return profile_points_elevation_gain(segment, &d);
        }
    }
    Vec::new()
}
