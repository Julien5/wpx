use crate::parameters::TrackPart;

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
