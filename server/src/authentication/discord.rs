#![feature(const_trait_impl)]
#![feature(const_trait_impl)]
use std::borrow::ToOwned;
use std::env;
use std::future::Future;
use std::str::FromStr;
use actix::{Addr, MailboxError};
use actix_web::{get, HttpResponse, web};
use actix_web::error::HttpError;
use attohttpc::Method;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, HttpRequest, RedirectUrl, Scope, TokenResponse, TokenUrl};
use oauth2::reqwest::{async_http_client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display};
use cult_common::{DiscordUser, get_false, get_true, JsonPrinter};
use crate::apis::api::{get_session, get_token, remove_cookie, set_cookie, set_session_token_cookie};
use crate::apis::data::{extract_value};
use crate::authentication::discord::DiscordRedirectURL::{Grant, Login};
use crate::servers::authentication::{AddAdminAccess, AuthenticationServer, CheckAdminAccess, RedeemAdminAccessToken};
use crate::servers::game::{AddDiscordAccount, DiscordData, GameServer, UserSession};

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
    let user_session = get_session(&req, &srv).await;

    if let Some(discord_data) = user_session.clone().discord_auth {
        if discord_data.discord_user.is_some() {
            return to_main_page(&user_session,&req)
        }
    }

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

    set_cookie(&mut response, &req,"user-session-id", &user_session.user_session_id.id.to_string());
    set_cookie(&mut response, &req,"session-token", &user_session.session_token.token);
    Ok(response)
}
#[allow(dead_code)]
struct OAuthCallback {
    code: String,
    state: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Code {
    code: Option<String>,
    state: Option<String>
}

impl Code {

    pub fn to_authorization_code(self) ->  AuthorizationCode {
        AuthorizationCode::new(self.code.unwrap_or_default())
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

impl DiscordME{

   pub fn to_discord_user(self) -> DiscordUser{
       DiscordUser{
           id: self.id,
           username: self.username,
           avatar_id: self.avatar,
           discriminator: self.discriminator,
           global_name: self.global_name,
       }

   }

    pub fn redeem_admin_access_token(self, token:usize) -> RedeemAdminAccessToken{
        RedeemAdminAccessToken{
            token,
            discord_id: self.id,
        }
    }


}





impl DiscordME {
    pub async fn get(token: BasicTokenResponse) -> std::option::Option<Self> {
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
        Some(discord_me)
    }
}



async fn add_discord_account(srv:web::Data<Addr<GameServer>>, user_session:UserSession, discord:DiscordUser, token: BasicTokenResponse  ) -> bool{
    let test = srv.send(AddDiscordAccount{
        user_session_id: user_session.user_session_id,
        discord_data: DiscordData{
            discord_user: Some(discord),
            basic_token_response: token
        },
    }).await;
    test.unwrap_or(false)
}





pub(crate) async fn is_admin(user_session: UserSession, auth: web::Data<Addr<AuthenticationServer>>) -> bool{
    let data = match user_session.discord_auth {
        None => return false,
        Some(data) => data,

    };
    let discord_id = match data.discord_user {
        None => return false,
        Some(discord_user) => discord_user.id,
    };
    return auth.send(CheckAdminAccess {
        discord_id,
    }).await.unwrap_or_else(|_| false);
}



#[get("/login")]
pub async fn login_only(
    req: actix_web::HttpRequest,
    srv: web::Data<Addr<GameServer>>,
    auth: web::Data<Addr<AuthenticationServer>>,
    code: web::Query<Code>,
    oauth_client: web::Data<LoginDiscordAuth>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {

    let user_session = get_session(&req, &srv).await;
    let mut response = HttpResponse::InternalServerError().finish();
    let mut printer = JsonPrinter::new();

    if let Some(discord_data) = user_session.clone().discord_auth {
        if discord_data.discord_user.is_some() {
            return to_main_page(&user_session,&req)
        }
    }


    if let Some(discord_token) = get_discord_token(&oauth_client.client, code.into_inner()).await {
        if let Some(discord_me) = DiscordME::get(discord_token.clone()).await {
            let added = add_discord_account(srv, user_session.clone(), discord_me.to_discord_user(), discord_token).await;
            printer.add("Added Discord Account to session", added);
            response = HttpResponse::Found().json(&printer)
        }
    }


    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}


fn to_response(mut response: HttpResponse, req: &actix_web::HttpRequest, user_session: &UserSession) -> anyhow::Result<HttpResponse, actix_web::Error> {
    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}

pub fn to_main_page(user_session: &UserSession, req:&actix_web::HttpRequest) -> anyhow::Result<HttpResponse, actix_web::Error> {
    println!("TO MAIN PAGE");
    let mut response = HttpResponse::Found()
        .append_header(("Location", "http://localhost:8000/"))
        .finish();
    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}




#[get("/grant")]
pub async fn grant_access(
    req: actix_web::HttpRequest,
    srv: web::Data<Addr<GameServer>>,
    code: web::Query<Code>,
    oauth_client: web::Data<GrantDiscordAuth>,
    auth: web::Data<Addr<AuthenticationServer>>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {
    let mut response = HttpResponse::NotFound().finish();
    let user_session = get_session(&req, &srv).await;
    let mut printer = JsonPrinter::new();

    let token = match get_token(&req){
        None => return to_main_page(&user_session,&req),
        Some(token) => token

    };

    printer.add("Has grant Token:", get_true());

    if is_admin(user_session.clone(), auth.clone()).await {
        return to_main_page(&user_session,&req);
    }

    if let Some(discord_data) = user_session.clone().discord_auth {
        if let Some(access_token) = discord_data.clone().redeem_admin_access_token(token).await {
            let result = redeem_admin_access_token(auth, access_token).await;
            printer.add("Found discord session:", get_true());
            printer.add("New discord session:", get_false());
            printer.add("Used Redeem Admin Access Token:", result);
            if let Some(discord_user) = discord_data.clone().discord_user{
                let added = add_discord_account(srv, user_session.clone(), discord_user, discord_data.basic_token_response).await;
                printer.add("Added Discord Account to session", added);
            }
            response = HttpResponse::Found().json(&printer);
        }
    } else if let Some(discord_token) = get_discord_token(&oauth_client.client, code.into_inner()).await {
        if let Some(discord_me) = DiscordME::get(discord_token.clone()).await {
            printer.add("Found discord session:", get_true());
            printer.add("New discord session:", get_true());
            let result = redeem_admin_access_token(auth, discord_me.clone().redeem_admin_access_token(token)).await;
            let added = add_discord_account(srv, user_session.clone(), discord_me.to_discord_user(), discord_token).await;
            printer.add("Added Discord Account to session", added);
            printer.add("Used Redeem Admin Access Token:", result);
            response = HttpResponse::Found().json(&printer);
        }
    }



    set_session_token_cookie(&mut response, &req, &user_session);
    remove_cookie(&mut response, &req, "token");
    Ok(response)
}






async fn redeem_admin_access_token(auth: web::Data<Addr<AuthenticationServer>>, access_token:RedeemAdminAccessToken) -> bool {
    auth.send(access_token).await.unwrap_or_else(|_| false)
}

async fn get_discord_token(basic_client: &BasicClient, code: Code) -> Option<BasicTokenResponse> {
    let token_result = basic_client
        .exchange_code(code.to_authorization_code())
        .request_async(async_http_client)
        .await;
    token_result.ok()
}