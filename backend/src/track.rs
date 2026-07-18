use super::wgs84point::WGS84Point;
use crate::error::TrackError;
use crate::geometry::mapgeometry::MapGeometry;
use crate::geometry::profilegeometry::ProfileGeometry;
use crate::gpsdata::distance_wgs84;
use crate::inputpoint::InputPoint;
use crate::inputpoint::InputPointMap;
use crate::mercator;
use crate::mercator::EuclideanBoundingBox;
use crate::mercator::MercatorPoint;
use crate::parameters::PowerParameters;
use crate::tile;
use crate::tile::Chunks;
use crate::tile::Tiles;
use crate::track_projection::ProjectionTrees;
use crate::track_projection::Resolution;
use crate::track_projection::TrackProjection;

use crate::geometry::powergeometry::ConstantPowerGeometry;
use crate::trackparts::ProtoTrack;

#[derive(Clone)]
pub struct Track {
    pub wgs84: Vec<WGS84Point>,
    pub map: MapGeometry,
    pub profile: ProfileGeometry,
    pub tiles: Tiles,
    pub name: String,
    trees: ProjectionTrees,
}

pub type SharedTrack = std::sync::Arc<Track>;

pub type WGS84BoundingBox = super::bbox::BoundingBox;

impl Track {
    pub fn len(&self) -> usize {
        self.wgs84.len()
    }

    pub fn ranges(&self) -> Vec<std::range::Range<usize>> {
        self.trees.ranges()
    }

    pub fn boxes(&self, start: f64, end: f64) -> (Tiles, Chunks) {
        let range = self.subrange(start, end);
        let mut tiles = Tiles::new();
        let mut chunks = Chunks::new();
        let tiles_margin = 1000f64;
        let chunks_margin = 50_000f64;
        for k in range.start..range.end {
            let e = self.map.point_at(k);
            tiles.insert(tile::Tile::for_point(e));
            tiles.insert(tile::Tile::for_point(&e.shift(0f64, tiles_margin)));
            tiles.insert(tile::Tile::for_point(&e.shift(0f64, -tiles_margin)));
            tiles.insert(tile::Tile::for_point(&e.shift(tiles_margin, 0f64)));
            tiles.insert(tile::Tile::for_point(&e.shift(-tiles_margin, 0f64)));

            chunks.insert(tile::Chunk::for_point(e));
            chunks.insert(tile::Chunk::for_point(&e.shift(0f64, chunks_margin)));
            chunks.insert(tile::Chunk::for_point(&e.shift(0f64, -chunks_margin)));
            chunks.insert(tile::Chunk::for_point(&e.shift(chunks_margin, 0f64)));
            chunks.insert(tile::Chunk::for_point(&e.shift(-chunks_margin, 0f64)));
        }
        (tiles, chunks)
    }

    pub fn wgs84_bounding_box(&self) -> WGS84BoundingBox {
        debug_assert!(!self.wgs84.is_empty());
        let mut ret = WGS84BoundingBox::new();
        for p in &self.wgs84 {
            ret.update(&p.point2d());
        }
        ret
    }

    pub fn euclidean_bounding_box(&self) -> EuclideanBoundingBox {
        self.map.bounding_box()
    }

    pub fn elevation(&self, index: usize) -> f64 {
        self.wgs84[index].z()
    }

    pub fn elevation_gain_on_range(&self, range: &std::range::Range<usize>) -> f64 {
        self.profile.gain_on_range(range)
    }

    pub fn elevation_gain(&self, index: usize) -> f64 {
        self.profile.elevation_gain(index)
    }

    pub fn distance(&self, index: usize) -> f64 {
        self.profile.distance(index)
    }

    pub fn total_distance(&self) -> f64 {
        self.profile.total_distance()
    }

    pub fn subrange(&self, d0: f64, d1: f64) -> std::range::Range<usize> {
        debug_assert!(self.profile.len() > 0);
        debug_assert!(d0 < d1);
        let startidx = self.profile.index_after(d0);
        let endidx = self.profile.index_before(d1) + 1;
        debug_assert!(endidx <= self.len());
        (startidx..endidx).into()
    }

