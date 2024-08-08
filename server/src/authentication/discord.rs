#![feature(const_trait_impl)]
#![feature(const_trait_impl)]
use std::borrow::ToOwned;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use actix::{Addr};
use actix_web::{get, HttpResponse, web};
use attohttpc::Method;
use cult_common::wasm_lib::DiscordUser;
use cult_common::{get_false, get_true, JsonPrinter, LOCATION, PROTOCOL};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl};
use oauth2::reqwest::{async_http_client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display};
use cult_common::wasm_lib::ids::discord::DiscordID;
use crate::apis::api::{get_session_or_create_new, get_session_or_create_new_session_request, get_token, remove_cookie, set_cookie, set_session_token_cookie};
use crate::apis::data::{extract_value};
use crate::authentication::discord::DiscordRedirectURL::{Grant, Login};
use crate::servers::authentication::{AddDiscordAccount, AuthenticationServer, CheckAdminAccess, DiscordAccountStatus, RedeemAdminAccessToken};
use crate::servers::db::MongoServer;
use crate::servers::game::{DiscordData, GameServer, UserSession};

#[derive(Clone, Display,Debug)]
enum DiscordRedirectURL{
    Grant,
    Login
}


#[derive(Clone,Debug)]
pub struct LoginDiscordAuth {
    client: BasicClient,
}
#[derive(Clone,Debug )]
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
        ).set_redirect_uri(RedirectUrl::new(format!("{}{}/{}",PROTOCOL,LOCATION,Self::REDIRECT_URL.to_string().to_lowercase())).expect("Invalid redirect URL"))
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
    grant:web::Data<Arc<GrantDiscordAuth>>,
    login: web::Data<Arc<LoginDiscordAuth>>,
    db: web::Data<Arc<MongoServer>>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {
    let user_session = get_session_or_create_new(&req, &db).await;
    if let Some(discord_data) = user_session.clone().discord_auth {
        if discord_data.discord_user.is_some() {
            return to_main_page(&user_session)
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
        Err(_error) => &login.client
    };



    let (authorize_url, _) = typ
        .authorize_url(move || CsrfToken::new_random())
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    let mut response = HttpResponse::Found()
        .append_header(("Location", authorize_url.to_string()))
        .finish();

    set_session_token_cookie(&mut response, &user_session);
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
    pub accent_color: Option<i64>,
    #[serde(rename = "global_name")]
    pub global_name: String,
    #[serde(rename = "avatar_decoration_data")]
    pub avatar_decoration_data: Value,
    #[serde(rename = "banner_color")]
    pub banner_color: Option<String>,
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
           discord_id: DiscordID::new(self.id),
           username: self.username,
           avatar_id: self.avatar,
           discriminator: self.discriminator,
           global_name: self.global_name,
       }

   }

    pub fn redeem_admin_access_token(self, token:usize) -> RedeemAdminAccessToken{
        RedeemAdminAccessToken{
            token,
            discord_id: DiscordID::new(self.id),
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
            Err(error) =>  {
                println!("Something wrong: {:?}", error);
                return None
            }
        };
        Some(discord_me)
    }
}



async fn add_discord_account(srv:&web::Data<Addr<AuthenticationServer>>, user_session:UserSession, discord:DiscordUser, token: BasicTokenResponse  ) -> DiscordAccountStatus {
    let added = srv.send(AddDiscordAccount{
        user_session_id: user_session.user_session_id,
        discord_data: DiscordData{
            discord_user: Some(discord),
            basic_token_response: token
        },
    }).await;


    added.unwrap_or(DiscordAccountStatus::NotAdded)
}





pub async fn is_admin(user_session: &UserSession, db: &web::Data<Arc<MongoServer>>) -> bool{
    let discord_id = match user_session.get_discord_id(){
        Some(id) => id,
        None => return false,
    };
    db.is_admin(&discord_id).await
}



