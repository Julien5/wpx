use geo::SimplifyIdx;
use gpx::TrackSegment;

use crate::error;
use crate::gpsdata::distance_wgs84;

use super::elevation;
use super::utm::UTMPoint;
use super::waypoint::Waypoint;
use super::waypoint::WaypointOrigin;

pub struct Track {
    pub wgs84: Vec<(f64, f64, f64)>,
    pub utm: Vec<UTMPoint>,
    _distance: Vec<f64>,
}

// (long,lat)
pub type WGS84BoundingBox = super::bbox::BoundingBox;

pub fn _osm(bbox: &WGS84BoundingBox) -> String {
    format!(
        "({}, {},{},{})",
        bbox.min.1, bbox.min.0, bbox.max.1, bbox.max.0
    )
}

pub fn osm3(bbox: &WGS84BoundingBox) -> String {
    format!(
        "({:0.3},{:0.3},{:0.3},{:0.3})",
        bbox.min.1, bbox.min.0, bbox.max.1, bbox.max.0
    )
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
    pub fn wgs84_bounding_box(&self) -> WGS84BoundingBox {
        let mut ret = WGS84BoundingBox::new();
        let _: Vec<_> = self
            .wgs84
            .iter()
            .map(|p| {
                ret.update(&(p.0, p.1));
            })
            .collect();
        ret
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

    pub fn from_segment(segment: &TrackSegment) -> Result<Track, error::Error> {
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
                    return Err(error::Error::MissingElevation { index: k });
                }
            };

            wgs.push((lon, lat, elevation));
            debug_assert_eq!(wgs.len(), k + 1);
            let mut p = (lon.to_radians(), lat.to_radians());
            proj4rs::transform::transform(&wgs84_spec, &utm_spec, &mut p).unwrap();
            utm.push(UTMPoint(p.0, p.1));
            if k > 0 {
                dacc += distance_wgs84(wgs[k - 1].0, wgs[k - 1].1, wgs[k].0, wgs[k].1);
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
