extern crate gpx;
extern crate geo;

use gpx::read;
use gpx::{Gpx, Track, TrackSegment};


use geo::Distance;
fn distance(x1:f64,y1:f64,x2:f64,y2:f64) -> f64 {
	//let p0 = geoutils::Location::new(xprev, yprev);
	//let p1 = geoutils::Location::new(x, y);
	//let distance = p0.distance_to(&p1).unwrap().meters();
	let p1 = geo::Point::new(x1,y1);
	let p2 = geo::Point::new(x2,y2);
	let ret = geo::Haversine::distance(p1,p2);
	ret
}


use plotters::prelude::*;
pub fn plot_data(segment:&TrackSegment)  -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("0.png", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

	let mut prev:Option<geo::Point> = None;
	let mut xdata = Vec::new();
	let mut ydata = Vec::new();
	let mut d = 0f64;
	for point in &segment.points {
		let (x,y)=point.point().x_y();
		match prev {
			Some(p) => {
				d+=distance(p.x(),p.y(),x,y);
				let dy=point.elevation.unwrap();
				xdata.push(d);
				ydata.push(dy);
			}
			_ => {}
		}
		//println!("point: ({x:.2},{y:.2}:{z:.2})");
		prev=Some(point.point());
	}
	let xmax = xdata.last().unwrap().clone();
	
    let mut chart = ChartBuilder::on(&root)
        .caption("elevation", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f64..xmax, 0f64..2000f64)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (0..xdata.len()).map(|k| (xdata[k],ydata[k])),
            &RED,
        ))?
        .label("profile")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}

pub fn worker(content : &Vec<u8>) {
	let reader_mem=std::io::Cursor::new(content);
	let gpx: Gpx = read(reader_mem).unwrap();
    let track: &Track = &gpx.tracks[0];
    let segment: &TrackSegment = &track.segments[0];
	match plot_data(segment) {
		Ok(()) => {},
		_ => {
			println!("failed");
		}
	}
}
