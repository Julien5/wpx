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

#[allow(dead_code)]
pub fn split_parts(parts: &Vec<TrackPart>, split_index: usize, name: &str) -> Vec<TrackPart> {
    let mut result = Vec::new();
    let mut current_start = 0usize;

    for part in parts {
        let part_end = current_start + part.length;

        if split_index <= current_start || split_index >= part_end {
            // Split point is outside this part — keep it unchanged
            result.push(part.clone());
        } else {
            // Split point falls inside this part
            let first_length = split_index - current_start;
            let second_length = part.length - first_length;
            result.push(TrackPart {
                name: format!("{}", name),
                part_index: part.part_index,
                length: first_length,
            });
            result.push(TrackPart {
                name: format!("{}", part.name),
                part_index: part.part_index,
                length: second_length,
            });
        }

        current_start = part_end;
    }

    result
}

#[allow(dead_code)]
pub fn join_parts(parts: &Vec<TrackPart>, join_index: usize) -> Vec<TrackPart> {
    let ranges = parts_to_ranges(parts);
    let start_at_split = ranges.iter().position(|range| range.start == join_index);
    let mut ret = parts.clone();
    if let Some(index) = start_at_split {
        // we must join index-1 and index.
        debug_assert!(index > 0);
        let current = &parts[index];
        let previous = &parts[index - 1];
        let joined = TrackPart {
            name: current.name.clone(),
            part_index: current.part_index,
            length: previous.length + current.length,
        };
        ret[index] = joined;
        ret.remove(index - 1);
    }
    ret
}

#[cfg(test)]
mod tests {
    use crate::{
        parameters::TrackPart,
        trackparts::{join_parts, split_parts},
    };

    #[test]
    fn split() {
        let _ = env_logger::try_init();
        let parts = vec![
            TrackPart {
                name: format!("part-{}", 1),
                part_index: 0,
                length: 5,
            },
            TrackPart {
                name: format!("part-{}", 2),
                part_index: 0,
                length: 5,
            },
        ];
        let splits = split_parts(&parts, 7, "to waypoint");
        for part in &splits {
            log::trace!("{:?}", part);
        }
        assert_eq!(splits.len(), 3);
        assert_eq!(splits[0].length, 5);
        assert_eq!(splits[1].length, 2);
        assert_eq!(splits[2].length, 3);
    }

    #[test]
    fn join() {
        let _ = env_logger::try_init();
        let parts = vec![
            TrackPart {
                name: format!("part-{}", 1),
                part_index: 0,
                length: 5,
            },
            TrackPart {
                name: format!("part-{}", 2),
                part_index: 0,
                length: 5,
            },
        ];
        let splits = join_parts(&parts, 5);
        for part in &splits {
            log::trace!("{:?}", part);
        }
        assert_eq!(splits.len(), 1);
    }
}
