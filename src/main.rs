use std::sync::OnceLock;

use arc_swap::ArcSwap;
use onedrive::Onedrive;

use tracing::{info, warn};

use crate::{utils::config::handle_error, web::web_server};

mod error;
mod onedrive;
mod utils;
mod web;
mod worker;

static DRIVE: OnceLock<ArcSwap<Onedrive>> = OnceLock::new();

// For replacing the name of the frontend
static NAME: OnceLock<String> = OnceLock::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting the program");

    info!("Loading the configuration");
    let mut config = match utils::config::Setting::load() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load the configuration: {:?}", e);
            handle_error(e).await;
            panic!("Failed to load the configuration")
        }
    };
    info!("Configuration loaded: {:?}", config);
    NAME.set(config.setting.name.clone()).unwrap();

    let onedrive = Onedrive::new(
        &config.auth.client_id,
        &config.auth.client_secret,
        &config.auth.refresh_token,
    )
    .await;
    DRIVE.set(ArcSwap::from_pointee(onedrive)).unwrap();

    worker::worker();

    web_server(config.clone()).await;

    info!("Saving the configuration");
    match config.save().await {
        Ok(_) => info!("Configuration saved"),
        Err(e) => warn!("Failed to save the configuration: {:?}", e),
    }
}
