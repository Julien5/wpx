use geo::SimplifyIdx;

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
use crate::tile::Chunks;
use crate::tile::Tiles;
use crate::track_projection::ProjectionTrees;
use crate::track_projection::Resolution;
use crate::track_projection::TrackProjection;

use super::elevation;

#[derive(Clone)]
pub struct Geometry {
    pub indices_xy: Vec<usize>,
    pub indices_z: Vec<usize>,
    pub xypoints: Vec<MercatorPoint>,
    deg: Vec<(f64, f64, f64)>, // distance, elevation, elevation gain
}

impl Geometry {
    pub fn distance(&self, index: usize) -> f64 {
        self.deg[index].0
    }

    pub fn elevation(&self, index: usize) -> f64 {
        self.deg[index].1
    }

    pub fn elevation_gain(&self, index: usize) -> f64 {
        self.deg[index].2
    }

    pub fn total_distance(&self) -> f64 {
        if self.deg.is_empty() {
            return 0f64;
        }
        self.distance(self.deg.len() - 1)
    }

    pub fn len(&self) -> usize {
        self.deg.len()
    }

    pub fn index_after(&self, distance: f64) -> usize {
        if distance < 0f64 {
            return 0;
        }
        let maxdist = self.total_distance();
        if distance > maxdist {
            return self.len();
        }
        let mut it = self.deg.iter();
        // positions stops on true
        it.position(|&d| d.0 >= distance).unwrap()
    }

    pub fn index_before(&self, distance: f64) -> usize {
        assert!(self.len() > 0);
        assert!(distance >= 0f64);
        let maxdist = self.total_distance();
        if distance >= maxdist {
            return self.len() - 1;
        }
        if distance <= 0f64 {
            return 0;
        }
        let mut it = self.deg.iter();
        match it.rposition(|&d| d.0 < distance) {
            Some(index) => index,
            None => {
                log::error!("no index_before distance {}", distance);
                0
            }
        }
    }

    fn point_at(
        &self,
        values: &Vec<(f64, f64, f64)>,
        get: impl Fn(&(f64, f64, f64)) -> f64,
        d: f64,
        k0: usize,
    ) -> f64 {
        assert!(!values.is_empty());
        if d <= 0.0 {
            return 0f64;
        }

        let last = values.last().unwrap();
        if d >= get(&last) {
            return (values.len() - 1) as f64;
        }

        // Binary search only in [k0..]
        let slice = &values[k0..];
        let k_local = slice.partition_point(|&dist| get(&dist) < d);
        let k = k0 + k_local;

        // k_local == 0 means d <= values[k0], so the point lies before k0
        // fall back to the segment ending at k0
        let (prev, next) = if k_local == 0 {
            let prev = if k0 > 0 { get(&values[k0 - 1]) } else { 0.0 };
            (prev, get(&values[k0]))
        } else {
            (get(&values[k - 1]), get(&values[k]))
        };

        let base = if k_local == 0 {
            k0.saturating_sub(1)
        } else {
            k - 1
        };
        let t = (d - prev) / (next - prev);
        base as f64 + t
    }

    pub fn point_at_distance(&self, d: f64, k0: usize) -> f64 {
        self.point_at(
            &self.deg,
            |values: &(f64, f64, f64)| -> f64 { values.0 },
            d,
            k0,
        )
    }

    pub fn point_at_elevation_gain(&self, d: f64, k0: usize) -> f64 {
        self.point_at(
            &self.deg,
            |values: &(f64, f64, f64)| -> f64 { values.2 },
            d,
            k0,
        )
    }

    pub fn copy(euclidean: &Vec<MercatorPoint>, distance: &Vec<f64>, elevation: &Vec<f64>) -> Self {
        let indices: Vec<_> = (0..euclidean.len()).collect();
        let mut de = Vec::new();
        debug_assert!(distance.len() == elevation.len());
        let mut last_elevation_gain = 0f64;
        for i in 0..distance.len() {
            let gain = if i == 0 {
                last_elevation_gain
            } else {
                let d = elevation[i] - elevation[i - 1];
                if d > 0.0 {
                    last_elevation_gain + d
                } else {
                    last_elevation_gain
                }
            };
            de.push((distance[i], elevation[i], gain));
            last_elevation_gain = gain;
        }

        Self {
            indices_xy: indices.clone(),
            xypoints: euclidean.clone(),
            indices_z: indices.clone(),
            deg: de,
        }
    }