    pub fn from_proto(proto: &ProtoTrack) -> Result<Track, TrackError> {
        let mut _distance = Vec::new();
        let mut dacc = 0f64;
        let projection = mercator::WebMercatorProjection::make();
        let mut euclidean = Vec::new();
        let mut last_point = None;
        for w in &proto.wgs84 {
            if last_point.is_some() && distance_wgs84(&last_point.unwrap(), &w) == 0f64 {
                // should have been filtered in proto
                debug_assert!(false);
                continue;
            }

            euclidean.push(projection.project(&w));
            if last_point.is_some() {
                let dloc = distance_wgs84(&last_point.unwrap(), &w);
                dacc += dloc;
            }
            last_point = Some(w.clone());
            _distance.push(dacc);
        }
        debug_assert_eq!(_distance.len(), proto.wgs84.len());

        let mut boxes = Tiles::new();
        for e in &euclidean {
            boxes.insert(tile::Tile::for_point(e));
        }
        for b in boxes.clone() {
            for n in tile::neighbors(&b) {
                boxes.insert(n);
            }
        }

        let track_distance = _distance.last().copied().unwrap_or(0.0);

        let map = MapGeometry::new(&euclidean, track_distance);
        let profile = ProfileGeometry::new(_distance.clone(), &|index: usize| -> f64 {
            proto.wgs84[index].z()
        });

        let trees = {
            let ranges = ProjectionTrees::make_parts(&euclidean, &Resolution::Topology);
            ProjectionTrees::make_from_parts(&euclidean, &ranges)
        };

        let ret = Track {
            wgs84: proto.wgs84.clone(),
            name: proto.name(),
            map,
            profile,
            tiles: boxes,
            trees,
        };
        Ok(ret)
    }

    pub fn project_point(&self, point: &mut InputPoint) {
        self.trees.project(
            point,
            self.map.all_points(),
            &|index| self.profile.distance(index),
            &|index| self.wgs84[index].z(),
        );
    }

    pub fn project_map(&self, map: &mut InputPointMap) {
        for tile in &self.tiles {
            if map.get_mut(tile).is_none() {
                continue;
            }
            let points = map.get_mut(tile).unwrap();
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
        let t = track_floating_index - track_floating_index.floor();

        debug_assert!(t < 1.0);
        let m_base = &self.map.point_at(base).point2d();
        let m_next = if base + 1 >= self.map.len() {
            m_base
        } else {
            &self.map.point_at(base + 1).point2d()
        };
        let m = *m_base + (*m_next - *m_base) * t;

        let mercator = MercatorPoint::from_point2d(&m);

        let w_base = &self.wgs84[base].point2d();
        let w_next = if base + 1 >= self.map.len() {
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
            euclidean: mercator,
            elevation: e,
            track_distance: 0f64,
            distance_on_track_to_projection: d,
        };

        let wgs = WGS84Point::new(&w.x, &w.y, &e);
        (wgs, proj)
    }

    pub fn point_at_distance(&self, d: f64, k0: usize) -> (WGS84Point, TrackProjection) {
        let f = self.profile.point_at_distance(d, k0);
        self.projection_at_track_floating_index(f)
    }

    pub fn point_at_elevation_gain(&self, d: f64, k0: usize) -> (WGS84Point, TrackProjection) {
        let f = self.profile.point_at_elevation_gain(d, k0);
        self.projection_at_track_floating_index(f)
    }

    pub fn make_power_geometry(
        &self,
        power_parameters: &PowerParameters,
        waypoints: &[InputPoint],
    ) -> ConstantPowerGeometry {
        let distances: Vec<f64> = (0..self.len()).map(|i| self.profile.distance(i)).collect();
        let elevations: Vec<f64> = self.wgs84.iter().map(|w| w.z()).collect();
        ConstantPowerGeometry::new(power_parameters, &distances, &elevations, waypoints)
    }
}
