use geo::SimplifyIdx;
use gpx::TrackSegment;

use super::wgs84point::WGS84Point;
use crate::error;
use crate::gpsdata::distance_wgs84;
use crate::mercator;
use crate::mercator::EuclideanBoundingBox;
use crate::mercator::MercatorPoint;

use super::elevation;
use super::waypoint::Waypoint;
use super::waypoint::WaypointOrigin;

pub struct Track {
    pub wgs84: Vec<WGS84Point>,
    pub euclidian: Vec<MercatorPoint>,
    _distance: Vec<f64>,
}

// (long,lat)
pub type WGS84BoundingBox = super::bbox::BoundingBox;

impl Track {
    pub fn create_on_track(&self, index: usize, origin: WaypointOrigin) -> Waypoint {
        Waypoint {
            wgs84: self.wgs84[index].clone(),
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
                ret.update(&(p.x(), p.y()));
            })
            .collect();
        ret
    }
    pub fn euclidean_bounding_box(&self) -> EuclideanBoundingBox {
        let mut ret = EuclideanBoundingBox::new();
        let _: Vec<_> = self
            .euclidian
            .iter()
            .map(|p| {
                ret.update(&(p.x(), p.y()));
            })
            .collect();
        ret
    }
    pub fn elevation(&self, index: usize) -> f64 {
        self.wgs84[index].z()
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
        match it.rposition(|&d| d < distance) {
            Some(index) => index,
            None => {
                log::error!("no index_before distance {}", distance);
                0
            }
        }
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
            let w = gpx::Waypoint::new(geo::Point::new(wgs.x(), wgs.y()));
            ret.points.push(w);
        }
        ret
    }

    pub fn from_segment(segment: &TrackSegment) -> Result<Track, error::Error> {
        let mut _distance = Vec::new();
        let mut wgs = Vec::new();
        let mut dacc = 0f64;
        let projection = mercator::WebMercatorProjection::make();
        let mut euclidean = Vec::new();
        for k in 0..segment.points.len() {
            let point = &segment.points[k];
            let (lon, lat) = point.point().x_y();
            let elevation = match point.elevation {
                Some(e) => e,
                None => {
                    return Err(error::Error::MissingElevation { index: k });
                }
            };

            let w = WGS84Point::new(&lon, &lat, &elevation);
            euclidean.push(projection.project(&w));
            wgs.push(w);

            if k > 0 {
                dacc += distance_wgs84(&wgs[k - 1], &wgs[k]);
            }
            _distance.push(dacc);
        }
        assert_eq!(_distance.len(), wgs.len());
        let ret = Track {
            wgs84: wgs,
            euclidian: euclidean,
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
