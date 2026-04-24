use crate::{inputpoint::InputPoint, parameters::TrackPart, track::Track};

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

pub struct TrackSegment {
    pub name: String,
    pub range: std::ops::Range<usize>,
}

pub fn control_to_segments(track: &Track, controls: &Vec<InputPoint>) -> Vec<TrackSegment> {
    let mut ret = Vec::new();
    if controls.is_empty() {
        let ranges = parts_to_ranges(&track.parts);
        for (index, part) in track.parts.iter().enumerate() {
            ret.push(TrackSegment {
                name: part.name.clone(),
                range: ranges[index].clone(),
            });
        }
        return ret;
    }
    let mut start = 0;
    for control in controls.iter() {
        let last_index = control.track_projections.first().unwrap().track_index;
        ret.push(TrackSegment {
            name: format!("to {}", control.name()),
            range: start..last_index + 1,
        });
        start = last_index;
    }
    // to the end => this is it
    if start < track.len() - 1 {
        debug_assert!(false);
        ret.push(TrackSegment {
            name: format!("to end"),
            range: start..track.len(),
        });
    }
    ret
}
