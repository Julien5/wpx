use std::collections::BTreeSet;

use geo::SimplifyIdx;
use gpx::TrackSegment;

use super::wgs84point::WGS84Point;
use crate::bbox::BoundingBox;
use crate::bboxes;
use crate::error;
use crate::gpsdata::distance_wgs84;
use crate::mercator;
use crate::mercator::EuclideanBoundingBox;
use crate::mercator::MercatorPoint;

use super::elevation;

pub struct TrackPart {
    pub name: String,
    pub begin: usize,
    pub end: usize,
}

pub struct Track {
    pub wgs84: Vec<WGS84Point>,
    pub smooth_elevation: Vec<f64>,
    pub smooth_elevation_gain: Vec<f64>,
    pub euclidian: Vec<MercatorPoint>,
    _distance: Vec<f64>,
    pub parts: Vec<TrackPart>,
    pub boxes: BTreeSet<BoundingBox>,
}

// (long,lat)
pub type WGS84BoundingBox = super::bbox::BoundingBox;

impl Track {
    pub fn len(&self) -> usize {
        self.wgs84.len()
    }

    pub fn wgs84_bounding_box(&self) -> WGS84BoundingBox {
        assert!(!self.wgs84.is_empty());
        let mut ret = WGS84BoundingBox::new();
        let _: Vec<_> = self
            .wgs84
            .iter()
            .map(|p| {
                ret.update(&p.point2d());
            })
            .collect();
        ret
    }

    pub fn euclidean_bounding_box(&self) -> EuclideanBoundingBox {
        assert!(!self.euclidian.is_empty());
        let mut ret = EuclideanBoundingBox::new();
        let _: Vec<_> = self
            .euclidian
            .iter()
            .map(|p| {
                ret.update(&p.point2d());
            })
            .collect();
        ret
    }

    pub fn elevation(&self, index: usize) -> f64 {
        self.wgs84[index].z()
    }

    pub fn elevation_gain_on_range(&self, range: &std::ops::Range<usize>) -> f64 {
        return self.elevation_gain(range.end - 1) - self.elevation_gain(range.start);
    }

    pub fn elevation_gain(&self, index: usize) -> f64 {
        self.smooth_elevation_gain[index]
    }

    pub fn distance(&self, index: usize) -> f64 {
        self._distance[index]
    }

    pub fn total_distance(&self) -> f64 {
        match self._distance.last() {
            Some(d) => *d,
            None => 0.0,
        }
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

    pub fn export_to_gpx(&self) -> TrackSegment {
        let mut ret = TrackSegment::new();
        for wgs in &self.wgs84 {
            // remove z coordinate to avoid automatic "low" and "hight points" on etrex 10
            let w = gpx::Waypoint::new(geo::Point::new(wgs.x(), wgs.y()));
            ret.points.push(w);
        }
        ret
    }

    fn compute_elevation_gain(smooth_elevation: &Vec<f64>) -> Vec<f64> {
        let mut ret = vec![0f64; smooth_elevation.len()];
        let range = std::ops::Range {
            start: 0,
            end: smooth_elevation.len(),
        };
        for k in range.start + 1..range.end {
            let d = smooth_elevation[k] - smooth_elevation[k - 1];
            if d > 0.0 {
                ret[k] = ret[k - 1] + d;
            } else {
                ret[k] = ret[k - 1];
            }
        }
        ret
    }

    pub fn from_tracks(gpxtracks: &Vec<gpx::Track>) -> Result<Track, error::Error> {
        let mut _distance = Vec::new();
        let mut wgs = Vec::new();
        let mut dacc = 0f64;
        let projection = mercator::WebMercatorProjection::make();
        let mut euclidean = Vec::new();
        let mut parts = Vec::new();

        for track in gpxtracks {
            assert_eq!(track.segments.len(), 1);
            let begin = wgs.len();
            for segment in &track.segments {
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
            }
            let name = track.name.as_ref().unwrap_or(&String::new()).clone();
            let part = TrackPart {
                name,
                begin,
                end: wgs.len(),
            };
            parts.push(part);
        }
        assert_eq!(_distance.len(), wgs.len());

        let track_smooth_elevation = elevation::smooth(
            200f64,
            wgs.len(),
            |index: usize| -> f64 { _distance[index] },
            |index: usize| -> f64 { wgs[index].z() },
        );

        let smooth_elevation_gain = Self::compute_elevation_gain(&track_smooth_elevation);

        let mut boxes: BTreeSet<BoundingBox> = BTreeSet::new();
        log::trace!("building boxes..");
        for e in &euclidean {
            boxes.insert(bboxes::pointbox(&e));
        }
        // we need to enlarge to make sure we dont miss points that are close to the track,
        // but not in a box on the track.
        for b in boxes.clone() {
            for n in bboxes::neighbors(&b) {
                boxes.insert(n);
            }
        }

        log::trace!("built {} boxes", boxes.len());

        let ret = Track {
            wgs84: wgs,
            euclidian: euclidean,
            smooth_elevation: track_smooth_elevation,
            smooth_elevation_gain,
            _distance,
            parts,
            boxes,
        };
        Ok(ret)
    }

    pub fn douglas_peucker(&self, epsilon: f64, range: &std::ops::Range<usize>) -> Vec<usize> {
        let mut coords = Vec::new();
        for k in range.start..range.end {
            let x = self.distance(k);
            //let y = self.elevation(k);
            let y = self.smooth_elevation[k];
            coords.push(geo::coord!(x:x, y:y));
        }
        let line = geo::LineString::new(coords);
        line.simplify_idx(&epsilon)
            .iter()
            .map(|k| k + range.start)
            .collect::<Vec<_>>()
    }
}
