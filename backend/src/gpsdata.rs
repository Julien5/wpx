use crate::error::Error;
use crate::track;
use crate::waypoint::WGS84Point;
use crate::waypoint::Waypoint;
use crate::waypoint::Waypoints;
use geo::Distance;

pub fn distance_wgs84(p1: &WGS84Point, p2: &WGS84Point) -> f64 {
    let p1 = geo::Point::new(p1.x(), p1.y());
    let p2 = geo::Point::new(p2.x(), p2.y());
    geo::Haversine::distance(p1, p2)
}

pub fn read_gpx_content(bytes: &Vec<u8>) -> Result<gpx::Gpx, Error> {
    let reader_mem = std::io::Cursor::new(bytes);
    match gpx::read(reader_mem) {
        Ok(d) => Ok(d),
        Err(_e) => Err(Error::GPXInvalid),
    }
}

pub fn read_segment(gpx: &mut gpx::Gpx) -> Result<gpx::TrackSegment, Error> {
    let tracks = &mut gpx.tracks;
    tracks.sort_by_key(|track| {
        let zero = "A".to_string();
        let infinity = "ziel".to_string();
        if track.name.is_none() {
            return zero;
        }
        let name = track.name.as_ref().unwrap().to_lowercase();
        if name.contains("end") {
            return infinity;
        }
        if name.contains("ziel") {
            return infinity;
        }
        if name.contains("start") {
            return zero;
        }
        return name;
    });
    let mut ret = gpx::TrackSegment::new();
    for track in tracks {
        let points = &track.segments.first().unwrap().points;
        for k in 0..points.len() {
            ret.points.push(points[k].clone());
        }
    }
    if ret.points.is_empty() {
        return Err(Error::GPXHasNoSegment);
    }
    Ok(ret)
}

#[derive(Clone)]
pub struct ProfileBoundingBox {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
}

fn snap_ceil(x: f64) -> f64 {
    (x / 500f64).ceil() * 500f64
}

fn snap_floor(x: f64) -> f64 {
    (x / 500f64).floor() * 500f64
}

impl ProfileBoundingBox {
    pub fn from_track(track: &track::Track, range: &std::ops::Range<usize>) -> ProfileBoundingBox {
        let mut ymin = f64::MAX;
        let mut ymax = f64::MIN;
        for k in range.start..range.end {
            let y = track.elevation(k);
            ymin = y.min(ymin);
            ymax = y.max(ymax);
        }
        let mut p = ProfileBoundingBox {
            xmin: track.distance(range.start),
            xmax: track.distance(range.end - 1),
            ymin: ymin,
            ymax: ymax,
        };
        p.fix_margins();
        p
    }

    fn fix_margins(&mut self) {
        let km = 1000f64;
        let shift = 20f64 * km;
        let margin = 10f64 * km;
        self.xmin = ((self.xmin - margin) / shift).floor() * shift;
        self.xmin = self.xmin.max(0f64);
        self.xmax = ((self.xmax + margin) / shift).ceil() * shift;
        self.ymin = snap_floor(self.ymin - 100f64);
        self.ymax = snap_ceil(self.ymax + 100f64).max(snap_floor(self.ymin + 500f64));
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        if x < self.xmin || x > self.xmax {
            return false;
        }
        if y < self.ymin || y > self.ymax {
            return false;
        }
        true
    }
}

pub fn read_waypoints(gpx: &gpx::Gpx) -> Waypoints {
    let mut ret = Vec::new();
    // TODO: remove proj4 duplicate
    for w in &gpx.waypoints {
        ret.push(Waypoint::from_gpx(w, w.name.clone(), w.description.clone()));
    }
    ret
}

#[cfg(test)]
mod tests {
    use geo::line_string;
    use geo::Simplify;
    #[test]
    fn simplify() {
        let line_string = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 4.0),
            (x: 11.0, y: 5.5),
            (x: 17.3, y: 3.2),
            (x: 27.8, y: 0.1),
        ];

        let simplified = line_string.simplify(&1.0);

        let expected = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 4.0),
            (x: 11.0, y: 5.5),
            (x: 27.8, y: 0.1),
        ];

        assert_eq!(expected, simplified);
    }
}
