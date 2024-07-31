use std::collections::HashSet;
use std::fs::{File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Duration;
use actix::{Actor, Addr, Context, Handler, Message, MessageResult};
use futures::AsyncWriteExt;
use rand::{random};
use serde::{Deserialize, Serialize};
use cult_common::wasm_lib::ids::discord::DiscordID;

use super::db::MongoServer;
use super::StartingServices;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct Admin{
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


pub struct CheckAdminAccess {
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





#[derive(Debug, Clone)]
pub struct AuthenticationServer {
    admin_token: HashSet<AdminAccessTokenResponse>,
    ////Maybe only IDs? DISCORD_ID
    // Mongodb
    mongo_server: MongoServer,
    admin: HashSet<Admin>
}



impl AuthenticationServer {
    pub fn new(mongo_server:MongoServer ) -> Self {
        let admins = mongo_server.get_admins();
        let mut token = HashSet::new();
        token.insert(AdminAccessTokenResponse {
            token: 123,
            lifetime: Duration::from_secs(24*60*60),
        });
        AuthenticationServer{
            admin_token: token,
            mongo_server: mongo_server,
            admin: admins,
        }
    }

    fn add_admin_access(&mut self, admin: Admin) -> bool{
        if self.admin.contains(&admin) {
            return true
        } else {
            self.mongo_server.add_admin(admin.clone());
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