#![feature(const_trait_impl)]
#![feature(const_trait_impl)]

use std::env;
use std::env::VarError;
use actix_web::{HttpResponse, web};
use oauth2::basic::{BasicClient, BasicErrorResponse, BasicTokenResponse};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, ErrorResponse, RedirectUrl, RequestTokenError, reqwest, Scope, TokenResponse, TokenUrl};
use oauth2::reqwest::{async_http_client, Error, http_client};
use oauth2::url::Url;
use serde::de::Unexpected::Option;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn create_oauth_client() -> BasicClient {

    println!("{:?}", env::var("CULT_PARDY_CLIENT_ID"));

    let client_id: String = env::var("CULT_PARDY_CLIENT_ID").unwrap_or_else(|_| "NOT SET".to_string());

    let client_secret: String = env::var("CULT_PARDY_CLIENT_SECRET").unwrap_or_else(|_| "NOT SET".to_string());
    const AUTHORIZATION_URL: &str = "https://discord.com/api/oauth2/authorize";
    const TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
    const REDIRECT_URL: &str = "http://localhost:8000/callback";

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(AUTHORIZATION_URL.to_string()).expect("AuthUrl"),
        Some(TokenUrl::new(TOKEN_URL.to_string()).expect("TokenUrl"))
    ).set_redirect_uri(RedirectUrl::new(REDIRECT_URL.to_string()).expect("Invalid redirect URL"))
}


pub async fn discord_oauth(
    oauth_client: web::Data<BasicClient>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {
    let (authorize_url, _) = oauth_client
        .authorize_url(move || CsrfToken::new_random())
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(HttpResponse::Found()
        .append_header(("Location", authorize_url.to_string()))
        .finish())
}

struct OAuthCallback {
    code: String,
    state: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Code {
    code: String,
    state: String
}

impl Code {

    pub fn to_authorization_code(self) ->  AuthorizationCode {
        AuthorizationCode::new(self.code)
    }


}




#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordME {
    pub id: String,
    pub username: String,
    pub avatar: String,
    pub discriminator: String,
    #[serde(rename = "public_flags")]
    pub public_flags: i64,
    pub flags: i64,
    pub banner: Value,
    #[serde(rename = "accent_color")]
    pub accent_color: i64,
    #[serde(rename = "global_name")]
    pub global_name: String,
    #[serde(rename = "avatar_decoration_data")]
    pub avatar_decoration_data: Value,
    #[serde(rename = "banner_color")]
    pub banner_color: String,
    pub clan: Value,
    #[serde(rename = "mfa_enabled")]
    pub mfa_enabled: bool,
    pub locale: String,
    #[serde(rename = "premium_type")]
    pub premium_type: i64,
    pub email: String,
    pub verified: bool,
}