#[get("/login")]
pub async fn login_only(
    req: actix_web::HttpRequest,
    _auth: web::Data<Addr<AuthenticationServer>>,
    code: web::Query<Code>,
    oauth_client: web::Data<Arc<LoginDiscordAuth>>,
    db: web::Data<Arc<MongoServer>>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {

    let mut user_session = get_session_or_create_new(&req, &db).await;

    let mut response = HttpResponse::Found()
        .append_header(("Location", format!("{}{}",PROTOCOL,LOCATION)))
        .finish();
    if let Some(discord_data) = user_session.clone().discord_auth {
        if discord_data.discord_user.is_some() {
            return to_main_page(&user_session)
        }
    }
    println!("Usersession_id: {:?}", &user_session.user_session_id);

    if let Some(discord_token) = get_discord_token(&oauth_client.client, code.into_inner()).await {
        if let Some(discord_me) = DiscordME::get(discord_token.clone()).await {
            let status : DiscordAccountStatus = add_discord_account(&_auth, user_session.clone(), discord_me.to_discord_user(), discord_token).await;
            match status {
                DiscordAccountStatus::Updated(update) => {
                    println!("Updated Discord Account: {:?}", update);
                    user_session = get_session_or_create_new_session_request(&update, &db).await;
                },
                _ => (),
            }
        }
    }
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}


fn to_response(mut response: HttpResponse,  user_session: &UserSession) -> anyhow::Result<HttpResponse, actix_web::Error> {
    set_session_token_cookie(&mut response,  &user_session);
    Ok(response)
}

pub fn to_main_page(user_session: &UserSession) -> anyhow::Result<HttpResponse, actix_web::Error> {
    println!("TO MAIN PAGE");
    let mut response = HttpResponse::Found()
        .append_header(("Location", format!("{}{}",PROTOCOL,LOCATION)))
        .finish();
    set_session_token_cookie(&mut response,  &user_session);
    Ok(response)
}




#[get("/grant")]
pub async fn grant_access(
    req: actix_web::HttpRequest,
    code: web::Query<Code>,
    oauth_client: web::Data<Arc<GrantDiscordAuth>>,
    auth: web::Data<Addr<AuthenticationServer>>,
    db: web::Data<Arc<MongoServer>>,
) -> anyhow::Result<HttpResponse, actix_web::Error> {
    let mut response = HttpResponse::NotFound().finish();
    let mut user_session = get_session_or_create_new(&req, &db).await;
    let mut printer = JsonPrinter::new();

    let token = match get_token(&req){
        None => return to_main_page(&user_session),
        Some(token) => token

    };

    printer.add("Has grant Token:", get_true());

    if is_admin(&user_session, &db).await {
        return to_main_page(&user_session);
    }

    if let Some(discord_data) = user_session.clone().discord_auth {
        if let Some(access_token) = discord_data.clone().redeem_admin_access_token(token).await {
            let result = redeem_admin_access_token(&auth, access_token).await;
            printer.add("Found discord session:", get_true());
            printer.add("New discord session:", get_false());
            printer.add("Used Redeem Admin Access Token:", result);
            if let Some(discord_user) = discord_data.clone().discord_user{
                let status = add_discord_account(&auth, user_session.clone(), discord_user, discord_data.basic_token_response).await;
                match status.clone() {
                    DiscordAccountStatus::Updated(update) => {
                        println!("Updated Discord Account: {:?}", update);
                        user_session = get_session_or_create_new_session_request(&update, &db).await;
                    },
                    _ => (),
                }
                printer.add("Added Discord Account to session", status.to_help());
            }
            response = HttpResponse::Found().json(&printer);
        }
    } else if let Some(discord_token) = get_discord_token(&oauth_client.client, code.into_inner()).await {
        if let Some(discord_me) = DiscordME::get(discord_token.clone()).await {
            printer.add("Found discord session:", get_true());
            printer.add("New discord session:", get_true());
            let result = redeem_admin_access_token(&auth, discord_me.clone().redeem_admin_access_token(token)).await;
            let status = add_discord_account(&auth, user_session.clone(), discord_me.to_discord_user(), discord_token).await;
            match status.clone() {
                DiscordAccountStatus::Updated(update) => {
                    println!("Updated Discord Account: {:?}", update);
                    user_session =  get_session_or_create_new_session_request(&update, &db).await;
                },
                _ => (),
            }
            printer.add("Added Discord Account to session", status.to_help());
            printer.add("Used Redeem Admin Access Token:", result);
            response = HttpResponse::Found().json(&printer);
        }
    }



    set_session_token_cookie(&mut response, &user_session);
    remove_cookie(&mut response, &req, "token");
    Ok(response)
}






async fn redeem_admin_access_token(auth: &web::Data<Addr<AuthenticationServer>>, access_token:RedeemAdminAccessToken) -> bool {
    auth.send(access_token).await.unwrap_or_else(|_| false)
}

async fn get_discord_token(basic_client: &BasicClient, code: Code) -> Option<BasicTokenResponse> {
    let token_result = basic_client
        .exchange_code(code.to_authorization_code())
        .request_async(async_http_client)
        .await;
    token_result.ok()
}