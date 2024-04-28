
use actix::Addr;
use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse};
use serde_json::json;
use std::env;
use std::path::PathBuf;
use crate::apis::api::{get_session, set_session_cookies};
use crate::servers::authentication::AuthenticationServer;
use crate::servers::{authentication, game};
use crate::servers::game::GameServer;

#[get("/game/{lobby_id}")]
async fn find_game(
    req: HttpRequest,
    lobby_id: web::Path<(Option<String>,)>,
    srv: web::Data<Addr<GameServer>>,
) -> Result<HttpResponse, actix_web::Error> {
    //TODO HACKY!!
    println!("{:?}", req.cookies());
    let user_session_id = get_session(&req, &srv).await;

    let lobby_id = match lobby_id.into_inner() {
        (None,) => return Ok(HttpResponse::from(HttpResponse::InternalServerError())),
        (Some(id),) => id,
    };

    let lobby = srv
        .send(game::Lobby {
            lobby_id: lobby_id.clone(),
        })
        .await
        .expect("No Lobby found!");
    let error = json!(
        {
            "Error": "Lobby not found",
            "Lobby": lobby_id
        }
    );
    let user = match lobby {
        None => {
            return Ok(HttpResponse::from(
                HttpResponse::InternalServerError().json(error),
            ))
        }
        Some(users) => users,
    };
    let _users = json!(
        {
            "Lobby": lobby_id,
            "User-session-id": user_session_id,
            "Users": user.len(),
            "Users": user
        }
    );
    let mut cexe = env::current_exe().unwrap();
    cexe.pop();
    cexe.push("www");
    cexe.push("index.html");
    let final_path = cexe.into_os_string().into_string().unwrap();
    println!("{}",final_path);
    let named_file = NamedFile::open(final_path).expect("{:?}File not found");
    let mut response = named_file.into_response(&req);
    set_session_cookies(
        &mut response,
        "user-session-id",
        &user_session_id.to_string(),
    );
    Ok(response)
}


#[get("/grant/{grand_id}")]
async fn grant_admin_access(
    req: HttpRequest,
    grand_id: web::Path<(usize)>,
    srv: web::Data<Addr<GameServer>>,
    auth: web::Data<Addr<AuthenticationServer>>,
) -> Result<HttpResponse, actix_web::Error> {
    print!("?");
    //let user_session_id = get_session(&req, &srv).await;


    let lobby = auth.send(authentication::CheckAdminAccessToken { token: grand_id.clone(), })
        .await
        .expect("No Lobby found!");
    print!("{lobby}");

    Ok(HttpResponse::Ok().json(lobby))
}

#[get("/")]
async fn index(
    req: HttpRequest,
    srv: web::Data<Addr<GameServer>>,
) -> actix_web::Result<HttpResponse> {
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
    set_session_cookies(
        &mut response,
        "user-session-id",
        &user_session_id.to_string(),
    );
    Ok(response)
}

#[get("/assets/{filename:.*}")]
async fn assets(
    req: HttpRequest,
    srv: web::Data<Addr<GameServer>>,
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
    println!("cexe: {:?}", cexe);
    let final_path = cexe.into_os_string().into_string().unwrap();
    let named_file = NamedFile::open(final_path).expect("File not found");

    let user_session_id = get_session(&req, &srv).await;

    let mut response = named_file.into_response(&req);
    set_session_cookies(
        &mut response,
        "user-session-id",
        &user_session_id.to_string(),
    );
    Ok(response)
}


#[get("/*")]
async fn test(
    req: HttpRequest,
    srv: web::Data<Addr<GameServer>>,
) -> actix_web::Result<HttpResponse> {
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
    set_session_cookies(
        &mut response,
        "user-session-id",
        &user_session_id.to_string(),
    );
    Ok(response)
}

