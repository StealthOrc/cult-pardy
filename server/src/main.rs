mod apis;
mod frontend;
mod ws;
mod authentication;
mod servers;
mod data;


use crate::apis::api::{board, create_game_lobby, discord_session, has_authorization, join_game};
use crate::apis::api::session_request;

use actix::Addr;
use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, get};
use anyhow::Result;

use apis::api::{get_session, session_data_request, set_session_token_cookie,upload_file_chunk, upload_file_data};
use apis::data::extract_header_string;
use authentication::discord::is_admin;
use dto::DTOFileData;
use servers::authentication::AuthenticationServer;
use servers::db::MongoServer;
use servers::game::GameServer;
use tokio::runtime::Runtime;
use cult_common::*;
use cult_common::backend::JeopardyBoard;
use wasm_lib::JeopardyMode;
use crate::authentication::discord;
use crate::frontend::frontend::{assets, find_game, grant_admin_access, index};
use crate::servers::input::{InputServer};
use crate::servers::Services;
use crate::ws::gamewebsocket;






#[actix_web::main]
async fn main() -> Result<()> {

    let addr = "0.0.0.0";
    let port = 8000;
    let addr = parse_addr_str(addr, port);

    let services = Services::init().await;



    let input_server =  InputServer::init(services.authentication_server.clone());
    let rt = Runtime::new().expect("Somethings wrong with the Runtime");
    rt.spawn(input_server.read_input());


    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(services.mongo_server.clone()))
            .app_data(web::Data::new(services.grant_client.clone()))
            .app_data(web::Data::new(services.login_client.clone()))
            .app_data(web::Data::new(services.game_server.clone()))
            .app_data(web::Data::new(services.authentication_server.clone()))
            .route("/ws", web::get().to(gamewebsocket::start_ws))
            .service(discord::discord_oauth)
            .service(discord::grant_access)
            .service(discord::login_only)
            .service(index)
            .service(find_game)
            .service(assets)
            .service(session_request)
            .service(session_data_request)
            .service(grant_admin_access)
            .service(has_authorization)
            .service(board)
            .service(discord_session)
            .service(create_game_lobby)
            .service(join_game)
            .service(download)
            .service(upload_file_chunk)
            .service(upload_file_data)
            .service(get_file_from_name)
            .default_service(
                web::route().to(not_found)
            )
    })
    .bind(addr)?
    .run();
    println!("Started {} HttpServer! ", addr);
    server.await?;
    rt.shutdown_background();
    Ok(())
}

async fn not_found() -> std::result::Result<HttpResponse, actix_web::Error> {
    let response = HttpResponse::PermanentRedirect()
        .append_header(("Location", format!("{}{}",PROTOCOL,LOCATION)))
        .finish();
    Ok(response)
}



#[get("/api/download")]
async fn download(_req: HttpRequest) -> std::result::Result<HttpResponse, actix_web::Error> {
    let json_data = serde_json::to_string_pretty(&JeopardyBoard::default(JeopardyMode::NORMAL)).expect("Test?");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .append_header(("Content-Disposition", "attachment; filename=test.json"))
        .body(json_data))
}



#[get("/api/file/{name}")]
async fn get_file_from_name(path: web::Path<String>, req: HttpRequest,  db: web::Data<MongoServer> ,srv: web::Data<Addr<GameServer>>, auth: web::Data<Addr<AuthenticationServer>> ) -> Result<HttpResponse, actix_web::Error> {
    let user_session = get_session(&req, &srv).await;

    let name = path.into_inner();
    if name.is_empty() {
        return Ok(HttpResponse::from(HttpResponse::BadRequest()));
    }
        
    if is_admin(user_session.clone(), auth).await == false {
        return Ok(HttpResponse::from(HttpResponse::Unauthorized()));
    }

    let file = db.get_cfile_from_name(&name);

    let mut response = match file {
        None => HttpResponse::from(HttpResponse::NotFound()),
        Some(data) => HttpResponse::from(HttpResponse::Ok().json(data)),
    };

    set_session_token_cookie(&mut response, &req, &user_session);
    Ok(response)
}
