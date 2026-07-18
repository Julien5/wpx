use crate::{
    error::TrackError, gpsdata::distance_wgs84, inputpoint::InputPoint, parameters::TrackPart,
    track::Track, wgs84point::WGS84Point,
};

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

pub fn controls_to_segments(track: &Track, controls: &Vec<InputPoint>) -> Vec<TrackSegment> {
    let mut ret = Vec::new();
    let mut start = 0;
    for control in controls.iter() {
        let last_index = control.track_projections.first().unwrap().track_index;
        // handle the case of the START control
        if last_index == 0 {
            continue;
        }
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

pub struct ProtoTrack {
    pub wgs84: Vec<WGS84Point>,
    pub parts: Vec<TrackPart>,
}

impl ProtoTrack {
    pub fn name(&self) -> String {
        debug_assert!(!self.parts.is_empty());
        self.parts.first().as_ref().unwrap().name.clone()
    }
}

pub fn proto(gpxtracks: &Vec<(String, gpx::Track)>) -> Result<ProtoTrack, TrackError> {
    let mut wgs = Vec::new();
    let mut parts = Vec::new();

    let mut last_point = None;
    for (index, (name, track)) in gpxtracks.iter().enumerate() {
        debug_assert_eq!(track.segments.len(), 1);
        let mut length = 0usize;
        for segment in &track.segments {
            for k in 0..segment.points.len() {
                let point = &segment.points[k];
                let (lon, lat) = point.point().x_y();
                let elevation = match point.elevation {
                    Some(e) => e,
                    None => {
                        return Err(TrackError::MissingElevation { index: k });
                    }
                };

                let w = WGS84Point::new(&lon, &lat, &elevation);

                if last_point.is_some() && distance_wgs84(&last_point.unwrap(), &w) == 0f64 {
                    continue;
                }

                wgs.push(w);
                length += 1;
                last_point = Some(w.clone());
            }
        }
        let part = TrackPart {
            name: name.clone(),
            length,
            part_index: index,
        };
        parts.push(part);
    }
    let ret = ProtoTrack {
        wgs84: wgs,
        parts: parts,
    };
    Ok(ret)
}
