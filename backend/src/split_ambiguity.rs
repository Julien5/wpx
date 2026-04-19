use std::collections::BTreeSet;

use crate::inputpoint::InputPoint;
use crate::track::Track;
use crate::trackparts::parts_to_ranges;

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

pub fn user_steps_split(
    steps: &Vec<InputPoint>,
    controls: &Vec<InputPoint>,
    track: &Track,
) -> Vec<usize> {
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

    let mut candidate_indices: BTreeSet<usize> = match controls.is_empty() {
        true => {
            let ranges = parts_to_ranges(&track.trees_parts());
            ranges.iter().map(|r| r.end).collect()
        }
        false => controls
            .iter()
            .map(|c| c.track_projections.first().unwrap().track_index)
            .collect(),
    };

    log::trace!("found {} candidate indices", candidate_indices.len());

    if !candidate_indices.contains(&(track.len() - 1)) {
        candidate_indices.insert(track.len() - 1);
    }

    log::trace!("controls {:?}", controls);

    let namb_max = 3;
    let mut good_range: Option<Range> = None;
    let mut good_ranges: Vec<Range> = Vec::new();
    for end in candidate_indices {
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
