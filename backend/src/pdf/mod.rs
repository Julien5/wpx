#[cfg(not(target_arch = "wasm32"))]
mod local;
#[cfg(target_arch = "wasm32")]
mod wasm;

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
