use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json,
};

use onedrive_api::{option::ObjectOption, resource::DriveItemField, ItemLocation};

use serde_json::json;
use snafu::{ResultExt, Snafu};

use crate::{model::item::parse_item, DRIVE};

use super::AppState;

async fn get_item(State(state): State<Arc<AppState>>, Path(p): Path<String>) -> impl IntoResponse {
    let drive = match DRIVE.get() {
        Some(drive) => drive,
        None => return Error::StillStarting.into_response(),
    };

    let home_dir = &state.home_dir;
    let p = format!("/{}", p);
    let dir = format!("{}{}", home_dir, p);

    let cache = &state.cache.file_cache;
    let cached_file = cache.get(&dir);
    let file = if let Some(file) = cached_file {
        file
    } else {
        let item_location = ItemLocation::from_path(&dir).ok_or(Error::LocationNotFound {
            location: dir.to_string(),
        });
        let item_location = match item_location {
            Ok(item_location) => item_location,
            Err(e) => return e.into_response(),
        };
        let option = ObjectOption::default().expand(DriveItemField::thumbnails, None);
        let file = drive
            .load()
            .drive
            .get_item_with_option(item_location, option)
            .await;

        let file = match file {
            Ok(Some(file)) => {
                let item = parse_item(&file, &state.cache, &state.home_dir).context(ParseSnafu);
                match item {
                    Ok(item) => Arc::new(item),
                    Err(e) => return e.into_response(),
                }
            }
            Ok(None) => {
                return Error::GetItemInfo.into_response();
            }

            Err(e) => return Error::GetFile { source: e }.into_response(),
        };
        file
    };

    (axum::http::StatusCode::OK, Json(json!({ "file": *file }))).into_response()
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
enum Error {
    #[snafu(display("Server still in the process of starting up"))]
    StillStarting,

    #[snafu(display("Failed to GetFile: {}", source))]
    GetFile { source: onedrive_api::Error },

    #[snafu(display("Failed to parse the thumb: {}", source))]
    ParseError { source: crate::model::item::Error },

    #[snafu(display("Failed to get item info"))]
    GetItemInfo,

    #[snafu(display("Location not found: {}", location))]
    LocationNotFound { location: String },
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
        .route("/*path", get(get_item))
        .with_state(state);

    axum::Router::new().nest("/info", route)
}
