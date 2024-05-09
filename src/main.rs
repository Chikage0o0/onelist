use std::sync::OnceLock;

use arc_swap::ArcSwap;
use onedrive::Onedrive;

use tracing::{info, warn};

use crate::web::web_server;

mod error;
mod onedrive;
mod utils;
mod web;
mod worker;

static DRIVE: OnceLock<ArcSwap<Onedrive>> = OnceLock::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting the program");

    info!("Loading the configuration");
    let mut config = utils::config::Setting::load().unwrap();
    info!("Configuration loaded: {:?}", config);

    let onedrive = Onedrive::new(
        &config.server.client_id,
        &config.server.client_secret,
        &config.server.refresh_token,
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
