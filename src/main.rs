use std::sync::{Mutex, OnceLock};

use onedrive::Onedrive;
use onedrive_api::ItemLocation;
use tracing::info;

mod cache;
mod error;
mod onedrive;
mod utils;

static DRIVE: OnceLock<Mutex<Option<Onedrive>>> = OnceLock::new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting the program");

    info!("Loading the configuration");
    let config = utils::config::Setting::load().unwrap();
    info!("Configuration loaded: {:?}", config);

    let onedrive = Onedrive::new(
        &config.server.client_id,
        &config.server.client_secret,
        &config.server.refresh_token,
    )
    .await;
    DRIVE.set(Mutex::new(Some(onedrive))).unwrap();

    let drive = onedrive::get_drive().await;
    println!("{:?}", drive.get_drive().await.unwrap());

    let item = drive.get_item(ItemLocation::root()).await.unwrap();
    println!("{:?}", item);

    info!("Saving the configuration");
    config.save().await.unwrap();
    info!("Configuration saved");
}
