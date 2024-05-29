use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json,
};
use onedrive_api::{option::ObjectOption, resource::DriveItemField, ItemId, ItemLocation};

use serde::{Deserialize, Serialize};
use serde_json::json;
use snafu::{ResultExt as _, Snafu};

use crate::{
    model::{thumb::parse_thumb, Thumbnails},
    DRIVE,
};

use super::AppState;

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
    mut req: Request,
) -> impl IntoResponse {
    let thumb = match thumb_inner(state.clone(), &id).await {
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

    let client = &state.client;

    req.headers_mut().remove("host");
    req.headers_mut().remove("referer");
    *req.uri_mut() = Uri::try_from(url).unwrap();

    let ret = client.request(req).await;
    match ret {
        Ok(response) => response.into_response(),
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

async fn thumb_inner(state: Arc<AppState>, id: &str) -> Result<Arc<Thumbnails>, Error> {
    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return Err(Error::StillStarting),
    };

    let thumb_cache = &state.cache.thumb_cache;
    let cached_thumb = thumb_cache.get(&id.to_string());
    match cached_thumb {
        Some(thumb) => Ok(thumb.clone()),
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
                    if let Some(thumbnails) = &item.thumbnails {
                        Arc::new(parse_thumb(thumbnails).context(ParseFailedSnafu)?)
                    } else {
                        return Err(Error::IdNotFound { id: id.to_string() });
                    }
                }
                Ok(None) => return Err(Error::IdNotFound { id: id.to_string() }),
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

    #[snafu(display("Location not found: {}", id))]
    IdNotFound { id: String },

    #[snafu(display("Failed to GetItemInfo: {}", source))]
    GetItemInfo { source: onedrive_api::Error },

    #[snafu(display("Failed to parse the thumb: {}", source))]
    ParseFailed { source: crate::model::thumb::Error },
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
