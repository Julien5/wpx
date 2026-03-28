const LIBERTINUS_FONT_FILES: &[&str] = &[
    "LibertinusSerif-Regular.ttf",
    "LibertinusSerif-Bold.ttf",
    "LibertinusSerif-Italic.ttf",
];

mod download_font {
    fn get_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn get(file: &str) -> Vec<u8> {
        log::trace!("download font data {}", file);
        use crate::pdf::get_font_url;
        let url = get_font_url(file);
        let client = get_client();
        let response = client.get(url).send().await.unwrap();
        let data = response.bytes().await;
        let data = data.ok();
        data.unwrap().to_vec()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get(file: &str) -> Vec<u8> {
        log::trace!("load embedded font data {}", file);
        if file.contains("Bold") {
            return include_bytes!("../../../frontend/ui/fonts/LibertinusSerif-Bold.ttf").to_vec();
        }
        if file.contains("Italic") {
            return include_bytes!("../../../frontend/ui/fonts/LibertinusSerif-Italic.ttf")
                .to_vec();
        }
        include_bytes!("../../../frontend/ui/fonts/LibertinusSerif-Regular.ttf").to_vec()
    }
}

pub async fn register_libertinus_fonts(db: &mut usvg::fontdb::Database) {
    for file in LIBERTINUS_FONT_FILES {
        let data = download_font::get(&file).await;
        db.load_font_data(data);
    }
    db.set_serif_family("Libertinus Serif");
    db.set_sans_serif_family("Libertinus Serif");
}
