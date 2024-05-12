use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json,
};
use onedrive_api::ItemLocation;

use serde_json::json;
use snafu::Snafu;

use crate::{
    model::{item::parse_item, FileInfo},
    DRIVE,
};

use super::AppState;

async fn list(State(state): State<Arc<AppState>>, Path(p): Path<String>) -> impl IntoResponse {
    let home_dir = &state.home_dir;
    let p = format!("/{}", p);

    let dir = format!("{}{}", home_dir, p);

    let children = match list_inner(state, dir).await {
        Ok(children) => children,
        Err(e) => return e.into_response(),
    };

    (
        axum::http::StatusCode::OK,
        Json(json!({ "files": *children })),
    )
        .into_response()
}

async fn list_home(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let home_dir = state.home_dir.clone();

    let children = match list_inner(state, home_dir).await {
        Ok(children) => children,
        Err(e) => return e.into_response(),
    };

    (
        axum::http::StatusCode::OK,
        Json(json!({ "files": *children })),
    )
        .into_response()
}

async fn list_inner(state: Arc<AppState>, dir: String) -> Result<Arc<Vec<FileInfo>>, Error> {
    let list_cache = &state.cache.list_cache;
    if let Some(cached) = list_cache.get(&dir) {
        return Ok(cached);
    }

    let item_location = ItemLocation::from_path(&dir).ok_or(Error::LocationNotFound {
        location: dir.to_string(),
    });
    let item_location = match item_location {
        Ok(item_location) => item_location,
        Err(e) => return Err(e),
    };

    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return Err(Error::StillStarting),
    };
    let children = drive.load().drive.list_children(item_location).await;

    match children {
        Ok(children) => {
            let children: Vec<_> = children
                .iter()
                .filter_map(|item| parse_item(item, &state.cache, &state.home_dir).ok())
                .collect();
            let children = Arc::new(children);
            list_cache.insert(dir.to_string(), children.clone());

            Ok(children)
        }
        Err(e) => Err(Error::ListChildren { source: e }),
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
enum Error {
    #[snafu(display("Server still in the process of starting up"))]
    StillStarting,

    #[snafu(display("Location not found: {}", location))]
    LocationNotFound { location: String },

    #[snafu(display("Failed to list the children: {}", source))]
    ListChildren { source: onedrive_api::Error },
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
        .route("/", get(list_home))
        .route("/*path", get(list))
        .with_state(state);

    axum::Router::new().nest("/list", route)
}
