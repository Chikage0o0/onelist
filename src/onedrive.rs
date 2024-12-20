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
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info};
use url::Url;

use crate::utils::config::Setting;

#[derive(Debug)]
pub struct Onedrive {
    pub auth: onedrive_api::Auth,
    pub client_secret: String,
    pub token: Token,
    pub drive: onedrive_api::OneDrive,
}
#[derive(Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Instant,
}

impl Onedrive {
    pub async fn new(config: &Setting) -> Self {
        let auth = onedrive_api::Auth::new(
            config.auth.client_id.clone(),
            onedrive_api::Permission::new_read().offline_access(true),
            "http://localhost:8077/redirect",
            config.auth.r#type.0.clone(),
        );

        // refresh or login
        let token = if let Some(refresh_token) = &config.auth.refresh_token {
            info!("refresh_token is found, refresh");
            Self::login_with_refresh_token(&auth, &config.auth.client_secret, refresh_token).await
        } else {
            info!("refresh_token is not found, login");
            Self::login(&auth, &config.auth.client_secret).await
        }
        .unwrap_or_else(|e| {
            panic!("Failed to login or refresh: {:?}", e);
        });

        let drive =
            onedrive_api::OneDrive::new(&token.access_token, onedrive_api::DriveLocation::me());

        Self {
            auth,
            client_secret: config.auth.client_secret.to_string(),
            token,
            drive,
        }
    }

    async fn login(auth: &Auth, client_secret: &str) -> Result<Token, Error> {
        let url = auth.code_auth_url();
        println!("Open the following URL in your browser:\n{}", url);

        // temporary webserver to get the code
        let listener = TcpListener::bind("0.0.0.0:8077")
            .await
            .context(BindFailedSnafu)?;

        let code: String;
        if let Ok((stream, _)) = listener.accept().await {
            code = handle_connection(stream).await.context(BindFailedSnafu)?;
        } else {
            return Err(Error::GetRedirectUrl {
                source: io::Error::new(io::ErrorKind::Other, "Failed to get the redirect url"),
            });
        }

        let code = parse_code(&code).ok_or(ParseCodeSnafu { s: code }.build())?;
        let client_secret = ClientCredential::Secret(client_secret.to_string());
        // Get the token from the code
        let token = auth
            .login_with_code(&code, &client_secret)
            .await
            .context(RefreshTokenSnafu)?
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

    #[allow(dead_code)]
    pub async fn refresh(&self) -> Result<Self, Error> {
        debug!("Refreshing the token");
        let new_token = Self::login_with_refresh_token(
            &self.auth,
            &self.client_secret,
            self.token.refresh_token.as_ref().unwrap(),
        )
        .await?;
        let new_drive =
            onedrive_api::OneDrive::new(&new_token.access_token, onedrive_api::DriveLocation::me());
        Ok(Self {
            token: new_token,
            drive: new_drive,
            auth: self.auth.clone(),
            client_secret: self.client_secret.clone(),
        })
    }
}

impl From<TokenResponse> for Token {
    fn from(val: TokenResponse) -> Self {
        Token {
            access_token: val.access_token,
            refresh_token: val.refresh_token,
            expires_at: Instant::now() + Duration::from_secs(val.expires_in_secs),
        }
    }
}

fn parse_code(url: &str) -> Option<String> {
    let url = Url::parse(format!("http://localhost:8077{}", url).as_str());
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
    RefreshToken { source: onedrive_api::Error },

    #[snafu(display("Failed to parse the code {}", s))]
    ParseCode { s: String },

    #[snafu(display("Failed to login {}", source))]
    GetRedirectUrl { source: io::Error },

    #[snafu(display("Failed to bind the 10080 port {}", source))]
    BindFailed { source: io::Error },
}

async fn handle_connection(mut stream: TcpStream) -> Result<String, std::io::Error> {
    // 创建缓冲区
    let mut buffer = [0; 1024];

    // 异步读取数据
    match stream.read(&mut buffer).await {
        Ok(n) => {
            let request = String::from_utf8_lossy(&buffer[..n]);

            // 解析请求的第一行来获取 URI
            let request_line = request.lines().next().unwrap_or("");
            let uri = request_line.split_whitespace().nth(1).unwrap_or("/");

            // 构建响应
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nYou requested: {}",
                uri
            );

            // 异步发送响应
            stream.write_all(response.as_bytes()).await?;
            stream.flush().await?;
            return Ok(uri.to_string());
        }
        Err(e) => return Err(e),
    }
}
