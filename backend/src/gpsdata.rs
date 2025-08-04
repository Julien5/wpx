use crate::error::Error;
use crate::project;
use crate::utm::UTMPoint;
use crate::waypoint::Waypoint;
use crate::waypoint::WaypointOrigin;
use geo::{Distance, SimplifyIdx};

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let p1 = geo::Point::new(x1, y1);
    let p2 = geo::Point::new(x2, y2);
    geo::Haversine::distance(p1, p2)
}

use gpx::TrackSegment;

use crate::elevation;

pub struct Track {
    pub wgs84: Vec<(f64, f64, f64)>,
    pub utm: Vec<UTMPoint>,
    _distance: Vec<f64>,
}

pub fn read_gpx_content(bytes: &Vec<u8>) -> Result<gpx::Gpx, Error> {
    let reader_mem = std::io::Cursor::new(bytes);
    match gpx::read(reader_mem) {
        Ok(d) => Ok(d),
        Err(_e) => Err(Error::GPXInvalid),
    }
}

pub fn read_segment(gpx: &mut gpx::Gpx) -> Result<gpx::TrackSegment, Error> {
    let mut t0 = gpx.tracks.swap_remove(0);
    if t0.segments.is_empty() {
        return Err(Error::GPXHasNoSegment);
    }
    let s0 = t0.segments.swap_remove(0);
    Ok(s0)
}

impl Track {
    pub fn create_on_track(&self, index: usize, origin: WaypointOrigin) -> Waypoint {
        Waypoint {
            wgs84: self.wgs84[index].clone(),
            utm: self.utm[index].clone(),
            track_index: Some(index),
            name: None,
            description: None,
            info: None,
            origin,
        }
    }
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

    pub fn from_segment(segment: &TrackSegment) -> Result<Track, Error> {
        let mut _distance = Vec::new();

        // see https://en.wikipedia.org/wiki/Universal_Transverse_Mercator_coordinate_system
        // we take the first point of each segment
        // we should wait until we have the user segments (pages) to ensure the same
        // zone for a minimap.
        let zone = match segment.points.is_empty() {
            true => 32i32,
            false => {
                let long = segment.points[0].point().x() as f64;
                (((long + 180f64) / 6f64).floor() + 1f64) as i32
            }
        };

        use proj4rs::proj::Proj;
        let spec = format!(
            "+proj=utm +zone={} +datum=WGS84 +units=m +no_defs +type=crs",
            zone
        );
        let utm_spec = Proj::from_proj_string(spec.as_str()).unwrap();

        let spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
        let wgs84_spec = Proj::from_proj_string(spec).unwrap();
        let mut utm = Vec::new();
        let mut wgs = Vec::new();
        let mut dacc = 0f64;
        for k in 0..segment.points.len() {
            let point = &segment.points[k];
            let (lon, lat) = point.point().x_y();
            let elevation = match point.elevation {
                Some(e) => e,
                None => {
                    return Err(Error::MissingElevation { index: k });
                }
            };

            wgs.push((lon, lat, elevation));
            debug_assert_eq!(wgs.len(), k + 1);
            let mut p = (lon.to_radians(), lat.to_radians());
            proj4rs::transform::transform(&wgs84_spec, &utm_spec, &mut p).unwrap();
            utm.push(UTMPoint(p.0, p.1));
            if k > 0 {
                dacc += distance(wgs[k - 1].0, wgs[k - 1].1, wgs[k].0, wgs[k].1);
            }
            _distance.push(dacc);
        }
        assert_eq!(_distance.len(), wgs.len());
        let ret = Track {
            wgs84: wgs,
            utm,
            _distance,
        };
        Ok(ret)
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

pub fn project_waypoints(track: &Track, waypoints: &mut Vec<Waypoint>) {
    let indexes = project::nearest_neighboor(&track.utm, &waypoints);
    debug_assert_eq!(waypoints.len(), indexes.len());
    for k in 0..indexes.len() {
        waypoints[k].track_index = Some(indexes[k]);
    }
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
