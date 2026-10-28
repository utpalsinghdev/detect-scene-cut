mod media;
mod model;
mod processing;
mod routes;
mod toolchain;

use axum::{
    Router,
    body::Body,
    extract::DefaultBodyLimit,
    http::{Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use model::App;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::{RwLock, Semaphore};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    toolchain::verify().await?;
    let root = PathBuf::from(std::env::var("DATA_DIR").unwrap_or_else(|_| "data".into()));
    let app = App {
        jobs: Arc::new(RwLock::new(HashMap::new())),
        root,
        worker: Arc::new(Semaphore::new(1)),
    };
    app.restore().await?;
    let router = Router::new()
        .route(
            "/api/health",
            get(|| async { axum::Json(serde_json::json!({"ok":true})) }),
        )
        .route("/api/uploads", post(routes::upload))
        .route("/api/jobs/{id}", get(routes::get).delete(routes::delete))
        .route("/api/jobs/{id}/split", post(routes::split))
        .route("/api/jobs/{id}/{asset}", get(routes::asset))
        .fallback_service(
            ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(middleware::from_fn(security))
        .with_state(app);
    // A Tailscale-only bind avoids exposing private footage on LAN/public interfaces.
    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "1900".into());
    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("Scene / split listening on {address}");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

async fn security(request: Request<Body>, next: Next) -> Response {
    if request.method() != axum::http::Method::GET
        && request.method() != axum::http::Method::HEAD
        && let Some(origin) = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|h| h.to_str().ok())
    {
        let host = request
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        if origin != format!("http://{host}") && origin != format!("https://{host}") {
            return (
                StatusCode::FORBIDDEN,
                "Cross-origin writes are not allowed.",
            )
                .into_response();
        }
    }
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        "nosniff".parse().expect("static header"),
    );
    headers.insert(
        "referrer-policy",
        "no-referrer".parse().expect("static header"),
    );
    headers.insert("x-frame-options", "DENY".parse().expect("static header"));
    headers.insert("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' blob: data:; media-src 'self' blob:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'".parse().expect("static header"));
    response
}
