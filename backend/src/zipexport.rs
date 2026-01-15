#![allow(non_snake_case)]
use std::io::{Cursor, Write};
use zip::write::{FileOptions, ZipWriter}; // Add Write trait

pub fn generate(gpx: &[u8], pdf: &[u8]) -> Vec<u8> {
    let buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buffer);

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    // Add GPX file
    zip.start_file("waypoints.gpx", options).unwrap();
    zip.write_all(gpx).unwrap();

    // Add PDF file
    zip.start_file("route.pdf", options).unwrap();
    zip.write_all(pdf).unwrap();

    // Finish and extract bytes
    zip.finish().unwrap().into_inner()
}
