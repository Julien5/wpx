use crate::gpsdata;
use crate::minimap;
use crate::pdf;
use crate::render;

pub fn worker(filename: &str) {
    let segment = gpsdata::read_segment(filename);
    let path = minimap::Path::from_segment(&segment);
    let geodata = gpsdata::GeoData::from_segment(&segment);
    let _ = render::profile(&geodata, "/tmp/profile.svg");
    render::map(&path);
    pdf::run();
}
