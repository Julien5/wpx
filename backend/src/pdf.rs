#[cfg(feature = "typstpdf")]
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};

#[cfg(feature = "typstpdf")]
pub fn compile(document: &str, debug: bool) -> Vec<u8> {
    if debug {
        std::fs::write("/tmp/document.typst", &document).unwrap();
    }

    let template = TypstEngine::builder()
        .main_file(document)
        .search_fonts_with(TypstKitFontOptions::default().include_system_fonts(false))
        .build();

    let doc = template
        .compile()
        .output
        .expect("typst::compile() returned an error!");

    let options = Default::default();

    let pdf = typst_pdf::pdf(&doc, &options).expect("Could not generate pdf.");
    pdf
}

#[cfg(not(feature = "typstpdf"))]
pub fn compile(document: &str, debug: bool) -> Vec<u8> {
    if debug {
        std::fs::write("/tmp/document.typst", &document).unwrap();
    }
    Vec::new()
}
