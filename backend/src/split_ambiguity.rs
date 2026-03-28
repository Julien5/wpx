use std::collections::BTreeSet;

use crate::inputpoint::InputPoint;
use crate::parameters::TrackPart;
use crate::track::Track;

struct Point {
    pub primary: usize,
    pub secondary: Vec<usize>,
}

type Range = std::ops::Range<usize>;

fn ambiguities_count(points: &Vec<Point>, range: &Range) -> usize {
    let mut count = 0;
    points.iter().for_each(|p| {
        if !range.contains(&p.primary) {
            return;
        }
        for sec in &p.secondary {
            if range.contains(sec) {
                count += 1;
            }
        }
    });
    count
}

fn clear_range(points: &mut Vec<Point>, range: &Range) {
    points.retain(|p| !range.contains(&p.primary));
}

#[allow(dead_code)]
pub fn user_steps_split(steps: &Vec<InputPoint>, track: &Track) -> Vec<usize> {
    if steps.is_empty() {
        log::warn!("no steps to export => no split.");
        return Vec::new();
    }

    // the steps do not have multiple projections natively
    // => we must compute them.
    let mut points = Vec::new();
    steps.iter().for_each(|w| {
        let primary = w.track_projections.first().unwrap().track_index;
        let mut clone = w.clone();
        // ugly hack to bypass locate.rs:234
        clone.tags.insert("wpxtype".to_string(), "GPX".to_string());
        clone.track_projections.clear();
        track.project_point(&mut clone);
        let mut secondary = Vec::new();
        clone.track_projections.iter().for_each(|proj| {
            if proj.track_index != primary {
                secondary.push(proj.track_index);
            }
        });
        points.push(Point { primary, secondary });
    });

    let parts = match track.parts.len() > 1 {
        true => {
            log::trace!("using track parts");
            track.parts.clone()
        }
        false => {
            log::trace!("using tree parts");
            track.trees_parts()
        }
    };
    log::trace!("found {} parts", parts.len());
    let controls: BTreeSet<usize> = parts_to_ranges(&parts)
        .iter()
        .map(|range| range.end)
        .collect();

    log::trace!("controls {:?}", controls);

    let namb_max = 3;
    let mut good_range: Option<Range> = None;
    let mut good_ranges: Vec<Range> = Vec::new();
    for end in controls {
        let start = match good_ranges.last() {
            Some(r) => r.end,
            None => 0,
        };
        debug_assert!(end > start);
        let candidate = Range { start, end };
        let namb = ambiguities_count(&points, &candidate);
        if namb > namb_max {
            if good_range.is_some() {
                let good = good_range.unwrap().clone();
                clear_range(&mut points, &good);
                good_ranges.push(good);
            } else {
                log::warn!(
                    "candidate {:?} is not good, but we have no other [{:.0}-{:.0}]:",
                    candidate,
                    track.distance(candidate.start) / 1000f64,
                    track.distance(candidate.end) / 1000f64,
                );
                let good = candidate;
                clear_range(&mut points, &good);
                good_ranges.push(good);
            }
            good_range = None;
        } else {
            good_range = Some(candidate.clone());
        }
    }
    good_ranges.iter().map(|r| r.end).collect()
}

pub fn parts_to_ranges(parts: &Vec<TrackPart>) -> Vec<std::ops::Range<usize>> {
    parts
        .iter()
        .scan(0usize, |offset, part| {
            let start = *offset;
            *offset += part.length;
            Some(start..start + part.length)
        })
        .collect()
}
