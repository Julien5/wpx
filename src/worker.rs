use crate::gpsdata;
use crate::pdf;
use crate::render;

pub fn worker(filename: &str) {
    let segment = gpsdata::read_segment(filename);
    let geodata = gpsdata::Track::from_segment(&segment);
    let typfile = render::compile(&geodata);
    let pdffile = typfile.replace(".typ", ".pdf");
    pdf::run(typfile.as_str(), pdffile.as_str());
}
