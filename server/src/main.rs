use axum::{
    body::Body,
    extract::State,
    http::{HeaderName, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;
use hyper::{service::service_fn, HeaderMap};
use hyper_util::rt::TokioIo;
use std::{io::Write, net::IpAddr, net::SocketAddr, path::PathBuf, process::Command, sync::Arc};
use tempfile::NamedTempFile;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tower::{Service, ServiceBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{error, info};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "/tmp/dist")]
    directory: PathBuf,

    #[arg(short, long, default_value = "cert.pem")]
    cert: PathBuf,

    #[arg(short, long, default_value = "key.pem")]
    key: PathBuf,

    #[arg(short, long, default_value = "8123")]
    port: u16,

    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,
}

async fn log_request(req: Request<Body>, next: Next) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let version = req.version();

    info!("Request: {} {} {:?}", method, uri, version);

    next.run(req).await
}

async fn hello_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("text/plain"));
    (headers, "Hello, World!")
}

async fn typst_handler(body: String) -> impl IntoResponse {
    // Create a temporary directory for our work
    let temp_dir = match tempfile::Builder::new().prefix("typst_").tempdir() {
        Ok(dir) => dir,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create temporary directory",
            )
                .into_response()
        }
    };

    // Generate unique filenames
    let input_path = temp_dir.path().join(format!("{}.typst", Uuid::new_v4()));
    let pdf_path = temp_dir.path().join(format!("{}.pdf", Uuid::new_v4()));

    // Write content to temporary file
    if let Err(_) = std::fs::write(&input_path, body) {
        let error = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to write temporary file",
        );
        return error.into_response();
    }

    // create tyspt command
    let status = Command::new("/opt/typst/typst-x86_64-unknown-linux-musl/typst")
        .arg("compile")
        .arg(&input_path)
        .arg(&pdf_path)
        .status();

    if let Err(_) = status {
        let error = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create pdf file",
        );
        return error.into_response();
    }

    // read and return the result
    match std::fs::read(&pdf_path) {
        Ok(pdf_content) => {
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("application/pdf"));
            headers.insert(
                "Content-Disposition",
                HeaderValue::from_str(&format!("attachment; filename=\"document.pdf\"")).unwrap(),
            );
            (headers, pdf_content).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read pdf file").into_response(),
    }
}

async fn cache_control_middleware(req: axum::extract::Request, next: Next) -> Response {
    let path = req.uri().path().to_string(); // Clone the path before moving req
    let mut res = next.run(req).await;

    let headers = res.headers_mut();

    // Don't cache index.html or root path
    if path == "/" || path == "/index.html" {
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        );
        headers.insert("Pragma", HeaderValue::from_static("no-cache"));
        headers.insert("Expires", HeaderValue::from_static("0"));
    } else {
        // Cache everything else aggressively
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("public, max-age=31535000, immutable"),
        );
    }

    res
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();

    // Load TLS configuration
    let certs = load_certs(&args.cert)?;
    let key = load_private_key(&args.key)?;
    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    let tls_config = Arc::new(config);

    // Set up CORS with required headers
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any)
        .expose_headers([
            HeaderName::from_static("cross-origin-opener-policy"),
            HeaderName::from_static("cross-origin-embedder-policy"),
        ]);

    // Set up static file serving with error handling
    let static_files = ServeDir::new(args.directory);

    // Build the router with middleware
    let app = Router::new()
        .nest_service("/", static_files)
        .route("/api/hello", get(hello_handler))
        .route("/api/typst", post(typst_handler))
        .layer(middleware::from_fn(log_request))
        .layer(middleware::from_fn(cache_control_middleware))
        .layer(cors)
        .layer(
            ServiceBuilder::new().map_response(|mut res: axum::response::Response| {
                let headers = res.headers_mut();
                headers.insert(
                    "Cross-Origin-Opener-Policy",
                    HeaderValue::from_static("same-origin"),
                );
                headers.insert(
                    "Cross-Origin-Embedder-Policy",
                    HeaderValue::from_static("require-corp"),
                );
                headers.insert(
                    "Access-Control-Allow-Headers",
                    HeaderValue::from_static("*"),
                );
                res
            }),
        );

    // Set up the HTTPS server
    let ip: IpAddr = args.host.parse()?;
    let addr = SocketAddr::new(ip, args.port);
    let listener = TcpListener::bind(addr).await?;
    info!("HTTPS server listening on {}", addr);

    let acceptor = TlsAcceptor::from(tls_config);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let app = app.clone();

        tokio::spawn(async move {
            info!("Accepting connection from {}", peer_addr);
            match acceptor.accept(stream).await {
                Ok(stream) => {
                    let io = TokioIo::new(stream);
                    let service = service_fn(move |req| {
                        let mut app = app.clone();
                        async move { Ok::<_, hyper::Error>(app.call(req).await.unwrap()) }
                    });

                    if let Err(err) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        match err.is_incomplete_message() {
                            true => {
                                // This is usually due to client disconnection, totally normal
                                info!("Client {} disconnected early: {}", peer_addr, err);
                            }
                            false => {
                                // Log other errors as they might be important
                                error!("Connection error from {}: {}", peer_addr, err);
                            }
                        }
                    }
                }
                Err(err) => error!("TLS error from {}: {}", peer_addr, err),
            }
        });
    }
}

fn load_certs(path: &PathBuf) -> Result<Vec<rustls::Certificate>, Box<dyn std::error::Error>> {
    let cert_file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(cert_file);
    Ok(rustls_pemfile::certs(&mut reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect())
}

fn load_private_key(path: &PathBuf) -> Result<rustls::PrivateKey, Box<dyn std::error::Error>> {
    let key_file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(key_file);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)?;
    if keys.is_empty() {
        return Err("No private keys found".into());
    }
    Ok(rustls::PrivateKey(keys[0].clone()))
}
