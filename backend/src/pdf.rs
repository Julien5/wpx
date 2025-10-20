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

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn get_host() -> Option<String> {
    let h = web_sys::window()
        .and_then(|win| win.location().host().ok())
        .unwrap();
    Some(format!("https://{}", h))
}

#[cfg(not(target_arch = "wasm32"))]
fn get_host() -> Option<String> {
    // TODO: load from config file.
    Some("https://vps-e637d6c5.vps.ovh.net:8123".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn get_client() -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

#[cfg(target_arch = "wasm32")]
fn get_client() -> reqwest::Client {
    reqwest::Client::new()
}

#[cfg(not(feature = "typstpdf"))]
pub async fn compile_remote(document: &str) -> Vec<u8> {
    use reqwest::{Client, ClientBuilder};
    let url = format!("{}/api/typst", get_host().unwrap());
    let client = get_client();
    let response = client
        .post(url)
        .header("User-Agent", "jbo/WPX")
        .header("Content-Type", "text/plain; charset=UTF-8")
        .body(format!("{}", document))
        .send()
        .await
        .unwrap();
    let data = response.bytes().await;
    let data = data.ok();
    data.unwrap().to_vec()
}

pub async fn compile(document: &str, debug: bool) -> Vec<u8> {
    if debug {
        std::fs::write("/tmp/document.typst", &document).unwrap();
    }
    compile_remote(document).await
}
