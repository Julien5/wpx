use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};

pub fn compile(document: &str) -> Vec<u8> {
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
