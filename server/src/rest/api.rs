use std::sync::Arc;
use crate::rest::error::{ApiError, ApiGameError, ApiRequestError, ApiSessionError, ToApiError, ToResponse};
use crate::data::{SessionRequest};
use crate::services::db::MongoServer;
use crate::services::game::{CreateLobby, FileMetadata};
use crate::services::lobby::CanJoinLobby;
use crate::settings::Settings;
use actix::Addr;

use actix_multipart::Multipart;
use actix_web::{get, HttpRequest, HttpResponse, post, web};
use bson::Bson;
use chrono::Local;
use cult_common::dto::api::{ApiResponse};
use cult_common::dto::file::{FileMultiPart};
use cult_common::wasm_lib::hashs::validate::ValidateHash;
use cult_common::wasm_lib::{DiscordUser, JeopardyMode};
use futures::{AsyncWriteExt, StreamExt};
use mongodb::gridfs::GridFsUploadStream;
use serde::Serialize;
use serde_json::json;
use cult_common::backend::JeopardyBoard;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use utoipa::ToSchema;
use crate::rest::data::{extract_header_string, extract_value, get_internal_server_error_json, get_lobby_id_from_header, get_session, get_session_with_token_update_or_create_new, set_session_token_cookie};
use crate::authentication::discord::is_admin;
use crate::services::game;
use crate::services::game::UserSession;

use super::data;
use super::error::ApiFileError;

#[utoipa::path(
    get,
    path = "/api/info",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
        ("lobby-id" = String, Header, description = "Lobby ID")
    ),
    responses(
        (status = 200, description = "Lobby exits", body = String),
        (status = 404, description = "No Lobby found", body = ApiError),
        (status = 500, description = "Game error", body = ApiError),
        (status = 500 , description = "Request error", body = ApiError)
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[get("/api/info")]
async fn game_info(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    println!("{:?}", extract_value(&req, "key"));

    let lobby_id = match extract_header_string(&req, "lobby-id") {
        Ok(data) => data,
        Err(error) => return Ok(error),
    };

    let lobby =  match srv.send(game::LobbyExists { lobby_id: LobbyId::of(lobby_id.clone())}).await {
        Ok(data) => data,
        Err(_) => return Ok(ApiGameError::GameError(lobby_id).to_response()),
    };

    let user = match lobby {
        false => return Ok(ApiGameError::LobbyNotFound(lobby_id).to_response()),
        true => "Found something"
    };
    Ok(HttpResponse::from(HttpResponse::Ok().json(user)))
}
#[derive(Debug, Clone,Serialize, ToSchema)]
pub struct UserSessionWithAdmin{
    user_session:UserSession,
    is_admin:bool
}


#[utoipa::path(
    get,
    path = "/api/session",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token")
    ),
    responses(
        (status = 200, description = "User session retrieved successfully", body = UserSessionWithAdmin),
        (status = 404, description = "No User Session", body = ApiSessionError)
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
    

)]
#[get("/api/session")]
async fn api_session_request(req: HttpRequest ,db:web::Data<Arc<MongoServer>>, settings: web::Data<Arc<Settings>>) -> Result<HttpResponse, actix_web::Error> {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(ApiError::Session(ApiSessionError::NotFound).to_response()),
    };
    let is_admin = is_admin(&user_session, &db).await;
    let mut response = HttpResponse::from(HttpResponse::Ok().json(UserSessionWithAdmin{user_session:user_session.clone(), is_admin}));
    set_session_token_cookie(&mut response, &settings, &user_session);
    Ok(response)
}








pub async fn file_part_error(stream: Option<GridFsUploadStream>,api_error:ApiError) -> HttpResponse {
    if let Some(mut stream) = stream {
        match stream.close().await {
            Ok(data) => data,
            Err(error) => return ApiError::File(ApiFileError::FileError(error.to_string())).to_response(),
        };
    }
    return api_error.to_response();
}


pub fn session_error(settings: &web::Data<Arc<Settings>>, user_session:&UserSession, str:&str) -> HttpResponse {
    let mut response = HttpResponse::InternalServerError().json(json!({
        "Error": str
    }));
    set_session_token_cookie(&mut response, settings, &user_session);
    response
}




#[utoipa::path(
    get,
    path = "/api/session-data",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
    ),
    responses(
        // 200
        (status = 200, description = "Request or updated sessiondata", body = SessionRequest),

    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[get("/api/session-data")]
async fn session_data_request(req: HttpRequest, db: web::Data<Arc<MongoServer>>, settings: web::Data<Arc<Settings>>) -> Result<HttpResponse, actix_web::Error> {
    println!("SESSION DATA REQUEST");
    let user_session = get_session_with_token_update_or_create_new(&req, &db).await;
    let sessionrequest : SessionRequest = SessionRequest{
     user_session_id: user_session.user_session_id.clone(), 
     session_token: user_session.session_token.clone() 
    };
    let mut response = HttpResponse::from(HttpResponse::Ok().json(sessionrequest));
    set_session_token_cookie(&mut response,&settings, &user_session);
    Ok(response)
}






#[utoipa::path(
    get,
    path = "/api/authorization",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
    ),
    responses(
        // 200
        (status = 200, description = "If the user has a discord session", body = ApiResponse),

        // Session
        (status = 404, description = "No User Session", body = ApiError),
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[get("/api/authorization")]
async fn has_authorization(req: HttpRequest, db: web::Data<Arc<MongoServer>>, settings: web::Data<Arc<Settings>>) -> HttpResponse {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return ApiSessionError::NotFound.to_api_error().to_response(),
    };
    let admin = is_admin(&user_session, &db).await;
    let mut response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::new(admin)));
    set_session_token_cookie(&mut response, &settings, &user_session);
    response
}


