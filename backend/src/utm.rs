use crate::waypoint::WGS84Point;

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Clone)]
pub struct UTMPoint(pub f64, pub f64);

impl UTMPoint {
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
    pub fn xy(&self) -> (f64, f64) {
        (self.0, self.1)
    }
}

pub struct UTMProjection {
    wgs84_spec: proj4rs::proj::Proj,
    utm_spec: proj4rs::proj::Proj,
}

impl UTMProjection {
    pub fn make(center: WGS84Point) -> UTMProjection {
        // see https://en.wikipedia.org/wiki/Universal_Transverse_Mercator_coordinate_system
        // we take the first point of each segment
        // we should wait until we have the user segments (pages) to ensure the same
        // zone for a minimap.
        let zone = (((center.longitude() + 180f64) / 6f64).floor() + 1f64) as i32;
        log::info!("projecting in UTM{}", zone);
        use proj4rs::proj::Proj;
        let spec = format!(
            "+proj=utm +zone={} +datum=WGS84 +units=m +no_defs +type=crs",
            zone
        );
        let utm_spec = Proj::from_proj_string(spec.as_str()).unwrap();

        let spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
        let wgs84_spec = Proj::from_proj_string(spec).unwrap();
        UTMProjection {
            wgs84_spec,
            utm_spec,
        }
    }
    pub fn project(&self, wgs84: &WGS84Point) -> UTMPoint {
        let mut p = (
            wgs84.longitude().to_radians(),
            wgs84.latitude().to_radians(),
        );
        proj4rs::transform::transform(&self.wgs84_spec, &self.utm_spec, &mut p).unwrap();
        UTMPoint(p.0, p.1)
    }
}
