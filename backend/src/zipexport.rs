#![allow(non_snake_case)]
use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod; // Add Write trait

pub fn generate(content: BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buffer);

    //let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    // Use a fixed timestamp to avoid system time calls on wasm
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::from_date_and_time(2024, 1, 1, 0, 0, 0).unwrap());

    // Add GPX file
    for (filename, filecontent) in content {
        zip.start_file(filename, options).unwrap();
        zip.write_all(&filecontent).unwrap();
    }

    // Finish and extract bytes
    zip.finish().unwrap().into_inner()
}
