use crate::bbox::BoundingBox;
use crate::error::Error;
use crate::inputpoint::InputPoint;
use crate::wgs84point::WGS84Point;
use crate::{mercator, track};
use geo::Distance;

pub fn distance_wgs84(p1: &WGS84Point, p2: &WGS84Point) -> f64 {
    let p1 = geo::Point::new(p1.x(), p1.y());
    let p2 = geo::Point::new(p2.x(), p2.y());
    geo::Haversine::distance(p1, p2)
}

pub fn read_gpx_content(bytes: &Vec<u8>) -> Result<gpx::Gpx, Error> {
    let reader_mem = std::io::Cursor::new(bytes);
    match gpx::read(reader_mem) {
        Ok(d) => Ok(d),
        Err(_e) => Err(Error::GPXInvalid),
    }
}

pub fn read_segment(gpx: &mut gpx::Gpx) -> Result<gpx::TrackSegment, Error> {
    let tracks = &mut gpx.tracks;
    tracks.sort_by_key(|track| {
        let zero = "A".to_string();
        let infinity = "ziel".to_string();
        if track.name.is_none() {
            return zero;
        }
        let name = track.name.as_ref().unwrap().to_lowercase();
        if name.contains("end") {
            return infinity;
        }
        if name.contains("ziel") {
            return infinity;
        }
        if name.contains("start") {
            return zero;
        }
        return name;
    });
    let mut ret = gpx::TrackSegment::new();
    for track in tracks {
        let points = &track.segments.first().unwrap().points;
        for k in 0..points.len() {
            ret.points.push(points[k].clone());
        }
    }
    if ret.points.is_empty() {
        return Err(Error::GPXHasNoSegment);
    }
    Ok(ret)
}

pub type ProfileBoundingBox = BoundingBox;

impl ProfileBoundingBox {
    pub fn from_track(track: &track::Track, range: &std::ops::Range<usize>) -> ProfileBoundingBox {
        let mut ymin = f64::MAX;
        let mut ymax = f64::MIN;
        for k in range.start..range.end {
            let y = track.elevation(k);
            ymin = y.min(ymin);
            ymax = y.max(ymax);
        }
        let xmin = track.distance(range.start);
        let xmax = track.distance(range.end - 1);
        BoundingBox::init((xmin, ymin), (xmax, ymax))
    }
}

pub fn read_waypoints(gpx: &gpx::Gpx) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    let projection = mercator::WebMercatorProjection::make();
    for w in &gpx.waypoints {
        let (lon, lat) = w.point().x_y();
        let wgs = WGS84Point::new(&lon, &lat, &0f64);
        ret.push(InputPoint::from_gpx(
            &wgs,
            &projection.project(&wgs),
            &w.name,
            &w.description,
        ));
    }
    ret
}

#[cfg(test)]
mod tests {
    use geo::line_string;
    use geo::Simplify;
    #[test]
    fn simplify() {
        let line_string = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 4.0),
            (x: 11.0, y: 5.5),
            (x: 17.3, y: 3.2),
            (x: 27.8, y: 0.1),
        ];

        let simplified = line_string.simplify(&1.0);

        let expected = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 4.0),
            (x: 11.0, y: 5.5),
            (x: 27.8, y: 0.1),
        ];

        assert_eq!(expected, simplified);
    }
}
