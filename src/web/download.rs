use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
    routing::get,
    Json,
};

use onedrive_api::{ItemId, ItemLocation};

use serde_json::json;
use snafu::Snafu;

use crate::DRIVE;

use super::{reverse_proxy, AppState};

async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return Error::StillStarting.into_response(),
    };

    let cache = state.cache.download_url_cache.clone();
    let cached_url = cache.get(&id);
    let url = if let Some(url) = cached_url {
        url
    } else {
        let url = drive
            .load()
            .drive
            .get_item_download_url(ItemLocation::from_id(&ItemId(id.clone())))
            .await;

        match url {
            Ok(url) => {
                cache.insert(id, url.clone());
                url
            }
            Err(e) => return Error::GetDownloadUrl { source: e }.into_response(),
        }
    };

    Redirect::to(&url).into_response()
}

async fn proxy_download_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return Error::StillStarting.into_response(),
    };

    let cache = state.cache.download_url_cache.clone();
    let cached_url = cache.get(&id);
    let url = if let Some(url) = cached_url {
        url
    } else {
        let url = drive
            .load()
            .drive
            .get_item_download_url(ItemLocation::from_id(&ItemId(id.clone())))
            .await;

        match url {
            Ok(url) => {
                cache.insert(id, url.clone());
                url
            }
            Err(e) => return Error::GetDownloadUrl { source: e }.into_response(),
        }
    };

    reverse_proxy(headers, url.to_string())
        .await
        .into_response()
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
enum Error {
    #[snafu(display("Server still in the process of starting up"))]
    StillStarting,

    #[snafu(display("Failed to get the download url: {}", source))]
    GetDownloadUrl { source: onedrive_api::Error },
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::http::Response<axum::body::Body> {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": self.to_string() })),
        )
            .into_response()
    }
}

pub fn router(state: Arc<AppState>, use_proxy: bool) -> axum::Router {
    let route = axum::Router::new();
    let route = if use_proxy {
        route.route("/:id", get(proxy_download_file))
    } else {
        route.route("/:id", get(download_file))
    }
    .with_state(state);

    axum::Router::new().nest("/download", route)
}
