use std::env;
use actix_web::{HttpResponse, web};
use attohttpc::Method;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl};
use oauth2::reqwest::{async_http_client};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn create_oauth_client() -> BasicClient {

    println!("CULT_PARDY_CLIENT_ID is? {:?}", env::var("CULT_PARDY_CLIENT_ID"));
    println!("CULT_PARDY_CLIENT_SECRET is? {:?}", env::var("CULT_PARDY_CLIENT_SECRET"));

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
#[allow(dead_code)]
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

impl DiscordME {
    async fn get(token: BasicTokenResponse) -> std::option::Option<Self> {
        let request_url = "https://discord.com/api/users/@me";

        let request = attohttpc::RequestBuilder::new(Method::GET, request_url).bearer_auth(token.access_token().secret());
        let respone = match request.send() {
            Ok(resonse) => resonse,
            Err(error) => {
                println!("Something wrong: {:?}", error);
                return None
            }
        };
        let discord_me = match respone.json() {
            Ok(me) => me,
            Err(_) => return None,
        };
        println!("Created {:?}", discord_me);
        Some(discord_me)
    }
}

pub async fn callback(
    code: web::Query<Code>,
    oauth_client: web::Data<BasicClient>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {
    let token_result = oauth_client
        .exchange_code(code.into_inner().to_authorization_code())
        .request_async(async_http_client)
        .await;
    match token_result {
        Ok(token) => {
            println!("{:?}", token.access_token().secret());
            match DiscordME::get(token).await {
                None => Ok(HttpResponse::InternalServerError().body(format!("Error"))),
                Some(discord) => Ok(HttpResponse::Found().json(discord)),
            }
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error: {:?}", e))),
    }
}