    pub fn make_simplified(
        euclidean: &Vec<MercatorPoint>,
        distance: &Vec<f64>,
        smooth_elevation: &Vec<f64>,
    ) -> Self {
        let track_distance = distance.last().unwrap_or(&0f64);
        let indices_xy = {
            let coords: Vec<geo::Coord> = euclidean
                .iter()
                .map(|p| geo::coord!(x: p.x(), y: p.y()))
                .collect();
            let line = geo::LineString::new(coords);
            let epsilon = track_distance * 500f64 / 1200_000f64;
            line.simplify_idx(epsilon)
        };
        let xypoints = indices_xy
            .iter()
            .map(|idx| euclidean[*idx].clone())
            .collect();
        let indices_z: Vec<usize> = {
            let coords: Vec<geo::Coord> = smooth_elevation
                .iter()
                .enumerate()
                .map(|(idx, elevation)| geo::coord!(x: distance[idx], y: *elevation))
                .collect();
            let line = geo::LineString::new(coords);
            let epsilon = 2f64;
            line.simplify_idx(epsilon)
        };
        let mut de = Vec::new();
        let mut last_elevation_gain = 0f64;
        for i in 0..distance.len() {
            let gain = if i == 0 {
                last_elevation_gain
            } else {
                let d = smooth_elevation[i] - smooth_elevation[i - 1];
                if d > 0.0 {
                    last_elevation_gain + d
                } else {
                    last_elevation_gain
                }
            };
            de.push((distance[i], smooth_elevation[i], gain));
            last_elevation_gain = gain;
        }
        Self {
            indices_xy,
            xypoints,
            indices_z,
            deg: de,
        }
    }
}

#[derive(Clone)]
pub struct Track {
    pub wgs84: Vec<WGS84Point>,
    pub simplified: std::sync::Arc<Geometry>,
    pub geometry: std::sync::Arc<Geometry>,
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

    pub fn trees_parts(&self) -> Vec<TrackPart> {
        self.trees.parts()
    }

    pub fn boxes(&self, start: f64, end: f64) -> (Tiles, Chunks) {
        let range = self.subrange(start, end);
        let mut tiles = Tiles::new();
        let mut chunks = Chunks::new();
        let tiles_margin = 1000f64;
        let chunks_margin = 50_000f64;
        for k in range.start..range.end {
            let e = &self.geometry.xypoints[k];
            tiles.insert(tile::Tile::for_point(&e));
            tiles.insert(tile::Tile::for_point(&e.shift(0f64, tiles_margin)));
            tiles.insert(tile::Tile::for_point(&e.shift(0f64, -tiles_margin)));
            tiles.insert(tile::Tile::for_point(&e.shift(tiles_margin, 0f64)));
            tiles.insert(tile::Tile::for_point(&e.shift(-tiles_margin, 0f64)));

            chunks.insert(tile::Chunk::for_point(&e));
            chunks.insert(tile::Chunk::for_point(&e.shift(0f64, chunks_margin)));
            chunks.insert(tile::Chunk::for_point(&e.shift(0f64, -chunks_margin)));
            chunks.insert(tile::Chunk::for_point(&e.shift(chunks_margin, 0f64)));
            chunks.insert(tile::Chunk::for_point(&e.shift(-chunks_margin, 0f64)));
        }
        (tiles, chunks)
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
        assert!(!self.geometry.xypoints.is_empty());
        let mut ret = EuclideanBoundingBox::new();
        let _: Vec<_> = self
            .geometry
            .xypoints
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
        self.simplified.elevation_gain(index)
    }

    pub fn distance(&self, index: usize) -> f64 {
        self.geometry.distance(index)
    }

    pub fn total_distance(&self) -> f64 {
        self.geometry.total_distance()
    }

    pub fn index_after(&self, distance: f64) -> usize {
        self.geometry.index_after(distance)
    }
    pub fn index_before(&self, distance: f64) -> usize {
        self.geometry.index_before(distance)
    }

