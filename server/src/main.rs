mod api;
mod auth;
mod data;
mod error;
mod frontend;
mod gamewebsocket;
mod server;
mod session;

use actix::{Actor, ActorTryFutureExt, Addr};
use std::future::Future;
use std::sync::Arc;

use actix_web::middleware::ErrorHandlerResponse::Response;
use actix_web::web::{to, Data};
use actix_web::{web, App, HttpResponse, HttpServer};
use anyhow::Result;
use oauth2::basic::{BasicClient, BasicErrorResponse, BasicTokenResponse};
use oauth2::reqwest::{async_http_client, http_client, Error};
use oauth2::{
    AuthorizationCode, AuthorizationRequest, CsrfToken, HttpRequest, RequestTokenError, Scope,
    TokenResponse,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::auth::{create_oauth_client, Code, DiscordME};
use cult_common::*;

#[actix_web::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1";
    let port = 8000;
    let addr = parse_addr_str(addr, port);
    let oauth_client = create_oauth_client();

    let server = server::GameServer::new().start();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(oauth_client.clone()))
            .app_data(web::Data::new(server.clone()))
            .route("/ws", web::get().to(gamewebsocket::start_ws))
            .route("/discord", web::get().to(auth::discord_oauth))
            .route("/callback", web::get().to(callback))
            .service(api::game_info)
            .service(frontend::index)
            .service(frontend::assets)
            .service(api::session)
            .service(frontend::game)
    })
    .bind(addr)?
    .run();
    println!("Started {} HttpServer! ", addr);
    server.await.expect("Server has crashed!");
    Ok(())
}

impl DiscordME {
    async fn get(token: BasicTokenResponse) -> Option<Self> {
        let request_url = "https://discord.com/api/users/@me";
        let respone = match Client::new()
            .get(request_url)
            .bearer_auth(token.access_token().secret())
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return None,
        };
        let discord_me = match respone.json().await {
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

