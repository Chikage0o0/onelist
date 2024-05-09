use std::sync::Arc;

use tracing::{debug, error, info};

use crate::DRIVE;

pub fn worker() {
    // Automatically refresh the token when it expires
    tokio::spawn(async {
        info!("Starting the worker");
        auto_refresh().await;
    });
}

async fn auto_refresh() {
    loop {
        if let Some(drive) = DRIVE.get() {
            let drive_load = drive.load();
            let expires_at = drive_load.token.expires_at;
            let now = std::time::Instant::now();
            let duration = expires_at.duration_since(now) - std::time::Duration::from_secs(60);
            debug!("Token will be refreshed in {:?}", duration);
            tokio::time::sleep(duration).await;

            debug!("Refreshing the token");

            let drive_load = drive.load_full();
            let ret = drive_load.refresh().await;
            match ret {
                Ok(d) => {
                    DRIVE.get().unwrap().store(Arc::new(d));
                    debug!("Token refreshed");
                }
                Err(e) => {
                    error!("Failed to refresh the token: {:?}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
            }
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}
