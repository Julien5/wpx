use geo::{Distance, SimplifyIdx};

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let p1 = geo::Point::new(x1, y1);
    let p2 = geo::Point::new(x2, y2);
    geo::Haversine::distance(p1, p2)
}

use gpx::TrackSegment;
use std::io::Read;

#[derive(Clone)]
pub struct UTMPoint(f64, f64);

impl UTMPoint {
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
}

pub struct Track {
    pub wgs84: Vec<(f64, f64, f64)>,
    pub utm: Vec<UTMPoint>,
    _distance: Vec<f64>,
}

pub fn read_gpx(filename: &str) -> Box<gpx::Gpx> {
    let file = std::fs::File::open(filename).unwrap();
    let mut reader_file = std::io::BufReader::new(file);
    let mut content: Vec<u8> = Vec::new();
    let _ = reader_file.read_to_end(&mut content);
    let reader_mem = std::io::Cursor::new(content);
    Box::new(gpx::read(reader_mem).unwrap())
}

pub fn read_segment(gpx: &mut gpx::Gpx) -> Box<gpx::TrackSegment> {
    let mut t0 = gpx.tracks.swap_remove(0);
    let s0 = t0.segments.swap_remove(0);
    Box::new(s0)
}

impl Track {
    pub fn len(&self) -> usize {
        self.wgs84.len()
    }
    pub fn elevation(&self, index: usize) -> f64 {
        self.wgs84[index].2
    }
    pub fn distance(&self, index: usize) -> f64 {
        self._distance[index]
    }

    pub fn segment(&self, d0: f64, d1: f64) -> std::ops::Range<usize> {
        assert!(!self._distance.is_empty());
        assert!(d0 < d1);
        let maxdist = *self._distance.last().unwrap();
        let end = self._distance.len();
        let mut it = self._distance.iter();
        if d0 > maxdist {
            return end..end;
        }
        let startidx = it.position(|&d| d >= d0).unwrap();
        // TODO: binary search.
        let endidx = if d1 > maxdist {
            end
        } else {
            self._distance.iter().rposition(|&d| d < d1).unwrap()
        };
        assert!(startidx <= endidx);
        startidx..endidx
    }

    pub fn from_segment(segment: &TrackSegment) -> Track {
        let mut dist = Vec::new();

        use proj4rs::proj::Proj;
        let spec = "+proj=utm +zone=32 +datum=WGS84 +units=m +no_defs +type=crs";
        let utm32n = Proj::from_proj_string(spec).unwrap();

        let spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
        let wgs84 = Proj::from_proj_string(spec).unwrap();
        let mut utm = Vec::new();
        let mut wgs = Vec::new();
        for k in 0..segment.points.len() {
            let point = &segment.points[k];
            let (lon, lat) = point.point().x_y();
            wgs.push((lon, lat, point.elevation.unwrap()));
            debug_assert_eq!(wgs.len(), k + 1);
            let mut p = (lon.to_radians(), lat.to_radians());
            proj4rs::transform::transform(&wgs84, &utm32n, &mut p).unwrap();
            utm.push(UTMPoint(p.0, p.1));

            if k == 0 {
                dist.push(0f64);
            } else {
                let dloc = distance(wgs[k - 1].0, wgs[k - 1].1, wgs[k].0, wgs[k].1);
                dist.push(dist[k - 1] + dloc);
            }
        }
        assert_eq!(dist.len(), wgs.len());
        Track {
            wgs84: wgs,
            utm,
            _distance: dist,
        }
    }

    pub fn interesting_indexes(&self) -> Vec<usize> {
        let mut coords = Vec::new();
        for k in 0..self.len() {
            let x = self.distance(k);
            let y = self.elevation(k);
            coords.push(geo::coord!(x:x, y:y));
        }
        let line = geo::LineString::new(coords);
        line.simplify_idx(&70.0)
    }
}

pub struct Waypoint {
    pub wgs84: (f64, f64, f64),
    pub utm: UTMPoint,
    pub track_index: usize,
}

pub fn read_waypoints(gpx: &gpx::Gpx) -> Vec<Waypoint> {
    let mut ret = Vec::new();
    // TODO: remove proj4 duplicate
    use proj4rs::proj::Proj;
    let spec = "+proj=utm +zone=32 +datum=WGS84 +units=m +no_defs +type=crs";
    let utm32n = Proj::from_proj_string(spec).unwrap();

    let spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
    let wgs84 = Proj::from_proj_string(spec).unwrap();

    for w in &gpx.waypoints {
        let (lon, lat) = w.point().x_y();
        let mut p = (lon.to_radians(), lat.to_radians());
        proj4rs::transform::transform(&wgs84, &utm32n, &mut p).unwrap();
        ret.push(Waypoint::from_gpx(w, UTMPoint(p.0, p.1)));
    }
    ret
}

impl Waypoint {
    pub fn from_gpx(gpx: &gpx::Waypoint, utm: UTMPoint) -> Waypoint {
        let (lon, lat) = gpx.point().x_y();
        Waypoint {
            wgs84: (lon, lat, gpx.elevation.unwrap()),
            utm: utm,
            track_index: usize::MAX,
        }
    }
    pub fn from_track(wgs: (f64, f64, f64), utm: UTMPoint, indx: usize) -> Waypoint {
        Waypoint {
            wgs84: wgs.clone(),
            utm: utm,
            track_index: indx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
