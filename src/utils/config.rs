use config::Config;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::error::{ConfigParseFailedSnafu, Error, WriteConfigFailedSnafu};

static CONFIG_PATH: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Setting {
    pub server: Server,
    pub home_dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Server {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
}

impl Setting {
    pub fn load() -> Result<Self, Error> {
        let settings = Config::builder()
            // Add in `./Settings.toml`
            .add_source(config::File::with_name(CONFIG_PATH))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(config::Environment::with_prefix("APP"))
            .build()
            .context(ConfigParseFailedSnafu)?;

        Ok(settings.try_deserialize().context(ConfigParseFailedSnafu)?)
    }

    pub async fn save(&mut self) -> Result<(), Error> {
        // update the refresh token
        if let Some(drive) = crate::DRIVE.get() {
            let refresh_token = drive.load().token.refresh_token.clone();
            if let Some(refresh_token) = refresh_token {
                self.server.refresh_token = Some(refresh_token);
            }
        }

        let toml = toml::to_string(self).unwrap();

        tokio::fs::write(CONFIG_PATH, toml)
            .await
            .context(WriteConfigFailedSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setting() {
        let mut setting = Setting {
            server: Server {
                client_id: "client_id".to_string(),
                client_secret: "client_secret".to_string(),
                refresh_token: None,
            },
            home_dir: "home_dir".to_string(),
        };

        setting.save().await.unwrap();

        let loaded_setting = Setting::load().unwrap();
        assert_eq!(setting.server.client_id, loaded_setting.server.client_id);

        std::fs::remove_file(CONFIG_PATH).unwrap();
    }
}
