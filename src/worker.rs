use crate::gpsdata;
use crate::minimap;
use crate::pdf;
use crate::render;

pub fn worker(filename: &str) {
    let segment = gpsdata::read_segment(filename);
    let path = minimap::Path::from_segment(&segment);
    let profile = gpsdata::Profile::from_segment(&segment);
    let _ = render::profile(&profile, "/tmp/profile.svg");
    render::map(&path);
    pdf::run();
}
