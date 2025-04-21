use geo::{Distance, SimplifyIdx};

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let p1 = geo::Point::new(x1, y1);
    let p2 = geo::Point::new(x2, y2);
    geo::Haversine::distance(p1, p2)
}

use std::io::Read;

use gpx::TrackSegment;

pub struct Profile {
    pub xdata: Vec<f64>,
    pub ydata: Vec<f64>,
}

pub fn read_segment(filename: &str) -> Box<gpx::TrackSegment> {
    let file = std::fs::File::open(filename).unwrap();
    let mut reader_file = std::io::BufReader::new(file);
    let mut content: Vec<u8> = Vec::new();
    let _ = reader_file.read_to_end(&mut content);
    let reader_mem = std::io::Cursor::new(content);
    let mut gpx: gpx::Gpx = gpx::read(reader_mem).unwrap();
    let mut t0 = gpx.tracks.swap_remove(0);
    let s0 = t0.segments.swap_remove(0);
    Box::new(s0)
}

impl Profile {
    pub fn from_segment(segment: &TrackSegment) -> Profile {
        let mut xdata = Vec::new();
        let mut ydata = Vec::new();
        let mut prev: Option<geo::Point> = None;
        let mut d = 0f64;
        for k in 0..segment.points.len() {
            let point = &segment.points[k];
            let (x, y) = point.point().x_y();
            if let Some(p) = prev {
                d += distance(p.x(), p.y(), x, y);
                let dy = point.elevation.unwrap();
                let kx = d / 1000f64;
                xdata.push(kx);
                ydata.push(dy);
                if kx > 100f64 {
                    break;
                }
            }
            prev = Some(point.point());
        }
        Profile { xdata, ydata }
    }

    pub fn get_automatic_points(&self) -> Vec<usize> {
        use geo::line_string;
        use geo::Simplify;
        let mut coords = Vec::new();
        for k in 0..self.xdata.len() {
            let x = self.xdata[k];
            let y = self.ydata[k];
            coords.push(geo::coord!(x:x, y:y));
        }
        let mut line = geo::LineString::new(coords);
        line.simplify_idx(&5.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
