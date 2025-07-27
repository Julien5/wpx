use crate::{step::Step, utm::UTMPoint};
use geo::{Distance, SimplifyIdx};

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let p1 = geo::Point::new(x1, y1);
    let p2 = geo::Point::new(x2, y2);
    geo::Haversine::distance(p1, p2)
}

use gpx::TrackSegment;
use std::{io::Read, str::FromStr};

use crate::elevation;

pub struct Track {
    pub wgs84: Vec<(f64, f64, f64)>,
    pub utm: Vec<UTMPoint>,
    _distance: Vec<f64>,
}

pub fn read_gpx_content(bytes: &Vec<u8>) -> Box<gpx::Gpx> {
    let reader_mem = std::io::Cursor::new(bytes);
    Box::new(gpx::read(reader_mem).unwrap())
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
    pub fn elevation_gain(&self, range: &std::ops::Range<usize>) -> f64 {
        // TODO: compute it.
        let smooth_elevation = elevation::smooth(&self, 200f64, |index: usize| -> f64 {
            self.elevation(index)
        });
        let mut ret = 0f64;
        for k in range.start + 1..range.end {
            let d = smooth_elevation[k] - smooth_elevation[k - 1];
            //let d = self.elevation(k) - self.elevation(k - 1);
            if d > 0.0 {
                ret = ret + d;
            }
        }
        ret
    }
    pub fn distance(&self, index: usize) -> f64 {
        self._distance[index]
    }

    pub fn index_after(&self, distance: f64) -> usize {
        if distance < 0f64 {
            return 0;
        }
        let maxdist = *self._distance.last().unwrap();
        let end = self._distance.len();
        if distance > maxdist {
            return end;
        }
        let mut it = self._distance.iter();
        // positions stops on true
        it.position(|&d| d >= distance).unwrap()
    }
    pub fn index_before(&self, distance: f64) -> usize {
        assert!(distance >= 0f64);
        let maxdist = *self._distance.last().unwrap();
        let end = self._distance.len();
        if distance > maxdist {
            return end;
        }
        let mut it = self._distance.iter();
        it.rposition(|&d| d < distance).unwrap()
    }

    pub fn segment(&self, d0: f64, d1: f64) -> std::ops::Range<usize> {
        assert!(!self._distance.is_empty());
        assert!(d0 < d1);
        let startidx = self.index_after(d0);
        let endidx = self.index_before(d1);
        startidx..endidx
    }

    pub fn to_segment(&self) -> TrackSegment {
        let mut ret = TrackSegment::new();
        for wgs in &self.wgs84 {
            let w = gpx::Waypoint::new(geo::Point::new(wgs.0, wgs.1));
            ret.points.push(w);
        }
        ret
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
        let mut dacc = 0f64;
        for k in 0..segment.points.len() {
            let point = &segment.points[k];
            let (lon, lat) = point.point().x_y();
            // TODO: handle the case where there is no evelation data
            // => error message.
            wgs.push((lon, lat, point.elevation.unwrap()));
            debug_assert_eq!(wgs.len(), k + 1);
            let mut p = (lon.to_radians(), lat.to_radians());
            proj4rs::transform::transform(&wgs84, &utm32n, &mut p).unwrap();
            utm.push(UTMPoint(p.0, p.1));
            if k > 0 {
                dacc += distance(wgs[k - 1].0, wgs[k - 1].1, wgs[k].0, wgs[k].1);
            }
            dist.push(dacc);
        }
        assert_eq!(dist.len(), wgs.len());
        Track {
            wgs84: wgs,
            utm,
            _distance: dist,
        }
    }

    pub fn interesting_indexes(&self, epsilon: f64) -> Vec<usize> {
        let mut coords = Vec::new();
        for k in 0..self.len() {
            let x = self.distance(k);
            let y = self.elevation(k);
            coords.push(geo::coord!(x:x, y:y));
        }
        let line = geo::LineString::new(coords);
        line.simplify_idx(&epsilon)
    }
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
    pub fn from_track(track: &Track, range: &std::ops::Range<usize>) -> ProfileBoundingBox {
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
        self.xmax = ((self.xmax + margin) / shift).ceil() * shift;
        self.ymin = snap_floor(self.ymin - 100f64);
        self.ymax = snap_ceil(self.ymax + 100f64).max(snap_floor(self.ymin + 500f64));
    }
}

#[derive(Clone, PartialEq)]
pub enum WaypointOrigin {
    GPX,
    DouglasPeucker,
    MaxStepSize,
}

#[derive(Clone)]
pub struct Waypoint {
    pub wgs84: (f64, f64, f64),
    pub utm: UTMPoint,
    pub track_index: usize,
    pub origin: WaypointOrigin,
    pub name: Option<String>,
    pub description: Option<String>,
    pub step: Option<Step>,
    pub hide: bool,
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
        ret.push(Waypoint::from_gpx(
            w,
            UTMPoint(p.0, p.1),
            w.name.clone(),
            w.description.clone(),
        ));
    }
    ret
}

fn trim_option(s: Option<String>) -> Option<String> {
    match s {
        Some(data) => Some(String::from_str(data.trim()).unwrap()),
        _ => None,
    }
}

impl Waypoint {
    pub fn create(
        wgs: (f64, f64, f64),
        utm: UTMPoint,
        indx: usize,
        origin: WaypointOrigin,
    ) -> Waypoint {
        Waypoint {
            wgs84: wgs.clone(),
            utm: utm,
            track_index: indx,
            name: None,
            description: None,
            step: None,
            origin: origin,
            hide: false,
        }
    }

    pub fn from_gpx(
        gpx: &gpx::Waypoint,
        utm: UTMPoint,
        name: Option<String>,
        description: Option<String>,
    ) -> Waypoint {
        let (lon, lat) = gpx.point().x_y();
        let z = match gpx.elevation {
            Some(_z) => _z,
            _ => 0f64,
        };
        Waypoint {
            //wgs84: (lon, lat, gpx.elevation.unwrap()),
            wgs84: (lon, lat, z),
            utm: utm,
            track_index: usize::MAX,
            origin: WaypointOrigin::GPX,
            name: trim_option(name),
            description: trim_option(description),
            step: None,
            hide: false,
        }
    }
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
