//! Serves the web app under `/`: from a configured directory, else from the binary.

use axum::Router;
use std::path::Path;
use tower_http::services::ServeDir;

/// Adds the web app as fallback; the API paths keep priority.
pub fn with_web_app(router: Router, web_dir: Option<&Path>) -> Router {
    match web_dir {
        Some(directory) => router.fallback_service(ServeDir::new(directory)),
        None => with_embedded_web_app(router),
    }
}

#[cfg(feature = "embedded-web")]
fn with_embedded_web_app(router: Router) -> Router {
    use axum::{
        http::{StatusCode, Uri, header},
        response::{IntoResponse, Response},
    };
    use include_dir::{Dir, include_dir};

    static WEB_APP: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../app/build/web");

    async fn serve(uri: Uri) -> Response {
        let requested = uri.path().trim_start_matches('/');
        let path = if requested.is_empty() {
            "index.html"
        } else {
            requested
        };
        match WEB_APP.get_file(path) {
            Some(file) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.to_string())], file.contents()).into_response()
            }
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    router.fallback(serve)
}

#[cfg(not(feature = "embedded-web"))]
fn with_embedded_web_app(router: Router) -> Router {
    router
}
