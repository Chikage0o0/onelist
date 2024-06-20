use std::{ops::Deref, path::Path};

use config::Config;
use onedrive_api::Tenant;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use tracing::{info, warn};

use crate::error::{ConfigParseFailedSnafu, Error, WriteConfigFailedSnafu};

static CONFIG_PATH: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Setting {
    pub auth: Auth,
    pub setting: UserSetting,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Auth {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
    pub r#type: ApiType,
}

#[derive(Debug, Clone)]
pub struct ApiType(pub Tenant);
impl Deref for ApiType {
    type Target = Tenant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for ApiType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self.0 {
            Tenant::Consumers => serializer.serialize_str("consumers"),
            Tenant::Organizations => serializer.serialize_str("organizations"),
            Tenant::Common => serializer.serialize_str("common"),
            Tenant::Issuer(ref s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for ApiType {
    fn deserialize<D>(deserializer: D) -> Result<ApiType, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "consumers" => Ok(ApiType(Tenant::Consumers)),
            "organizations" => Ok(ApiType(Tenant::Organizations)),
            "common" => Ok(ApiType(Tenant::Common)),
            _ => Ok(ApiType(Tenant::Issuer(s))),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserSetting {
    pub home_dir: String,
    pub use_proxy: bool,
    pub name: String,
    pub port: u16,
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
        settings.try_deserialize().context(ConfigParseFailedSnafu)
    }

    pub async fn save(&mut self) -> Result<(), Error> {
        // update the refresh token
        if let Some(drive) = crate::DRIVE.get() {
            let refresh_token = drive.load().token.refresh_token.clone();
            if let Some(refresh_token) = refresh_token {
                self.auth.refresh_token = Some(refresh_token);
            }
        }

        let toml = toml::to_string(self).unwrap();

        tokio::fs::write(CONFIG_PATH, toml)
            .await
            .context(WriteConfigFailedSnafu)?;

        Ok(())
    }
}

pub async fn handle_error(e: Error) {
    if let Error::ConfigParseFailed { source: _ } = e {
        let path = Path::new(CONFIG_PATH);
        if path.exists() {
            warn!("Failed to parse the config file, bakup the config file");
            // bakup the config file
            let backup_path = path.with_extension("bak");
            let _ = tokio::fs::rename(path, backup_path).await;
        }

        // create a new config file
        let mut new_config = Setting {
            auth: Auth {
                client_id: "".to_string(),
                client_secret: "".to_string(),
                refresh_token: None,
                r#type: ApiType(Tenant::Consumers),
            },
            setting: UserSetting {
                home_dir: "/".to_string(),
                use_proxy: false,
                name: "Onelist".to_string(),
                port: 3000,
            },
        };

        let _ = new_config.save().await;
        info!("A new config file has been created, please fill in the necessary information");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setting() {
        let mut setting = Setting {
            auth: Auth {
                client_id: "client_id".to_string(),
                client_secret: "client_secret".to_string(),
                refresh_token: None,
                r#type: ApiType(Tenant::Consumers),
            },
            setting: UserSetting {
                home_dir: "/".to_string(),
                use_proxy: false,
                name: "name".to_string(),
                port: 3000,
            },
        };

        setting.save().await.unwrap();

        let loaded_setting = Setting::load().unwrap();
        assert_eq!(setting.auth.client_id, loaded_setting.auth.client_id);

        std::fs::remove_file(CONFIG_PATH).unwrap();
    }
}