#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DiscordSessionResponse{
    discord_user:Option<DiscordUser>,
}


#[utoipa::path(
    get,
    path = "/api/discord_session",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
    ),
    responses(
        // 200
        (status = 200, description = "If the user has a discord session", body = DiscordSessionResponse),

        // Session
        (status = 404, description = "No User Session", body = ApiError),
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[get("/api/discord_session")]
async fn discord_session(req: HttpRequest, db: web::Data<Arc<MongoServer>>, settings: web::Data<Arc<Settings>>) -> HttpResponse {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return ApiSessionError::NotFound.to_api_error().to_response(),
    };
    let discord_user = match user_session.clone().discord_auth{
        None => None,
        Some(data) => data.discord_user,
    };
    let mut response = HttpResponse::from(HttpResponse::Ok().json(discord_user));
    set_session_token_cookie(&mut response, &settings, &user_session);
    response
}







#[utoipa::path(
    post,
    path = "/api/create",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
    ),
    request_body(content = JeopardyBoard, description = "Upload JeopardyBoard and create a lobby", content_type = "application/json"),
    responses(
        // 200
        (status = 200, description = "JeopardyBoard uploaded successfully", body = LobbyCreateResponse),
        
        // Session
        (status = 404, description = "No User Session", body = ApiError),
        (status = 404, description = "No JeopardyBoard found", body = ApiError),
        (status = 401, description = "Not a admin Session", body = ApiError),
        (status = 403, description = "No Discord Data", body = ApiError),

        //GAME
        (status = 500, description = "Game error", body = ApiError),
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[post("/api/create")]
async fn create_game_lobby(req: HttpRequest,json: web::Json<Option<JeopardyBoard>>, srv: web::Data<Addr<game::GameServer>>, db: web::Data<Arc<MongoServer>>, settings: web::Data<Arc<Settings>>) -> HttpResponse {
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return ApiSessionError::NotFound.to_api_error().to_response(),
    };

    if !is_admin(&user_session, &db).await {
        return ApiSessionError::NotAdmin.to_api_error().to_response();
    }
    let discord_id = match user_session.discord_auth.clone() {
        Some(data) => data.discord_user.unwrap().discord_id,
        None => return ApiSessionError::NoDiscordData.to_api_error().to_response(),
    };

    let data = match  srv.send(CreateLobby { user_session_id: user_session.user_session_id.clone(), discord_id, jeopardy_board: json.into_inner() }).await {
        Ok(data) => data,
        Err(_) => return ApiGameError::GameError("No JeopardyBoard found".to_string()).to_response(),
    };

    let mut response = HttpResponse::from(HttpResponse::Ok().json(data));
    set_session_token_cookie(&mut response, &settings, &user_session);
    response
}




#[utoipa::path(
    get,
    path = "/api/join",
    params(
        ("user_session_id" = Option<String>, Query, description = "User session ID"),
        ("user_session_token" = Option<String>, Query, description = "User session token"),
        ("lobby-id" = String, Header, description = "Lobby id"),
    ),
    responses(
        // 200
        (status = 200, description = "Allowed to join the lobby", body = ApiResponse),
        
        // Session
        (status = 404, description = "No User Session", body = ApiError),
        (status = 404, description = "No JeopardyBoard found", body = ApiError),

        //GAME
        (status = 404, description = "No Lobby found", body = ApiError),
        (status = 500, description = "Game error", body = ApiError),
    ),
    security(
        ("cookie" = ["user_session_id", "user_session_token"])
    )
)]
#[get("/api/join")]
async fn join_game(req: HttpRequest, srv: web::Data<Addr<game::GameServer>>, db: web::Data<Arc<MongoServer>>, settings: web::Data<Arc<Settings>>) -> Result<HttpResponse, actix_web::Error> {
    println!("{:?}", extract_header_string(&req, "lobby-id"));
    let lobby_id = match get_lobby_id_from_header(&req){
        Some(data) => data,
        None => return Ok(ApiGameError::LobbyInvalid("No Lobby ID found".to_string()).to_response()),
    };
    let user_session = match get_session(&req, &db).await {
        Some(data) => data,
        None => return Ok(ApiSessionError::NotFound.to_api_error().to_response()),
    };
    let opt_addr = match srv.send(game::LobbyAddrRequest{lobby_id:lobby_id.clone()}).await {
        Ok(data) => data,
        Err(_) => return Ok(ApiGameError::LobbyNotFound(lobby_id.clone().id).to_response()),
    };
    let lobby_adrr = match opt_addr {
        Some(data) => data,
        None => return Ok(ApiGameError::LobbyNotFound(lobby_id.clone().id).to_response()),
    };
    let can_join = match lobby_adrr.send(CanJoinLobby { user_session_id: user_session.user_session_id.clone()}).await {
        Ok(data) => data,
        Err(_) => return Ok(ApiGameError::GameError("No Lobby found!".to_string()).to_response()),
    };
    let mut response = HttpResponse::from(HttpResponse::Ok().json(ApiResponse::new(can_join)));
    set_session_token_cookie(&mut response, &settings, &user_session);
    Ok(response)
}




#[utoipa::path(
    get,
    path = "/api/board",
    responses(
        // 200
        (status = 200, description = "Get default JeopardyBoard", body = JeopardyBoard),
    ),
)]
#[get("/api/board")]
async fn board() -> HttpResponse {
    let response = HttpResponse::from(HttpResponse::Ok().json(JeopardyBoard::default(JeopardyMode::NORMAL)));
    response
}

