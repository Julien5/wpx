use crate::bbox::BoundingBox;
use crate::error::TrackError;
use crate::geometry::profilegeometry::ProfileGeometry;
use crate::inputpoint::InputPoint;
use crate::math::Point2D;
use crate::mercator;
use crate::parameters::TrackPart;
use crate::wgs84point::WGS84Point;

pub fn distance_wgs84(p1: &WGS84Point, p2: &WGS84Point) -> f64 {
    use geo::Distance;
    let gp1 = geo::Point::new(p1.x(), p1.y());
    let gp2 = geo::Point::new(p2.x(), p2.y());
    geo::Haversine.distance(gp1, gp2)
}

fn read_gpx_content(bytes: &Vec<u8>) -> Result<gpx::Gpx, TrackError> {
    let reader_mem = std::io::Cursor::new(bytes);
    match gpx::read(reader_mem) {
        Ok(d) => Ok(d),
        Err(_e) => Err(TrackError::GPXInvalid),
    }
}

fn make_track_from_segment(segment: &gpx::TrackSegment, name: String) -> gpx::Track {
    let mut ret = gpx::Track::new();
    ret.segments.push(segment.clone());
    ret.name = Some(name);
    return ret;
}

fn make_track_from_route(route: &gpx::Route, name: String) -> gpx::Track {
    let mut ret = gpx::Track::new();
    let mut segment = gpx::TrackSegment::new();
    let points = &route.points;
    for k in 0..points.len() {
        segment.points.push(points[k].clone());
    }
    ret.segments.push(segment);
    ret.name = Some(name);
    return ret;
}

fn read_routes(gpx: &mut gpx::Gpx) -> Result<Vec<gpx::Track>, TrackError> {
    let routes = &mut gpx.routes;
    let mut ret: Vec<gpx::Track> = Vec::new();
    for route in routes {
        ret.push(make_track_from_route(route, "foo".to_string()));
    }
    if ret.is_empty() {
        return Err(TrackError::GPXHasNoSegment);
    }
    Ok(ret)
}

fn read_tracks(gpx: &mut gpx::Gpx) -> Result<Vec<gpx::Track>, TrackError> {
    let tracks = &mut gpx.tracks;
    let mut ret: Vec<gpx::Track> = Vec::new();
    for track in tracks {
        for (index, segment) in track.segments.iter().enumerate() {
            let name = if track.segments.len() > 1 {
                format!(
                    "segment-{:0>2}: {}",
                    index + 1,
                    track.name.clone().unwrap_or_default()
                )
            } else {
                track.name.clone().unwrap_or_default()
            };
            ret.push(make_track_from_segment(segment, name));
        }
    }
    if ret.is_empty() {
        return Err(TrackError::GPXHasNoSegment);
    }
    Ok(ret)
}

pub struct GpxData {
    pub waypoints: Vec<InputPoint>,
    pub tracks: Vec<(String, gpx::Track)>,
}

impl GpxData {
    pub fn merge(data: Vec<GpxData>) -> Self {
        let mut waypoints = Vec::new();
        let mut tracks = Vec::new();
        for d in data {
            waypoints.extend_from_slice(&d.waypoints);
            tracks.extend_from_slice(&d.tracks);
        }
        GpxData { waypoints, tracks }
    }

    pub fn read_contents(contents: &Vec<Vec<u8>>) -> Result<Self, TrackError> {
        let mut gpxdatas = Vec::new();
        for content in contents {
            let result = Self::read_content(content);
            if result.is_err() {
                return Err(result.err().unwrap());
            }
            gpxdatas.push(result.ok().unwrap());
        }
        Ok(Self::merge(gpxdatas))
    }

