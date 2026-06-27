use crate::error::TrackError;
use std::sync::{OnceLock, RwLock};

const LIBERTINUS_FONT_FILES: &[&str] = &[
    "LibertinusSerif-Regular.ttf",
    "LibertinusSerif-Bold.ttf",
    "LibertinusSerif-Italic.ttf",
];

/// Cached font data singleton
struct FontData {
    font_bytes: Vec<Vec<u8>>,
}

static FONT_CACHE: OnceLock<RwLock<Option<FontData>>> = OnceLock::new();

mod download_font {
    #[cfg(target_arch = "wasm32")]
    pub async fn get(file: &str) -> Vec<u8> {
        log::trace!("download font data {}", file);
        use crate::pdf::get_font_url;
        let url = get_font_url(file);
        let client = reqwest::Client::new();
        let response = client.get(url).send().await.unwrap();
        let data = response.bytes().await;
        let data = data.ok();
        data.unwrap().to_vec()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get(file: &str) -> Vec<u8> {
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

/// Initialize fonts by downloading/loading all font data asynchronously.
/// This should be called once during application startup.
pub async fn init_fonts() -> Result<(), TrackError> {
    let mut font_data = Vec::new();

    for file in LIBERTINUS_FONT_FILES {
        let data = download_font::get(file).await;
        font_data.push(data);
    }

    let font_cache = FONT_CACHE.get_or_init(|| RwLock::new(None));
    let mut cache = font_cache.write().map_err(|_| TrackError::IOError)?;
    *cache = Some(FontData {
        font_bytes: font_data,
    });

    Ok(())
}

/// Apply cached fonts to a fontdb Database.
/// Requires init_fonts() to have been called first.
pub fn apply_cached_fonts(db: &mut svg2pdf::usvg::fontdb::Database) -> Result<(), TrackError> {
    let font_cache = FONT_CACHE.get_or_init(|| RwLock::new(None));
    let cache = font_cache.read().map_err(|_| TrackError::IOError)?;

    let font_data = cache.as_ref().ok_or(TrackError::IOError)?;

    for bytes in &font_data.font_bytes {
        db.load_font_data(bytes.clone());
    }

    db.set_serif_family("Libertinus Serif");
    db.set_sans_serif_family("Libertinus Serif");

    Ok(())
}
