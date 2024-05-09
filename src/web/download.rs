use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Json,
};
use onedrive_api::{ItemId, ItemLocation};
use serde_json::json;
use snafu::Snafu;

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
    if let Some(url) = cached_url {
        return Redirect::to(&url).into_response();
    }

    let url = drive
        .load()
        .drive
        .get_item_download_url(ItemLocation::from_id(&ItemId(id.clone())))
        .await;

    match url {
        Ok(url) => {
            cache.insert(id, url.clone());
            Redirect::to(&url).into_response()
        }
        Err(e) => Error::GetDownloadUrl { source: e }.into_response(),
    }
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

pub fn router(state: Arc<AppState>) -> axum::Router {
    let route = axum::Router::new()
        .route("/:id", get(download_file))
        .with_state(state);

    axum::Router::new().nest("/download", route)
}
