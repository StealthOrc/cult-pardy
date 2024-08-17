
use actix::{ActorStreamExt, Addr};
use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use cult_common::wasm_lib::ids::lobby::LobbyId;
use crate::apis::api::session_error;
use crate::apis::data::{get_session_with_token_update_or_create_new, remove_cookie, set_cookie, set_session_token_cookie};
use crate::authentication::discord::{is_admin, to_main_page};
use crate::services::authentication::{AuthenticationServer, CheckAdminAccessToken};
use crate::services::db::MongoServer;
use crate::services::lobby::CanJoinLobby;
use crate::services::{game};
use crate::services::game::{GameServer};
use crate::settings;


pub fn index_response(req: &HttpRequest) -> HttpResponse{
    let mut cexe = match env::current_exe() {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    cexe.pop();
    cexe.push("www");
    cexe.push("index.html");

    let final_path = match cexe.into_os_string().into_string() {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let file = match NamedFile::open(final_path) {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    file.into_response(&req)
}




#[get("/game/{lobby_id}")]
async fn find_game(
    req: HttpRequest,
    lobby_id: web::Path<String>,
    srv: web::Data<Addr<GameServer>>,
    db : web::Data<Arc<MongoServer>>,
    settings: web::Data<Arc<settings::Settings>>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_session = get_session_with_token_update_or_create_new(&req, &db).await;

    let id = lobby_id.to_string(); 
    let lobby_id = LobbyId::of(id);




    let lobby_addr = match srv.send(game::LobbyAddrRequest{lobby_id:lobby_id.clone()}).await.expect("No Lobby found!") {
        Some(data) => data,
        None => return Ok(session_error(&req, &user_session, "No Lobby Found")),
    };

    let can_join = lobby_addr.send(CanJoinLobby { user_session_id: user_session.user_session_id.clone()}).await.expect("No Lobby found!");

   
    println!("HasLobby?{}", can_join);
    if !can_join {
        return to_main_page(&user_session, &settings)
    }
    let mut response = index_response(&req);
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}




#[get("/grant/{grand_id}")]
async fn grant_admin_access(
    req: HttpRequest,
    grand_id: web::Path<usize>,
    auth: web::Data<Addr<AuthenticationServer>>,
    db : web::Data<Arc<MongoServer>>,
    settings: web::Data<Arc<settings::Settings>>,
) -> Result<HttpResponse, actix_web::Error> {
    let url = settings.backend_settings.get_host();
    let user_session = get_session_with_token_update_or_create_new(&req, &db).await;
    let mut response = HttpResponse::Found()
        .append_header(("Location", format!("{}/discord?type=grant", url)))
        .finish();

    if is_admin(&user_session,&db).await {
        return to_main_page(&user_session,&settings);
    }

    match auth.send(CheckAdminAccessToken{ token: grand_id.clone()}).await {
        Ok(valid) => {
            if !valid {
                return to_main_page(&user_session, &settings)
            }
        }
        Err(_) =>  return to_main_page(&user_session, &settings)
    }

    if let Some(discord_data) = user_session.clone().discord_auth {
        if discord_data.discord_user.is_some() {
            response = HttpResponse::Found().append_header(("Location", format!("{}/grant", url))).finish();
        }
    }

    set_session_token_cookie(&mut response, &user_session);
    set_cookie(&mut response,"token", &grand_id.to_string());
    Ok(response)
}

#[get("/")]
async fn index(
    req: HttpRequest,
    db : web::Data<Arc<MongoServer>>,
) -> actix_web::Result<HttpResponse> {
    let user_session = get_session_with_token_update_or_create_new(&req, &db).await;
    let mut response = index_response(&req);
    remove_cookie(&mut response, &req, "token");
    set_session_token_cookie(&mut response, &user_session);
    Ok(response)
}

#[get("/assets/{filename:.*}")]
async fn assets(
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let path: PathBuf = req
        .match_info()
        .query("filename")
        .parse()
        .expect("assets(): could not parse filename");
    let mut cexe = env::current_exe().unwrap();
    cexe.pop();
    cexe.push("www");
    cexe.push(path);
    let final_path = cexe.into_os_string().into_string().unwrap();
    let named_file = NamedFile::open(final_path).expect("File not found");
    Ok(named_file.into_response(&req))
}

