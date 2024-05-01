#![feature(const_trait_impl)]
#![feature(const_trait_impl)]
use std::borrow::ToOwned;
use std::env;
use std::fmt::Display;
use std::str::FromStr;
use actix::Addr;
use actix_web::{get, HttpResponse, web};
use attohttpc::Method;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, HttpRequest, RedirectUrl, Scope, TokenResponse, TokenUrl};
use oauth2::reqwest::{async_http_client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display, EnumString};
use crate::apis::api::{get_session, set_session_cookies};
use crate::apis::data::{extract_header_string, extract_value};
use crate::authentication::auth::DiscordRedirectURL::{Grant, Login};
use crate::servers::game::GameServer;

#[derive(Clone, Display,Debug)]
enum DiscordRedirectURL{
    Grant,
    Login
}


#[derive(Clone,Debug)]
pub struct LoginDiscordAuth {
    client: BasicClient,
}
#[derive(Clone,Debug)]
pub struct GrantDiscordAuth {
    client: BasicClient,
}

impl LoginDiscordAuth {
    pub fn init() -> Self {
        LoginDiscordAuth{
            client : Self::create_oauth_client(),
        }
    }
}

impl GrantDiscordAuth {
    pub fn init() -> Self {
        GrantDiscordAuth{
            client : Self::create_oauth_client(),
        }
    }
}

impl FromStr for DiscordRedirectURL{

    type Err = ();

    fn from_str(input: &str) -> Result<DiscordRedirectURL, ()>{
        match input.to_uppercase().as_str() {
             "GRANT" => Ok(Grant),
             "LOGIN" => Ok(Login),
            _ => Err(()),
        }
    }
}





trait DiscordAuth{


    const REDIRECT_URL: DiscordRedirectURL;
    const AUTHORIZATION_URL: &'static str = "https://discord.com/api/oauth2/authorize";
    const TOKEN_URL: &'static str = "https://discord.com/api/oauth2/token";

    fn create_oauth_client() -> BasicClient {

        println!("CULT_PARDY_CLIENT_ID = {:?}", env::var("CULT_PARDY_CLIENT_ID"));
        println!("CULT_PARDY_CLIENT_SECRET = {:?}", env::var("CULT_PARDY_CLIENT_SECRET"));

        BasicClient::new(
            ClientId::new(env::var("CULT_PARDY_CLIENT_ID").unwrap_or_else(|_| "NOT SET".to_string())),
            Some(ClientSecret::new(env::var("CULT_PARDY_CLIENT_SECRET").unwrap_or_else(|_| "NOT SET".to_string()))),
            AuthUrl::new(Self::AUTHORIZATION_URL.to_owned()).expect("AuthUrl"),
            Some(TokenUrl::new(Self::TOKEN_URL.to_owned()).expect("TokenUrl"))
        ).set_redirect_uri(RedirectUrl::new(format!("http://localhost:8000/{}",Self::REDIRECT_URL.to_string().to_lowercase())).expect("Invalid redirect URL"))
    }

}


impl DiscordAuth for GrantDiscordAuth {
    const REDIRECT_URL: DiscordRedirectURL = Grant;
}
impl DiscordAuth for LoginDiscordAuth{
    const REDIRECT_URL: DiscordRedirectURL = Login;
}








#[get("/discord")]
pub async fn discord_oauth(
    req: actix_web::HttpRequest,
    grant:web::Data<GrantDiscordAuth>,
    login: web::Data<LoginDiscordAuth>,
    srv: web::Data<Addr<GameServer>>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {

    let user_session_id = get_session(&req, &srv).await;

    let typ = match extract_value(&req, "type") {
        Ok(data) => {
            let typ = DiscordRedirectURL::from_str(data.as_str()).unwrap_or(Login);
            match typ {
                Grant => &grant.client,
                Login => &login.client
            }
        },
        Err(error) => &login.client
    };



    let (authorize_url, _) = typ
        .authorize_url(move || CsrfToken::new_random())
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    let mut response = HttpResponse::Found()
        .append_header(("Location", authorize_url.to_string()))
        .finish();

    set_session_cookies(
        &mut response,
        "user-session-id",
        &user_session_id.to_string(),
    );
    Ok(response)
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
#[get("/login")]
pub async fn login_only(
    code: web::Query<Code>,
    oauth_client: web::Data<LoginDiscordAuth>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {
    let token_result = oauth_client.client
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

#[get("/grant")]
pub async fn grant_access(
    code: web::Query<Code>,
    oauth_client: web::Data<GrantDiscordAuth>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {
    println!("?");



    let token_result = oauth_client.client
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