mod fonts;
#[cfg(not(target_arch = "wasm32"))]
mod local;
pub mod render;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[allow(dead_code)]
#[cfg(not(target_arch = "wasm32"))]
pub async fn compile(document: &str, debug: bool) -> Vec<u8> {
    if debug {
        std::fs::write("/tmp/document.typst", &document).unwrap();
    }
    local::compile(document, debug).await
}

#[cfg(target_arch = "wasm32")]
pub async fn compile(document: &str, debug: bool) -> Vec<u8> {
    wasm::compile(document).await
}

/*
let host = location.host().unwrap();       // "localhost:3000"
let hostname = location.hostname().unwrap(); // "localhost"
let port = location.port().unwrap();         // "3000"
*/

#[cfg(target_arch = "wasm32")]
pub fn get_host() -> String {
    let h = web_sys::window()
        .and_then(|win| win.location().host().ok())
        .unwrap();
    format!("https://{}", h)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_host() -> String {
    // used for remote typst
    "https://vps-e637d6c5.vps.ovh.net:8123".to_string()
}

fn extract_ui_version() -> String {
    let bytes = include_bytes!("../../../frontend/ui/pubspec.yaml");
    let yaml = std::str::from_utf8(bytes).unwrap();
    yaml.lines()
        .find(|line| line.starts_with("version:"))
        .unwrap()
        .split_once(':')
        .unwrap()
        .1
        .trim()
        .into()
}

pub fn get_host_with_version() -> String {
    // https://localhost:8124/0.5.0+3/assets/fonts/LibertinusSerif-Regular.ttf
    format!("{}/{}", get_host(), extract_ui_version())
}

pub fn get_font_url(name: &str) -> String {
    // https://localhost:8124/0.5.0+3/assets/fonts/LibertinusSerif-Regular.ttf
    format!("{}/assets/fonts/{}", get_host_with_version(), name)
}
