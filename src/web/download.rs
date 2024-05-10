use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json,
};

use onedrive_api::{ItemId, ItemLocation};
use reqwest::header;
use serde_json::json;
use snafu::{ResultExt, Snafu};

use crate::DRIVE;

use super::AppState;

async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return Error::StillStarting.into_response(),
    };

    let cache = state.download_cache.clone();
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
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return Error::StillStarting.into_response(),
    };

    let cache = state.download_cache.clone();
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

    let req = reqwest::Client::new()
        .get(&url)
        .header(
            header::ACCEPT,
            headers
                .get(header::ACCEPT)
                .map(|v| v.to_str().unwrap())
                .unwrap_or("*/*"),
        )
        .header(
            header::CONTENT_RANGE,
            headers
                .get(header::CONTENT_RANGE)
                .map(|v| v.to_str().unwrap())
                .unwrap_or(""),
        )
        .send()
        .await
        .context(ProxyDownloadSnafu);
    let (headers, body) = match req {
        Ok(resp) => {
            let headers = resp.headers().clone();
            let body = resp.bytes_stream();
            (headers, body)
        }
        Err(e) => return e.into_response(),
    };

    Response::builder()
        .status(200)
        .header(
            header::CONTENT_TYPE,
            headers
                .get(header::CONTENT_TYPE)
                .map(|v| v.to_str().unwrap())
                .unwrap_or("application/octet-stream"),
        )
        .header(
            header::CONTENT_DISPOSITION,
            headers
                .get(header::CONTENT_DISPOSITION)
                .map(|v| v.to_str().unwrap())
                .unwrap_or("attachment"),
        )
        .header(
            header::CONTENT_LENGTH,
            headers
                .get(header::CONTENT_LENGTH)
                .map(|v| v.to_str().unwrap())
                .unwrap_or("0"),
        )
        .header(
            header::CACHE_CONTROL,
            headers
                .get(header::CACHE_CONTROL)
                .map(|v| v.to_str().unwrap())
                .unwrap_or("no-cache"),
        )
        .body(Body::from_stream(body))
        .unwrap()
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
enum Error {
    #[snafu(display("Server still in the process of starting up"))]
    StillStarting,

    #[snafu(display("Failed to get the download url: {}", source))]
    GetDownloadUrl { source: onedrive_api::Error },

    #[snafu(display("Failed to proxy the download: {}", source))]
    ProxyDownload { source: reqwest::Error },
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

pub fn router(state: Arc<AppState>) -> axum::Router {
    let route = axum::Router::new()
        .route("/raw/:id", get(download_file))
        .route("/proxy/:id", get(proxy_download_file))
        .with_state(state);

    axum::Router::new().nest("/download", route)
}