    pub fn subrange(&self, d0: f64, d1: f64) -> std::ops::Range<usize> {
        assert!(self.geometry.len() > 0);
        assert!(d0 < d1);
        let startidx = self.index_after(d0);
        // past the end
        let endidx = self.index_before(d1) + 1;
        assert!(endidx <= self.len());
        startidx..endidx
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

                    // Remove duplicates to ensure clean export/import:
                    // exporting duplicates ends of segments.
                    if last_point.is_some() && last_point.unwrap() == w {
                        continue;
                    }

                    euclidean.push(projection.project(&w));
                    wgs.push(w);
                    length += 1;

                    if last_point.is_some() {
                        let dloc = distance_wgs84(&last_point.unwrap(), &w);
                        dacc += dloc;
                    }
                    last_point = Some(w.clone());
                    _distance.push(dacc);
                }
            }
            parts.push(TrackPart {
                name: name.clone(),
                length,
                part_index: index,
            });
        }
        assert_eq!(_distance.len(), wgs.len());

        let smooth_elevation = elevation::smooth(
            200f64,
            wgs.len(),
            |index: usize| -> f64 { _distance[index] },
            |index: usize| -> f64 { wgs[index].z() },
        );
        assert_eq!(smooth_elevation.len(), wgs.len());

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

        // Compute simplified euclidean using Douglas-Peucker (for the map)

        let simplified = Geometry::make_simplified(&euclidean, &_distance, &smooth_elevation);

        let trees = match parts.len() > 1 {
            true => {
                log::trace!("making projection trees from parts");
                ProjectionTrees::make_from_parts(&euclidean, &simplified.xypoints, &parts)
            }
            false => {
                log::trace!("making appropriate projection trees");
                let parts = ProjectionTrees::make_parts(&euclidean, &Resolution::Topology);
                ProjectionTrees::make_from_parts(&euclidean, &simplified.xypoints, &parts)
            }
        };
        let elevation: Vec<_> = wgs.iter().map(|w| w.z()).collect();
        let geometry = Geometry::copy(&euclidean, &_distance, &elevation);
        let ret = Track {
            wgs84: wgs,
            simplified: std::sync::Arc::new(simplified),
            geometry: std::sync::Arc::new(geometry),
            parts,
            tiles: boxes,
            trees,
        };
        Ok(ret)
    }

    pub fn douglas_peucker_z(&self, epsilon: f64, range: &std::ops::Range<usize>) -> Vec<usize> {
        let mut coords = Vec::new();
        for k in range.start..range.end {
            let x = self.distance(k);
            //let y = self.elevation(k);
            let y = self.simplified.elevation(k);
            coords.push(geo::coord!(x:x, y:y));
        }
        let line = geo::LineString::new(coords);
        line.simplify_idx(epsilon)
            .iter()
            .map(|k| k + range.start)
            .collect::<Vec<_>>()
    }

    pub fn project_point(&self, point: &mut InputPoint) {
        self.trees.project(
            point,
            &self.geometry.xypoints,
            &|index| self.distance(index),
            &|index| self.elevation(index),
        );
    }

    pub fn project_graphics(&self, point: &MercatorPoint) -> TrackProjection {
        self.trees
            .project_graphics(point, &self.simplified.xypoints)
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
        let m_base = &self.geometry.xypoints[base].point2d();
        let m_next = if base + 1 >= self.geometry.xypoints.len() {
            m_base
        } else {
            &self.geometry.xypoints[base + 1].point2d()
        };
        let m = *m_base + (*m_next - *m_base) * t;

        let mercator = MercatorPoint::from_point2d(&m);

        let w_base = &self.wgs84[base].point2d();
        let w_next = if base + 1 >= self.geometry.xypoints.len() {
            w_base
        } else {
            &self.wgs84[base + 1].point2d()
        };
        let w = *w_base + (*w_next - *w_base) * t;

        let e_base = self.wgs84[base].z();
        let e_next = self.wgs84[base].z();
        let e = e_base + (e_next - e_base) * t;

        let d_base = self.distance(base);
        let d_next = self.distance(base);
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

    pub fn point_at_distance(&self, d: f64, k0: usize) -> (WGS84Point, TrackProjection) {
        let f = self.geometry.point_at_distance(d, k0);
        self.projection_at_track_floating_index(f)
    }
    pub fn point_at_elevation_gain(&self, d: f64, k0: usize) -> (WGS84Point, TrackProjection) {
        let f = self.simplified.point_at_elevation_gain(d, k0);
        self.projection_at_track_floating_index(f)
    }
}
