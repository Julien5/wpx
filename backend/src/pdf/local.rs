use std::process::Command;

fn get_host() -> Option<String> {
    // TODO: load from config file.
    Some("https://vps-e637d6c5.vps.ovh.net:8123".to_string())
}

fn get_client() -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

async fn compile_remote(document: &str) -> Vec<u8> {
    log::trace!("compile remote");
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

fn compile_local(body: &str) -> Option<Vec<u8>> {
    let temp_dir = match tempfile::Builder::new().prefix("typst_").tempdir() {
        Ok(dir) => dir,
        Err(_) => {
            return None;
        }
    };

    // Generate unique filenames
    let input_path = temp_dir.path().join(format!("{}.typst", "document"));
    let pdf_path = temp_dir.path().join(format!("{}.pdf", "document"));

    // Write content to temporary file
    if let Err(_) = std::fs::write(&input_path, body) {
        return None;
    }

    log::trace!("compile local");

    // create tyspt command
    let status = Command::new("/opt/typst/typst-x86_64-unknown-linux-musl/typst")
        .arg("compile")
        .arg(&input_path)
        .arg(&pdf_path)
        .status();

    if let Err(_) = status {
        return None;
    }

    // read and return the result
    match std::fs::read(&pdf_path) {
        Ok(pdf_content) => {
            return Some(pdf_content);
        }
        Err(_) => {
            return None;
        }
    }
}

pub async fn compile(document: &str, debug: bool) -> Vec<u8> {
    if debug {
        std::fs::write("/tmp/document.typst", &document).unwrap();
    }
    match compile_local(document) {
        Some(ret) => ret,
        None => compile_remote(document).await,
    }
}
