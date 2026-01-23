use std::collections::BTreeSet;

use geo::SimplifyIdx;
use gpx::TrackSegment;

use super::wgs84point::WGS84Point;
use crate::error;
use crate::gpsdata::distance_wgs84;
use crate::inputpoint::InputPoint;
use crate::inputpoint::InputPointMap;
use crate::mercator;
use crate::mercator::EuclideanBoundingBox;
use crate::mercator::MercatorPoint;
use crate::tile;
use crate::tile::Tiles;
use crate::track_projection::ProjectionTrees;

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
    pub euclidean: Vec<MercatorPoint>,
    _distance: Vec<f64>,
    pub parts: Vec<TrackPart>,
    pub tiles: Tiles,
    trees: ProjectionTrees,
}

pub type SharedTrack = std::sync::Arc<Track>;

// (long,lat)
pub type WGS84BoundingBox = super::bbox::BoundingBox;

impl Track {
    pub fn len(&self) -> usize {
        self.wgs84.len()
    }

    pub fn subboxes(&self, start: f64, end: f64) -> Tiles {
        let range = self.subrange(start, end);
        let mut boxes = BTreeSet::new();
        for k in range.start..range.end {
            let e = &self.euclidean[k];
            boxes.insert(tile::Tile::for_point(&e));
        }
        // we need to enlarge to make sure we dont miss points that are close to the track,
        // but not in a box on the track.
        for b in boxes.clone() {
            for n in tile::neighbors(&b) {
                boxes.insert(n);
            }
        }
        boxes
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
        assert!(!self.euclidean.is_empty());
        let mut ret = EuclideanBoundingBox::new();
        let _: Vec<_> = self
            .euclidean
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
        assert!(range.end <= self.len());
        assert!(range.start < self.len());
        return self.elevation_gain(range.end - 1) - self.elevation_gain(range.start);
    }

    pub fn elevation_gain(&self, index: usize) -> f64 {
        assert_eq!(self.smooth_elevation_gain.len(), self.len());
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
        assert!(self.len() > 0);
        assert!(distance >= 0f64);
        let maxdist = self.total_distance();
        let end = self.len();
        if distance >= maxdist {
            return end - 1;
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

    pub fn subrange(&self, d0: f64, d1: f64) -> std::ops::Range<usize> {
        assert!(!self._distance.is_empty());
        assert!(d0 < d1);
        let startidx = self.index_after(d0);
        // past the end
        let endidx = self.index_before(d1) + 1;
        assert!(endidx <= self.len());
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
        assert_eq!(ret.len(), smooth_elevation.len());
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
        assert_eq!(track_smooth_elevation.len(), wgs.len());

        let smooth_elevation_gain = Self::compute_elevation_gain(&track_smooth_elevation);

        let mut boxes = Tiles::new();
        for e in &euclidean {
            boxes.insert(tile::Tile::for_point(&e));
        }
        // we need to enlarge to make sure we dont miss points that are close to the track,
        // but not in a box on the track.
        for b in boxes.clone() {
            for n in tile::neighbors(&b) {
                boxes.insert(n);
            }
        }

        let trees = ProjectionTrees::make(&euclidean);
        let ret = Track {
            wgs84: wgs,
            euclidean,
            smooth_elevation: track_smooth_elevation,
            smooth_elevation_gain,
            _distance,
            parts,
            tiles: boxes,
            trees,
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

    pub fn project_point(&self, point: &mut InputPoint) {
        self.trees.project(
            point,
            &self.euclidean,
            &|index| self.distance(index),
            &|index| self.elevation(index),
        );
    }

    pub fn project_map(&self, map: &mut InputPointMap) {
        for tile in &self.tiles {
            if map.get_mut(&tile).is_none() {
                continue;
            }
            let points = map.get_mut(&tile).unwrap();
            for mut point in points {
                self.project_point(&mut point);
            }
        }
    }
}
