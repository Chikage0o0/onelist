//! OneDrive storage implementation
//! How to get the client_id and client_secret:
//! 1. Go to https://entra.microsoft.com/
//! 2. Create a new app registration
//! 3. Go to the app registration and create a new client secret
//! 4. Use the client id and client secret as the credentials
//! 5. Add the following permissions to the app registration:
//!    - Files.ReadWrite
//!    - offline_access
//!    - User.Read

use std::{
    io,
    time::{Duration, Instant},
};

use onedrive_api::{Auth, ClientCredential, TokenResponse};
use snafu::{ResultExt, Snafu};
use tracing::info;
use url::Url;

#[derive(Debug)]
pub struct Onedrive {
    pub auth: onedrive_api::Auth,
    pub client_secret: String,
    pub token: Token,
}
#[derive(Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Instant,
}

impl Onedrive {
    pub async fn new(client_id: &str, client_secret: &str, refresh_token: &Option<String>) -> Self {
        let auth = onedrive_api::Auth::new(
            client_id,
            onedrive_api::Permission::new_read()
                .offline_access(true)
                .write(true),
            "http://localhost:10080/redirect",
            onedrive_api::Tenant::Organizations,
        );

        // refresh or login
        let token = if let Some(refresh_token) = refresh_token {
            info!("refresh_token is found, refresh");
            Self::login_with_refresh_token(&auth, client_secret, refresh_token).await
        } else {
            info!("refresh_token is not found, login");
            Self::login(&auth, client_secret).await
        }
        .unwrap_or_else(|e| {
            panic!("Failed to login or refresh: {:?}", e);
        });

        Self {
            auth,
            client_secret: client_secret.to_string(),
            token,
        }
    }

    async fn login(auth: &Auth, client_secret: &str) -> Result<Token, Error> {
        let url = auth.code_auth_url();
        println!("Open the following URL in your browser:\n{}", url);

        println!("Please enter the redirect URL:");
        let input = tokio::task::spawn_blocking(move || {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .context(GetRedirectUrlFailedSnafu)?;
            Ok(input)
        });

        let input = input.await.unwrap()?;

        let code = parse_code(&input).ok_or(ParseCodeFailedSnafu { s: input }.build())?;
        let client_secret = ClientCredential::Secret(client_secret.to_string());
        // Get the token from the code
        let token = auth
            .login_with_code(&code, &client_secret)
            .await
            .context(RefreshTokenFailedSnafu)?
            .into();

        Ok(token)
    }

    async fn login_with_refresh_token(
        auth: &Auth,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<Token, Error> {
        let credential = ClientCredential::Secret(client_secret.to_string());
        let ret = auth
            .login_with_refresh_token(refresh_token, &credential)
            .await;

        match ret {
            Ok(token) => Ok(token.into()),
            Err(e) => {
                println!("Failed to refresh the token: {}", e);
                println!("Please login again");
                Self::login(auth, client_secret).await
            }
        }
    }

    pub async fn refresh(self) -> Result<Self, Error> {
        let new_token = Self::login_with_refresh_token(
            &self.auth,
            &self.client_secret,
            &self.token.refresh_token.as_ref().unwrap(),
        )
        .await?;
        Ok(Self {
            token: new_token,
            ..self
        })
    }
}

impl Into<Token> for TokenResponse {
    fn into(self) -> Token {
        Token {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_at: Instant::now() + Duration::from_secs(self.expires_in_secs),
        }
    }
}

fn parse_code(url: &str) -> Option<String> {
    let url = Url::parse(url);
    let url = match url {
        Ok(url) => url,
        Err(_) => return None,
    };
    // Rest of the code...
    let pairs = url.query_pairs();

    for (key, value) in pairs {
        if key == "code" {
            return Some(value.to_string());
        }
    }

    None
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Failed to refresh the token {}", source))]
    RefreshTokenFailed { source: onedrive_api::Error },

    #[snafu(display("Failed to parse the code {}", s))]
    ParseCodeFailed { s: String },

    #[snafu(display("Failed to login {}", source))]
    GetRedirectUrlFailed { source: io::Error },
}
