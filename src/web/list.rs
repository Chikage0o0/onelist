use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json,
};
use mini_moka::sync::Cache;
use onedrive_api::{resource::DriveItem, ItemLocation};
use serde::Serialize;
use serde_json::json;
use snafu::Snafu;

use crate::DRIVE;

use super::AppState;

#[derive(Debug, Serialize)]
pub struct FileInfo {
    id: String,
    name: String,
    size: i64,
    last_modified_date_time: i64,
    created_date_time: i64,
    #[serde(rename = "type")]
    file_type: FileTypes,
}
#[derive(Debug, Serialize)]
enum FileTypes {
    File,
    Folder,
}

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
    let list_cache = &state.list_cache;
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
            let children = Arc::new(map_item(children, Some(&state.download_cache)));
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

fn map_item(item: Vec<DriveItem>, cache: Option<&Cache<String, String>>) -> Vec<FileInfo> {
    let iter = item.into_iter().map(|child| {
        (
            child.name.unwrap_or_default(),
            child.size.unwrap_or_default(),
            child.id.and_then(|id| id.0.into()).unwrap_or_default(),
            date_time_to_timestamp(child.last_modified_date_time),
            date_time_to_timestamp(child.created_date_time),
            child.download_url.unwrap_or_default(),
        )
    });

    let iter = if let Some(cache) = cache {
        iter.map(
            |(name, size, id, last_modified_date_time, created_date_time, download_url)| {
                if !download_url.is_empty() && !id.is_empty() {
                    cache.insert(id.clone(), download_url.clone());
                }
                let file_type = if download_url.is_empty() {
                    FileTypes::Folder
                } else {
                    FileTypes::File
                };

                FileInfo {
                    id,
                    name,
                    size,
                    last_modified_date_time,
                    created_date_time,
                    file_type,
                }
            },
        )
        .collect()
    } else {
        iter.map(
            |(name, size, id, last_modified_date_time, created_date_time, download_url)| {
                let file_type = if download_url.is_empty() {
                    FileTypes::Folder
                } else {
                    FileTypes::File
                };
                FileInfo {
                    id,
                    name,
                    size,
                    last_modified_date_time,
                    created_date_time,
                    file_type,
                }
            },
        )
        .collect()
    };

    iter
}

fn date_time_to_timestamp(date_time: Option<String>) -> i64 {
    date_time
        .and_then(|date_time| {
            chrono::DateTime::parse_from_rfc3339(&date_time)
                .ok()
                .map(|date_time| date_time.timestamp())
        })
        .unwrap_or_default()
}
