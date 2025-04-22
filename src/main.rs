pub mod gpsdata;
pub mod pdf;
pub mod render;
pub mod speed;
pub mod worker;

fn main() {
    let filename = "data/blackforest.gpx";
    worker::worker(filename);
}
