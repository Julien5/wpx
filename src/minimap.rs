// Get the WGS84 UTM zone 32N projection

pub struct Path {
    pub path: Vec<(f64, f64)>,
    pub min: (i64, i64),
    pub max: (i64, i64),
}

impl Path {
    pub fn from_segment(segment: &gpx::TrackSegment) -> Path {
        use proj4rs::proj::Proj;
        // https://epsg.io/32632 = UTM32N
        let mut path = Vec::new();
        let mut min = (i64::MAX, i64::MAX);
        let mut max = (i64::MIN, i64::MIN);

        let s = "+proj=utm +zone=32 +datum=WGS84 +units=m +no_defs +type=crs";
        let utm32n = Proj::from_proj_string(s).unwrap();

        let wgs84 =
            Proj::from_proj_string("+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs").unwrap();

        for p in &segment.points {
            let (lon, lat) = p.point().x_y();
            let mut p = (lon.to_radians(), lat.to_radians());
            proj4rs::transform::transform(&wgs84, &utm32n, &mut p).unwrap();
            let (x, y, _) = (p.0, p.1, 0f64);
            min.0 = std::cmp::min(min.0, x.floor() as i64);
            min.1 = std::cmp::min(min.1, y.floor() as i64);
            max.0 = std::cmp::max(max.0, x.ceil() as i64);
            max.1 = std::cmp::max(max.1, y.ceil() as i64);
            path.push((x, y));
        }
        Path { path, min, max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proj() {
        use proj4rs::proj::Proj;

        let s = "+proj=utm +zone=32 +datum=WGS84 +units=m +no_defs +type=crs";
        let utm32n = Proj::from_proj_string(s).unwrap();

        let wgs84 =
            Proj::from_proj_string("+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs").unwrap();

        let mut point_3d = (9.789994f64.to_radians(), 48.098441f64.to_radians(), 0.0);
        match proj4rs::transform::transform(&wgs84, &utm32n, &mut point_3d) {
            Ok(()) => {}
            Err(e) => {
                println!("err:{}", e);
                assert!(false);
            }
        }
        // XXX Note that angular unit is radians, not degrees !

        assert_eq!(point_3d.0, 558817.6038974144);
        assert_eq!(point_3d.1, 5327543.438389254);
    }
}
