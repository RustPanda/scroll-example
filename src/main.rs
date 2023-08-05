use axum::{
    http::{header, Response, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use local_ip_address::local_ip;
use qrcode_generator::QrCodeEcc;
use rust_embed::RustEmbed;

static INDEX_HTML: &str = "index.html";

#[derive(RustEmbed)]
#[folder = "webui"]
struct Dist;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/qrcode", get(qrcode_gen))
        .route("/qrcode.png", get(qrcode_gen))
        .fallback(static_handler);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    println!("Server run on: http://localhost:8080");

    axum::serve(listener, app).await.unwrap();
}

async fn qrcode_gen() -> Response<axum::body::Body> {
    let local_ip = local_ip().unwrap();
    let data = qrcode_generator::to_png_to_vec(
        format!("Server run on: http://{local_ip}:8080"),
        QrCodeEcc::Low,
        1024,
    )
    .unwrap();

    let mime = mime_guess::mime::PNG;

    ([(header::CONTENT_TYPE, mime.as_ref())], data).into_response()
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return index_html().await;
    }

    match Dist::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return not_found().await;
            }

            index_html().await
        }
    }
}

async fn index_html() -> Response<axum::body::Body> {
    match Dist::get(INDEX_HTML) {
        Some(content) => Html(content.data).into_response(),
        None => not_found().await,
    }
}

async fn not_found() -> Response<axum::body::Body> {
    (StatusCode::NOT_FOUND, "404").into_response()
}
