use wasm_bindgen::prelude::*;

fn get_host() -> Option<String> {
    let h = web_sys::window()
        .and_then(|win| win.location().host().ok())
        .unwrap();
    Some(format!("https://{}", h))
}

fn get_client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn compile_remote(document: &str) -> Vec<u8> {
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

pub async fn compile(document: &str) -> Vec<u8> {
    compile_remote(document).await
}
