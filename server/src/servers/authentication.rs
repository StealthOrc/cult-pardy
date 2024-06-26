use std::collections::{HashMap, HashSet};
use std::fs::{File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Duration;
use actix::{Actor, Addr, Context, Handler, Message, MessageResult};
use futures::AsyncWriteExt;
use rand::{random};
use serde::{Deserialize, Serialize};
use cult_common::{DiscordID, UserSessionId};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
struct Admin{
    discord_id:DiscordID,
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


pub struct AddAdminAccess {
    pub discord_id: DiscordID,

}
impl actix::Message for AddAdminAccess {
    type Result = bool;
}


pub struct CheckAdminAccessToken{
    pub token: usize,

}
impl actix::Message for CheckAdminAccessToken {
    type Result = bool;
}


pub struct RedeemAdminAccessToken{
    pub token: usize,
    pub discord_id: DiscordID,
}

impl RedeemAdminAccessToken{
    pub fn new(token: usize, discord_id: DiscordID) -> Self{
        RedeemAdminAccessToken{
            token,
            discord_id,
        }
    }
}


impl actix::Message for RedeemAdminAccessToken {
    type Result = bool;
}


pub struct CheckAdminAccess{
    pub discord_id: DiscordID,

}
impl actix::Message for CheckAdminAccess {
    type Result = bool;
}

pub struct GetAdminAccess {}
impl actix::Message for GetAdminAccess {
    type Result = Vec<DiscordID>;
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

fn append_structs_to_file(file_path: &str, structs_to_append: Admin) -> Result<(), ()> {
    let mut reader = get_file_reader(file_path);
    let mut struct_list: Vec<Admin> = serde_json::from_reader(&mut reader).unwrap_or_else(|_| Vec::new());
    struct_list.push(structs_to_append);
    let file = File::create(file_path).expect("Somethings wrong");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &struct_list).expect("Somethings wrong");
    Ok(())
}


#[derive(Debug, Clone)]
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

    fn add_admin_access(&mut self, admin: Admin) -> bool{
        if self.admin.contains(&admin) {
            return true
        } else {
            append_structs_to_file(&"Admin.json", admin.clone()).expect("Somethings wrong");
            self.admin.insert(admin)
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

impl Handler<AddAdminAccess> for AuthenticationServer {
    type Result = bool;

    fn handle(&mut self, msg: AddAdminAccess, _ctx: &mut Self::Context) -> Self::Result {
        let admin = Admin{ discord_id:msg.discord_id };
        self.add_admin_access(admin)
    }
}



impl Handler<RedeemAdminAccessToken> for AuthenticationServer {
    type Result = bool;

    fn handle(&mut self, msg: RedeemAdminAccessToken, _ctx: &mut Self::Context) -> Self::Result {
        let original_len = self.admin_token.len().clone();
        self.admin_token.retain(|token| token.token != msg.token);
        let token_removed = self.admin_token.len() < original_len;

        if token_removed {
            let admin = Admin { discord_id: msg.discord_id };
            self.add_admin_access(admin);
            true
        } else {
            false
        }
    }
}

impl Handler<CheckAdminAccess> for AuthenticationServer {
    type Result = bool;

    fn handle(&mut self, msg: CheckAdminAccess, _ctx: &mut Self::Context) -> Self::Result {
        let admin = Admin{ discord_id:msg.discord_id };
        self.admin.contains(&admin)
    }
}

impl Handler<CheckAdminAccessToken> for AuthenticationServer {
    type Result = bool;

    fn handle(&mut self, _msg: CheckAdminAccessToken, _ctx: &mut Self::Context) -> Self::Result {
        return self.admin_token.iter().any(|token | token.token.eq(&_msg.token))
    }
}

impl Handler<GetAdminAccess> for AuthenticationServer {
    type Result = Vec<DiscordID>;

    fn handle(&mut self, _msg: GetAdminAccess, _ctx: &mut Self::Context) -> Self::Result {
        self.admin.iter().map(|admin| admin.discord_id.clone()).collect::<Vec<DiscordID>>()
    }
}