    pub fn read_content(content: &Vec<u8>) -> Result<Self, TrackError> {
        let mut gpx = read_gpx_content(content)?;
        let raw_tracks = if gpx.tracks.is_empty() {
            match read_routes(&mut gpx) {
                Ok(s) => s,
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            match read_tracks(&mut gpx) {
                Ok(s) => s,
                Err(e) => {
                    return Err(e);
                }
            }
        };
        let named_tracks: Vec<_> = raw_tracks
            .iter()
            .enumerate()
            .map(|(_index, track)| (Self::track_name(&track), track.clone()))
            .collect();
        Ok(GpxData {
            tracks: named_tracks,
            waypoints: read_waypoints(&gpx),
        })
    }

    fn track_name(track: &gpx::Track) -> String {
        track.name.as_ref().unwrap_or(&String::new()).clone()
    }

    fn track_part(track: &gpx::Track, id: usize) -> TrackPart {
        debug_assert_eq!(track.segments.len(), 1);
        let points = &track.segments.first().as_ref().unwrap().points;
        let name = Self::track_name(track);
        TrackPart {
            name,
            part_index: id,
            length: points.len(),
        }
    }

    pub fn track_parts(&self) -> Vec<TrackPart> {
        self.tracks
            .iter()
            .enumerate()
            .map(|(index, namedtrack)| Self::track_part(&namedtrack.1, index))
            .collect()
    }

    fn _check_begin_end(&self) {
        fn to_wgs84(point: &gpx::Waypoint) -> WGS84Point {
            let (lon, lat) = point.point().x_y();
            let elevation = match point.elevation {
                Some(e) => e,
                None => 0f64,
            };
            WGS84Point::new(&lon, &lat, &elevation)
        }
        let mut last_end = None;
        for (index, t) in self.tracks.iter().enumerate() {
            let track_begin = t.1.segments.first().unwrap().points.first().unwrap();
            let track_end = t.1.segments.first().unwrap().points.last().unwrap();
            let name = &t.0;
            if last_end.is_some() {
                let p1 = to_wgs84(last_end.unwrap());
                let p2 = to_wgs84(track_begin);
                let d = distance_wgs84(&p1, &p2);
                log::info!("index:{} name:{:25} d(end,begin)={:.1}", index, name, d);
            }
            last_end = Some(track_end);
        }
    }

    pub fn reorder(&self, order: &Vec<usize>) -> Self {
        // The waypoints are not affected.
        let mut new_tracks = Vec::new();
        for index in order {
            new_tracks.push(self.tracks[*index].clone());
        }

        let ret = GpxData {
            tracks: new_tracks,
            waypoints: self.waypoints.clone(),
        };
        //ret.check_begin_end();
        ret
    }
}

pub type ProfileBoundingBox = BoundingBox;

impl ProfileBoundingBox {
    pub fn from_track(profile: &ProfileGeometry, start: &f64, end: &f64) -> ProfileBoundingBox {
        let mut ymin = f64::MAX;
        let mut ymax = f64::MIN;
        for k in 0..profile.len() {
            let y = profile.elevation(k);
            ymin = y.min(ymin);
            ymax = y.max(ymax);
        }
        let xmin = *start;
        let xmax = *end;
        BoundingBox::minmax(Point2D::new(xmin, ymin), Point2D::new(xmax, ymax))
    }
}

pub fn read_waypoints(gpx: &gpx::Gpx) -> Vec<InputPoint> {
    let mut ret = Vec::new();
    let projection = mercator::WebMercatorProjection::make();
    for w in gpx.waypoints.iter() {
        let (lon, lat) = w.point().x_y();
        let wgs = WGS84Point::new(&lon, &lat, &0f64);
        let euc = projection.project(&wgs);
        let p = InputPoint::from_gpx(&wgs, &euc, &w.name, &w.description);
        ret.push(p);
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

        let simplified = line_string.simplify(1.0);

        let expected = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 4.0),
            (x: 11.0, y: 5.5),
            (x: 27.8, y: 0.1),
        ];

        debug_assert_eq!(expected, simplified);
    }
}
