use std::env;
use std::path::PathBuf;
use actix::Addr;
use actix_files::NamedFile;
use actix_web::{get, HttpRequest, HttpResponse, web};
use actix_web::cookie::Cookie;
use serde_json::json;
use cult_common::UserSessionRequest;
use crate::api::{get_session, session, set_session_cookies};
use crate::data::extract_value;
use crate::{api, server};

#[get("/game/{lobby_id}")]
async fn game(req: HttpRequest, lobby_id: web::Path<(Option<String>,)>, srv: web::Data<Addr<server::GameServer>>) -> Result<HttpResponse, actix_web::Error> {
    //TODO HACKY!!
    println!("{:?}", req.cookies());
    let user_session_id = get_session(&req, &srv).await;

    let lobby_id = match lobby_id.into_inner() {
        (None,) => return Ok(HttpResponse::from(HttpResponse::InternalServerError())),
        (Some(id),) => id,
    };

    let lobby = srv.send(server::Lobby{lobby_id: lobby_id.clone()}).await.expect("No Lobby found!");
    let error = json!(
        {
            "Error": "Lobby not found",
            "Lobby": lobby_id
        }
    );
    let user =match lobby {
        None => return Ok(HttpResponse::from(HttpResponse::InternalServerError().json(error))),
        Some(users) => users
    };
    let users = json!(
        {
            "Lobby": lobby_id,
            "User_session_id": user_session_id,
            "Users": user.len(),
            "Users": user
        }
    );
    let mut response = HttpResponse::from(HttpResponse::Ok().json(users));
    set_session_cookies(&mut response, "user_session_id", &user_session_id.to_string());
    Ok(response)
}




#[get("/")]
async fn index(req: HttpRequest, srv: web::Data<Addr<server::GameServer>>) -> actix_web::Result<HttpResponse> {
    let mut cexe = env::current_exe().unwrap();
    cexe.pop();
    cexe.push("www");
    cexe.push("index.html");
    let final_path = cexe.into_os_string().into_string().unwrap();
    println!("path:{}", final_path);
    let named_file = NamedFile::open(final_path).expect("File not found");

    //TODO HACKY!!

    let user_session_id = get_session(&req, &srv).await;


    let mut response = named_file.into_response(&req);
    set_session_cookies(&mut response, "user_session_id", &user_session_id.to_string());
    Ok(response)
}




#[get("/file/{filename:.*}")]
async fn file(req: HttpRequest, srv: web::Data<Addr<server::GameServer>>) -> actix_web::Result<HttpResponse> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let mut cexe = env::current_exe().unwrap();
    cexe.pop();
    cexe.push("www");
    cexe.push(path);
    let final_path = cexe.into_os_string().into_string().unwrap();
    let named_file = NamedFile::open(final_path).expect("File not found");

    let user_session_id = get_session(&req, &srv).await;

    let mut response = named_file.into_response(&req);
    set_session_cookies(&mut response, "user_session_id", &user_session_id.to_string());
    Ok(response)
}
