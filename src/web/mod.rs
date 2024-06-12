use std::{
    borrow::Cow,
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use mini_moka::sync::Cache;
use rust_embed::RustEmbed;
use tokio::signal;
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;

use crate::{model::Caches, utils::config::Setting, NAME};

mod download;
mod item;
mod list;
mod thumb;

type Client = hyper_util::client::legacy::Client<HttpsConnector<HttpConnector>, Body>;

pub async fn web_server(config: Setting) {
    let app = router(config.clone());

    info!("Starting the web server");

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.setting.port))
        .await
        .unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("Web server stopped");
}

#[derive(Debug)]
struct AppState {
    home_dir: String,
    cache: Caches,
    client: Client,
}

const CACHE_DURATION: Duration = Duration::from_secs(60 * 10);

fn router(config: Setting) -> Router {
    let home_dir = if config.setting.home_dir.starts_with('/') {
        config.setting.home_dir
    } else {
        format!("/{}", config.setting.home_dir)
    };

    let client = hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new()).build(
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("no native root CA certificates found")
            .https_or_http()
            .enable_all_versions()
            .build(),
    );

    let download_url_cache = Cache::builder().time_to_live(CACHE_DURATION).build();
    let list_cache = Cache::builder().time_to_live(CACHE_DURATION).build();
    let thumb_cache = Cache::builder().time_to_live(CACHE_DURATION).build();
    let file_cache = Cache::builder().time_to_live(CACHE_DURATION).build();
    let state = Arc::new(AppState {
        home_dir,
        cache: Caches {
            download_url_cache,
            list_cache,
            thumb_cache,
            file_cache,
        },
        client,
    });

    let router = Router::new()
        .merge(list::router(state.clone()))
        .merge(thumb::router(state.clone(), config.setting.use_proxy))
        .merge(download::router(state.clone(), config.setting.use_proxy))
        .merge(item::router(state.clone()));

    Router::new()
        .nest("/api", router)
        .fallback_service(get(static_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

static INDEX_HTML: &str = "index.html";

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct Assets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return index_html().await;
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => index_html().await,
    }
}

static INDEX_BYTE: OnceLock<Cow<'static, [u8]>> = OnceLock::new();

async fn index_html() -> Response {
    match Assets::get(INDEX_HTML) {
        Some(content) => {
            // replace the placeholder with the actual name
            let byte = INDEX_BYTE.get_or_init(|| {
                let name = NAME.get().unwrap();
                let content = content.data;
                let string = String::from_utf8_lossy(&content);
                let content = string.replace("{{NAME}}", name).as_bytes().to_vec();

                Cow::Owned(content)
            });

            Html(byte.clone()).into_response()
            // replace the placeholder with the actual name
        }
        None => not_found().await,
    }
}

async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "404").into_response()
}
