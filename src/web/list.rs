use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json,
};
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
    download_url: String,
}

async fn list(State(state): State<Arc<AppState>>, Path(p): Path<String>) -> impl IntoResponse {
    let home_dir = &state.home_dir;

    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return ListError::StillStarting.into_response(),
    };

    let p = format!("/{}", p);

    let dir = format!("{}{}", home_dir, p);
    let cache = &state.list_cache;
    if let Some(cached) = cache.get(&dir) {
        return (
            axum::http::StatusCode::OK,
            Json(json!({ "files": *cached })),
        )
            .into_response();
    }

    let item_location = ItemLocation::from_path(&dir).ok_or(ListError::LocationNotFound {
        location: dir.to_string(),
    });

    let item_location = match item_location {
        Ok(item_location) => item_location,
        Err(e) => return e.into_response(),
    };

    let children = drive.load().drive.list_children(item_location).await;

    match children {
        Ok(children) => {
            let children = Arc::new(map_item(children));
            cache.insert(home_dir.to_string(), children.clone());

            (
                axum::http::StatusCode::OK,
                Json(json!({ "files": *children })),
            )
                .into_response()
        }
        Err(e) => ListError::ListChildren { source: e }.into_response(),
    }
}

async fn list_home(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let home_dir = &state.home_dir;

    let list_cache = &state.list_cache;
    let download_cache = &state.download_cache;
    if let Some(cached) = list_cache.get(home_dir) {
        return (
            axum::http::StatusCode::OK,
            Json(json!({ "files": *cached })),
        )
            .into_response();
    }

    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return ListError::StillStarting.into_response(),
    };

    let item_location = ItemLocation::from_path(&home_dir).ok_or(ListError::LocationNotFound {
        location: home_dir.to_string(),
    });

    let item_location = match item_location {
        Ok(item_location) => item_location,
        Err(e) => return e.into_response(),
    };

    let children = drive.load_full().drive.list_children(item_location).await;

    match children {
        Ok(children) => {
            let children = Arc::new(map_item(children));
            list_cache.insert(home_dir.to_string(), children.clone());
            for child in children.iter() {
                if !child.download_url.is_empty() && !child.id.is_empty() {
                    download_cache.insert(child.id.clone(), child.download_url.clone());
                }
            }

            (
                axum::http::StatusCode::OK,
                Json(json!({ "files": *children })),
            )
                .into_response()
        }
        Err(e) => ListError::ListChildren { source: e }.into_response(),
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
enum ListError {
    #[snafu(display("Server still in the process of starting up"))]
    StillStarting,

    #[snafu(display("Location not found: {}", location))]
    LocationNotFound { location: String },

    #[snafu(display("Failed to list the children: {}", source))]
    ListChildren { source: onedrive_api::Error },
}

impl IntoResponse for ListError {
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

fn map_item(item: Vec<DriveItem>) -> Vec<FileInfo> {
    item.into_iter()
        .map(|child| FileInfo {
            name: child.name.unwrap_or_default(),
            size: child.size.unwrap_or_default(),
            id: child.id.and_then(|id| id.0.into()).unwrap_or_default(),
            last_modified_date_time: date_time_to_timestamp(child.last_modified_date_time),
            created_date_time: date_time_to_timestamp(child.created_date_time),
            download_url: child.download_url.unwrap_or_default(),
        })
        .collect()
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
