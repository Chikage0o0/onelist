use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json,
};
use onedrive_api::{option::ObjectOption, resource::DriveItemField, ItemId, ItemLocation};
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::json;
use snafu::{ResultExt as _, Snafu};

use crate::DRIVE;

use super::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thumbnails {
    pub small: String,
    pub medium: String,
    pub large: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Size {
    Small,
    Medium,
    Large,
}

pub fn router(state: Arc<AppState>, use_proxy: bool) -> axum::Router {
    let route = axum::Router::new();
    let route = if use_proxy {
        route.route("/:size/:id", get(proxy_thumb))
    } else {
        route.route("/:size/:id", get(thumb))
    }
    .with_state(state);

    axum::Router::new().nest("/thumb", route)
}

async fn thumb(
    State(state): State<Arc<AppState>>,
    Path((size, id)): Path<(Size, String)>,
) -> impl IntoResponse {
    let thumb = match thumb_inner(state, &id).await {
        Ok(thumb) => thumb,
        Err(e) => return e.into_response(),
    };

    let url = match size {
        Size::Small => &thumb.small,
        Size::Medium => &thumb.medium,
        Size::Large => &thumb.large,
    };

    if url.is_empty() {
        return Response::builder().status(404).body(Body::empty()).unwrap();
    }

    Redirect::to(url).into_response()
}

async fn proxy_thumb(
    State(state): State<Arc<AppState>>,
    Path((size, id)): Path<(Size, String)>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let thumb = match thumb_inner(state, &id).await {
        Ok(thumb) => thumb,
        Err(e) => return e.into_response(),
    };

    let url = match size {
        Size::Small => &thumb.small,
        Size::Medium => &thumb.medium,
        Size::Large => &thumb.large,
    };

    if url.is_empty() {
        return Response::builder().status(404).body(Body::empty()).unwrap();
    }

    let req = reqwest::Client::new()
        .get(url)
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
                .unwrap_or("inline"),
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

async fn thumb_inner(state: Arc<AppState>, id: &str) -> Result<Arc<Thumbnails>, Error> {
    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return Err(Error::StillStarting),
    };

    let thumb_cache = &state.thumb_cache;
    let cached_thumb = thumb_cache.get(&id.to_string());
    match cached_thumb {
        Some(thumb) => return Ok(thumb.clone()),
        None => {
            let thumb = drive
                .load()
                .drive
                .get_item_with_option(
                    ItemLocation::from_id(&ItemId(id.to_owned())),
                    ObjectOption::default().expand(DriveItemField::thumbnails, None),
                )
                .await;

            let thumb = match thumb {
                Ok(Some(item)) => {
                    if let Some(thumbnails) = item.thumbnails {
                        let small = thumbnails[0]
                            .get("small")
                            .and_then(|v| v.get("url"))
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        let medium = thumbnails[0]
                            .get("medium")
                            .and_then(|v| v.get("url"))
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        let large = thumbnails[0]
                            .get("large")
                            .and_then(|v| v.get("url"))
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();

                        Arc::new(Thumbnails {
                            small: small.to_string(),
                            medium: medium.to_string(),
                            large: large.to_string(),
                        })
                    } else {
                        return Err(Error::LocationNotFound {
                            location: id.to_string(),
                        });
                    }
                }
                Ok(None) => {
                    return Err(Error::LocationNotFound {
                        location: id.to_string(),
                    })
                }
                Err(e) => return Err(Error::GetItemInfo { source: e }),
            };

            thumb_cache.insert(id.to_string(), thumb.clone());

            Ok(thumb)
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
enum Error {
    #[snafu(display("Server still in the process of starting up"))]
    StillStarting,

    #[snafu(display("Location not found: {}", location))]
    LocationNotFound { location: String },

    #[snafu(display("Failed to GetItemInfo: {}", source))]
    GetItemInfo { source: onedrive_api::Error },

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
