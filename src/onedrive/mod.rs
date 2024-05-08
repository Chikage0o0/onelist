mod login;

use std::time::Instant;

pub use login::Onedrive;

use onedrive_api::DriveLocation;

pub async fn get_drive() -> onedrive_api::OneDrive {
    let mut drive = crate::DRIVE.get().unwrap().lock().unwrap();
    let expires_at = drive.as_ref().unwrap().token.expires_at;
    if expires_at < Instant::now() {
        let drive_inner = drive.take().unwrap();
        let new_drive = drive_inner.refresh().await.unwrap();
        *drive = Some(new_drive);
    }

    onedrive_api::OneDrive::new(
        &drive.as_ref().unwrap().token.access_token,
        DriveLocation::me(),
    )
}
