extern crate gpx;
extern crate geo;

use gpx::read;
use gpx::{Gpx, Track, TrackSegment};


use geo::Distance;
fn _distance(x1:f64,y1:f64,x2:f64,y2:f64) -> f64 {
	//let p0 = geoutils::Location::new(xprev, yprev);
	//let p1 = geoutils::Location::new(x, y);
	//let distance = p0.distance_to(&p1).unwrap().meters();
	let p1 = geo::Point::new(x1,y1);
	let p2 = geo::Point::new(x2,y2);
	let ret = geo::Haversine::distance(p1,p2);
	ret
}

pub fn worker(content : &Vec<u8>) {
	let reader_mem=std::io::Cursor::new(content);
	let gpx: Gpx = read(reader_mem).unwrap();
    let track: &Track = &gpx.tracks[0];
    let segment: &TrackSegment = &track.segments[0];

	for point in &segment.points {
		let (x,y)=point.point().x_y();
		let z = point.elevation.unwrap();
		println!("point: ({x:.2},{y:.2}:{z:.2})");
	}
}
