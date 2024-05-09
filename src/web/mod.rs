use std::{sync::Arc, time::Duration};

use axum::Router;
use mini_moka::sync::Cache;
use tokio::signal;
use tracing::info;

use crate::utils::config::Setting;

use self::list::FileInfo;

mod download;
mod list;

pub async fn web_server(config: Setting) {
    let app = router(config);

    info!("Starting the web server");

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("Web server stopped");
}

#[derive(Debug)]
struct AppState {
    home_dir: String,
    download_cache: Cache<String, String>,
    list_cache: Cache<String, Arc<Vec<FileInfo>>>,
}

fn router(config: Setting) -> Router {
    let home_dir = if config.home_dir.starts_with('/') {
        config.home_dir
    } else {
        format!("/{}", config.home_dir)
    };

    let download_cache = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 10))
        .build();
    let list_cache = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 30))
        .build();

    let state = Arc::new(AppState {
        home_dir,
        download_cache,
        list_cache,
    });
    let router = Router::new()
        .merge(list::router(state.clone()))
        .merge(download::router(state.clone()));
    Router::new().nest("/api", router)
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
