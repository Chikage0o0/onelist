use std::sync::Arc;

use onedrive_api::resource::DriveItem;
use snafu::Snafu;

use super::{date_time_to_timestamp, thumb, Caches, FileInfo, FileTypes};

pub fn parse_item(item: &DriveItem, caches: &Caches, home_path: &str) -> Result<FileInfo, Error> {
    let id = item
        .id
        .as_ref()
        .ok_or(Error::MissingId { item: item.clone() })?
        .0
        .to_string();
    let name = item.name.to_owned().unwrap_or_default();
    let size = item.size.unwrap_or_default();
    let last_modified_date_time = date_time_to_timestamp(item.last_modified_date_time.to_owned());
    let created_date_time = date_time_to_timestamp(item.created_date_time.to_owned());
    let download_url = item.download_url.to_owned().unwrap_or_default();

    let path = item
        .parent_reference
        .as_ref()
        .and_then(|parent| parent.get("path"))
        .and_then(|path| path.as_str())
        .unwrap_or_default()
        .to_owned()
        .replace("/drive/root:", "");
    let path = if path.starts_with(home_path) {
        path.replace(home_path, "")
    } else {
        path
    };
    let full_path = format!("{}/{}", path, name);

    let mime = item
        .file
        .as_ref()
        .and_then(|file| file.get("mimeType"))
        .and_then(|mime| mime.as_str())
        .unwrap_or_default();
    let file_type = if mime.starts_with("video") {
        FileTypes::Video
    } else if mime.starts_with("audio") {
        FileTypes::Audio
    } else if download_url.is_empty() {
        FileTypes::Folder
    } else {
        FileTypes::File
    };

    // Cache the thumbnail if it exists
    if let Some(thumb) = &item.thumbnails {
        if let Ok(thumb) = thumb::parse_thumb(thumb) {
            caches.thumb_cache.insert(id.clone(), Arc::new(thumb));
        }
    }

    // Cache the download URL if it exists
    if !download_url.is_empty() {
        caches.download_url_cache.insert(id.clone(), download_url);
    }

    let file_info = FileInfo {
        id: id.clone(),
        name,
        size,
        last_modified_date_time,
        created_date_time,
        full_path,
        file_type,
    };

    caches.file_cache.insert(id, Arc::new(file_info.clone()));

    Ok(file_info)
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Missing ID"))]
    MissingId { item: DriveItem },
}
