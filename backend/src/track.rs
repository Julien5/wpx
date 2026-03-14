use std::collections::BTreeSet;

use geo::SimplifyIdx;
use gpx::TrackSegment;

use super::wgs84point::WGS84Point;
use crate::error::TrackError;
use crate::gpsdata::distance_wgs84;
use crate::inputpoint::InputPoint;
use crate::inputpoint::InputPointMap;
use crate::mercator;
use crate::mercator::EuclideanBoundingBox;
use crate::mercator::MercatorPoint;
use crate::parameters::TrackPart;
use crate::tile;
use crate::tile::Tiles;
use crate::track_projection::ProjectionTrees;
use crate::track_projection::TrackProjection;

use super::elevation;

pub struct Simplified {
    pub xy: Vec<usize>,
    pub xypoints: Vec<MercatorPoint>,
    pub dz: Vec<usize>,
}

impl Simplified {
    pub fn make(
        euclidean: &Vec<MercatorPoint>,
        distance: &Vec<f64>,
        smooth_elevation: &Vec<f64>,
    ) -> Self {
        let track_distance = distance.last().unwrap_or(&0f64);
        let xy = {
            let coords: Vec<geo::Coord> = euclidean
                .iter()
                .map(|p| geo::coord!(x: p.x(), y: p.y()))
                .collect();
            let line = geo::LineString::new(coords);
            let epsilon = track_distance * 500f64 / 1200_000f64;
            line.simplify_idx(&epsilon)
        };
        let xypoints = xy.iter().map(|idx| euclidean[*idx].clone()).collect();
        let dz = {
            let coords: Vec<geo::Coord> = smooth_elevation
                .iter()
                .enumerate()
                .map(|(idx, elevation)| geo::coord!(x: distance[idx], y: *elevation))
                .collect();
            let line = geo::LineString::new(coords);
            let epsilon = 2f64;
            line.simplify_idx(&epsilon)
        };
        Self { xy, xypoints, dz }
    }
}

pub struct Track {
    pub wgs84: Vec<WGS84Point>,
    pub smooth_elevation: Vec<f64>,
    pub smooth_elevation_gain: Vec<f64>,
    pub euclidean: Vec<MercatorPoint>,
    pub simplified: Simplified,
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

    pub fn tiles(&self, start: f64, end: f64) -> Tiles {
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

    pub fn from_tracks(gpxtracks: &Vec<(String, gpx::Track)>) -> Result<Track, TrackError> {
        let mut _distance = Vec::new();
        let mut wgs = Vec::new();
        let mut dacc = 0f64;
        let projection = mercator::WebMercatorProjection::make();
        let mut euclidean = Vec::new();
        let mut parts = Vec::new();

        let mut last_point = None;
        for (index, (name, track)) in gpxtracks.iter().enumerate() {
            debug_assert_eq!(track.segments.len(), 1);
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
                    euclidean.push(projection.project(&w));
                    wgs.push(w);

                    if last_point.is_some() {
                        let dloc = distance_wgs84(&last_point.unwrap(), &w);
                        if dloc > 1000f64 {
                            log::trace!("name={} k={} dloc={}", name, k, dloc);
                        }
                        dacc += dloc;
                    }
                    last_point = Some(w.clone());
                    _distance.push(dacc);
                }
            }
            parts.push(TrackPart {
                name: name.clone(),
                length: track.segments.first().unwrap().points.len(),
                part_index: index,
            });
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

        // Compute simplified euclidean using Douglas-Peucker
        let simplified = Simplified::make(&euclidean, &_distance, &track_smooth_elevation);

        let trees = ProjectionTrees::make(&euclidean, &simplified.xypoints);

        let ret = Track {
            wgs84: wgs,
            euclidean,
            simplified,
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

    pub fn project_simplified(&self, point: &MercatorPoint) -> TrackProjection {
        self.trees.simple_project(point, &self.simplified.xypoints)
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

    pub fn projection_at_track_floating_index(
        &self,
        track_floating_index: f64,
    ) -> (WGS84Point, TrackProjection) {
        let base = track_floating_index.floor() as usize;
        let track_index = track_floating_index.round() as usize;
        let t = track_floating_index - track_floating_index.floor();

        assert!(t < 1.0);
        let m_base = &self.euclidean[base].point2d();
        let m_next = &self.euclidean[base + 1].point2d();
        let m = *m_base + (*m_next - *m_base) * t;

        let mercator = MercatorPoint::from_point2d(&m);

        let w_base = &self.wgs84[base].point2d();
        let w_next = &self.wgs84[base + 1].point2d();
        let w = *w_base + (*w_next - *w_base) * t;

        let e_base = self.wgs84[base].z();
        let e_next = self.wgs84[base].z();
        let e = e_base + (e_next - e_base) * t;

        let d_base = self._distance[base];
        let d_next = self._distance[base];
        let d = d_base + (d_next - d_base) * t;

        let proj = TrackProjection {
            track_floating_index,
            track_index,
            euclidean: mercator,
            elevation: e,
            track_distance: 0f64,
            distance_on_track_to_projection: d,
        };

        let wgs = WGS84Point::new(&w.x, &w.y, &e);
        (wgs, proj)
    }

    fn point_at(&self, values: &Vec<f64>, d: f64, k0: usize) -> (WGS84Point, TrackProjection) {
        assert!(!values.is_empty());
        if d <= 0.0 {
            return self.projection_at_track_floating_index(0f64);
        }
        let last = *values.last().unwrap();
        if d >= last {
            let ret = (values.len() - 1) as f64;
            return self.projection_at_track_floating_index(ret);
        }

        // Binary search only in [k0..]
        let slice = &values[k0..];
        let k_local = slice.partition_point(|&dist| dist < d);
        let k = k0 + k_local;

        // k_local == 0 means d <= values[k0], so the point lies before k0
        // fall back to the segment ending at k0
        let (prev, next) = if k_local == 0 {
            let prev = if k0 > 0 { values[k0 - 1] } else { 0.0 };
            (prev, values[k0])
        } else {
            (values[k - 1], values[k])
        };

        let base = if k_local == 0 {
            k0.saturating_sub(1)
        } else {
            k - 1
        };
        let t = (d - prev) / (next - prev);
        let track_floating_index = base as f64 + t;
        self.projection_at_track_floating_index(track_floating_index)
    }

    pub fn point_at_distance(&self, d: f64, k0: usize) -> (WGS84Point, TrackProjection) {
        self.point_at(&self._distance, d, k0)
    }
    pub fn point_at_elevation_gain(&self, d: f64, k0: usize) -> (WGS84Point, TrackProjection) {
        self.point_at(&self.smooth_elevation_gain, d, k0)
    }
}
