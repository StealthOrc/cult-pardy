use std::collections::HashSet;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::Duration;
use actix::{Actor, Addr, Context, Handler, Message, MessageResult};
use futures::AsyncWriteExt;
use rand::{random, Rng};
use serde::{Deserialize, Serialize};
use tokio::fs::read_to_string;
use crate::servers::game::{Disconnect, GameServer};

#[derive(Serialize,Deserialize, Debug, Eq, PartialEq,Hash)]
struct Admin{
    discord_id:String,
    name:String
}



#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct AdminAccessTokenResponse {
    pub token: usize,
    pub lifetime: Duration,
}





pub struct NewAdminAccessToken;

impl Message for NewAdminAccessToken {
    type Result = AdminAccessTokenResponse;
}

pub struct CheckAdminAccessToken{
    pub token: usize,

}
impl actix::Message for CheckAdminAccessToken {
    type Result = bool;
}


impl AdminAccessTokenResponse {
    fn random() -> Self {
        AdminAccessTokenResponse {
            token: random::<usize>(),
            lifetime: Duration::from_secs(24*60*60),
        }
    }

}



fn get_file_reader(file_path: &str) -> BufReader<File> {
    let path = Path::new(file_path);
    let is_file = match path.exists() {
        true => File::open(path),
        false => File::create(file_path).and_then(|mut file| {
            file.write_all(b"[]").expect("Can´t write on file");
            file.flush().expect("Failed to flush file");
            File::open(file_path)
        }),
    };

    let file = is_file.expect("Failed to open file");
    let buf_reader = BufReader::new(file);
    buf_reader
}


fn read_structs_from_file(file_path: &str) -> Result<Vec<Admin>, ()> {
    let reader = get_file_reader(file_path);
    let struct_list: Vec<Admin> = serde_json::from_reader(reader).unwrap_or_else(|error| {
        eprintln!("Error deserializing JSON: {}", error);
        Vec::new()
    });
    Ok(struct_list)
}

fn append_structs_to_file(file_path: &str, structs_to_append: Vec<Admin>) -> Result<(), ()> {
    let mut reader = get_file_reader(file_path);
    let mut struct_list: Vec<Admin> = serde_json::from_reader(&mut reader).unwrap_or_else(|_| Vec::new());
    struct_list.extend(structs_to_append);
    let file = File::create(file_path).expect("Somethings wrong");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &struct_list).expect("Somethings wrong");
    Ok(())
}


#[derive(Debug)]
pub struct AuthenticationServer {
    admin_token: HashSet<AdminAccessTokenResponse>,
    ////Maybe only IDs? DISCORD_ID
    admin: HashSet<Admin>
}



impl AuthenticationServer {
    pub fn new() -> Self {
        let admins = match read_structs_from_file("Admin.json") {
            Ok(admins) => HashSet::from_iter(admins),
            Err(_) => HashSet::new(),
        };
        let mut token = HashSet::new();
        token.insert(AdminAccessTokenResponse {
            token: 123,
            lifetime: Duration::from_secs(24*60*60),
        });
        AuthenticationServer{
            admin_token: token,
            admin: admins,
        }
    }
}


impl Actor for AuthenticationServer {
    type Context = Context<Self>;

    fn start(self) -> Addr<Self> where Self: Actor<Context = Context<Self>> {
        Context::new().run(self)
    }
}

impl Handler<NewAdminAccessToken> for AuthenticationServer {
    type Result = MessageResult<NewAdminAccessToken>;

    fn handle(&mut self, _msg: NewAdminAccessToken, _ctx: &mut Self::Context) -> Self::Result {
        let mut token= AdminAccessTokenResponse::random();
        while self.admin_token.contains(&token) {
            token = AdminAccessTokenResponse::random()
        }
        self.admin_token.insert(token);
        return MessageResult(token)
    }
}

impl Handler<CheckAdminAccessToken> for AuthenticationServer {
    type Result = bool;

    fn handle(&mut self, _msg: CheckAdminAccessToken, _ctx: &mut Self::Context) -> Self::Result {
        return self.admin_token.iter().any(|token | token.token.eq(&_msg.token))
    }
}