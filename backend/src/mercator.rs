use crate::{math::Point2D, track::WGS84BoundingBox, wgs84point::WGS84Point};
use serde::{Deserialize, Serialize};

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct MercatorPoint(pub f64, pub f64);

impl MercatorPoint {
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
    pub fn xy(&self) -> (f64, f64) {
        (self.0, self.1)
    }
    pub fn from_xy((x, y): &(f64, f64)) -> MercatorPoint {
        MercatorPoint(*x, *y)
    }
    pub fn point2d(&self) -> Point2D {
        Point2D::new(self.x(), self.y())
    }
    pub fn from_point2d(p: &Point2D) -> MercatorPoint {
        MercatorPoint(p.x, p.y)
    }
    pub fn d2(&self, other: &MercatorPoint) -> f64 {
        let dx = self.0 - other.0;
        let dy = self.1 - other.1;
        dx * dx + dy * dy
    }
}

pub struct WebMercatorProjection {
    wgs84_spec: proj4rs::proj::Proj,
    dst_spec: proj4rs::proj::Proj,
}

impl WebMercatorProjection {
    pub fn make() -> WebMercatorProjection {
        // The PROJ.4 parameters for EPSG:3857 (also known as Web Mercator or Pseudo-Mercator) are:
        // +proj=merc +lon_0=0 +k=1 +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs
        // https://gis.stackexchange.com/questions/159572/proj4-for-epsg3857
        use proj4rs::proj::Proj;
        let spec = format!(
			"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0 +k=1.0 +units=m +nadgrids=@null +wktext  +no_defs"
		);
        let dst_spec = Proj::from_proj_string(spec.as_str()).unwrap();

        let spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
        let wgs84_spec = Proj::from_proj_string(spec).unwrap();
        WebMercatorProjection {
            wgs84_spec,
            dst_spec,
        }
    }
    pub fn project(&self, wgs84: &WGS84Point) -> MercatorPoint {
        let mut p = (
            wgs84.longitude().to_radians(),
            wgs84.latitude().to_radians(),
        );
        proj4rs::transform::transform(&self.wgs84_spec, &self.dst_spec, &mut p).unwrap();
        MercatorPoint(p.0, p.1)
    }
    pub fn unproject(&self, merc: &MercatorPoint) -> WGS84Point {
        let mut p = (merc.0, merc.1);
        proj4rs::transform::transform(&self.dst_spec, &self.wgs84_spec, &mut p).unwrap();
        let ret = WGS84Point::new(&p.0.to_degrees(), &p.1.to_degrees(), &0f64);
        log::trace!("unproject {:?} to {:?}", merc, ret);
        ret
    }
}

pub type EuclideanBoundingBox = super::bbox::BoundingBox;

impl EuclideanBoundingBox {
    pub fn unproject(&self) -> WGS84BoundingBox {
        let proj = WebMercatorProjection::make();
        let min = proj.unproject(&MercatorPoint::from_point2d(&self.get_min()));
        let max = proj.unproject(&MercatorPoint::from_point2d(&self.get_max()));
        WGS84BoundingBox::minmax(min.point2d(), max.point2d())
    }
}

#[cfg(test)]
mod tests {
    fn approx(f: Point2D) -> String {
        format!("{:.4},{:.4}", f.x, f.y)
    }

    use super::*;
    #[test]
    fn project_unproject() {
        // Mercator=EPSG:3857
        // WGS84=EPSG:4326
        //let euc = MercatorPoint::from_xy(&(909111.0f64, 6217006.0f64));
        let wgs = WGS84Point::from_xy(&(8.1876716, 48.676003));
        let proj = WebMercatorProjection::make();
        let eucp = proj.project(&wgs);
        let wgsp = proj.unproject(&eucp);
        debug_assert_eq!(approx(wgs.point2d()), approx(wgsp.point2d()));
    }
}
