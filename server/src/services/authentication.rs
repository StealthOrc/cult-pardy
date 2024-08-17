
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use actix::{Actor, Addr, Context, Handler, Message, MessageResult, ResponseActFuture, WrapFuture};
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use mongodb::bson::doc;
use rand::random;
use serde::{Deserialize, Serialize};
use cult_common::wasm_lib::ids::discord::DiscordID;
use strum::{Display, EnumIter};

use crate::data::SessionRequest;

use super::{db::MongoServer, game::DiscordData};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct Admin{
    pub discord_id:DiscordID,
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

pub struct GetAdminAccess {

}
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

#[derive(Message)]
#[rtype(result = "DiscordAccountStatus")]
pub struct AddDiscordAccount {
    pub user_session_id: UserSessionId,
    pub discord_data: DiscordData,
}

#[derive(Debug, Clone,PartialEq, EnumIter, Display)]
pub enum DiscordAccountStatus {
    Added,
    Updated(SessionRequest),
    NotAdded,
}

impl DiscordAccountStatus {
    pub fn to_help(self) -> bool {
    match self {
        DiscordAccountStatus::Added => true,
        DiscordAccountStatus::Updated(_) => true,
        DiscordAccountStatus::NotAdded => false,
        }
    }
}





#[derive(Debug, Clone)]
pub struct AuthenticationServer {
    admin_token: HashSet<AdminAccessTokenResponse>,
    ////Maybe only IDs? DISCORD_ID
    // Mongodb
    mongo_server: Arc<MongoServer>,
}



impl AuthenticationServer {
    pub fn new(mongo_server:Arc<MongoServer>) -> Self {
        let mut token = HashSet::new();
        token.insert(AdminAccessTokenResponse {
            token: 123,
            lifetime: Duration::from_secs(24*60*60),
        });
        AuthenticationServer{
            admin_token: token,
            mongo_server,
        }
    }

    async fn add_admin_access(&mut self, admin: Admin) -> bool {
        if self.mongo_server.find_admin(&admin.discord_id).await.is_none(){
            self.mongo_server.add_admin(admin.clone()).await
        } else {
            false
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
    type Result = ResponseActFuture<Self, bool>;

    fn handle(&mut self, msg: AddAdminAccess, _ctx: &mut Self::Context) -> Self::Result {
        let db = self.mongo_server.clone();
        Box::pin(
            async move {
                if db.find_admin(&msg.discord_id).await.is_none(){
                    db.add_admin(Admin{
                        discord_id: msg.discord_id.clone(),
                    }).await
                } else {
                    false
                }
            }
            .into_actor(self)
        )
    }
}



impl Handler<RedeemAdminAccessToken> for AuthenticationServer {
    type Result = ResponseActFuture<Self, bool>;

    fn handle(&mut self, msg: RedeemAdminAccessToken, _ctx: &mut Self::Context) -> Self::Result {
        let original_len = self.admin_token.len().clone();
        self.admin_token.retain(|token| token.token != msg.token);
        let token_removed = self.admin_token.len() < original_len;
        let admin = Admin{
            discord_id: msg.discord_id.clone(),
        };

        let db: Arc<MongoServer> = self.mongo_server.clone();
        Box::pin(
            async move {
                if token_removed {
                    if db.find_admin(&admin.discord_id).await.is_none(){
                        db.add_admin(admin.clone()).await
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            .into_actor(self)
        )
    }
    
}

impl Handler<CheckAdminAccess> for AuthenticationServer {
    type Result = ResponseActFuture<Self, bool>;

    fn handle(&mut self, msg: CheckAdminAccess, _ctx: &mut Self::Context) -> Self::Result {
        let db = self.mongo_server.clone();
        Box::pin(
            async move {
                db.find_admin(&msg.discord_id).await.is_some()
            }
            .into_actor(self)
        )
    }
}

impl Handler<CheckAdminAccessToken> for AuthenticationServer {
    type Result = bool;

    fn handle(&mut self, _msg: CheckAdminAccessToken, _ctx: &mut Self::Context) -> Self::Result {
        return self.admin_token.iter().any(|token | token.token.eq(&_msg.token))
    }
}

impl Handler<GetAdminAccess> for AuthenticationServer {
    type Result = ResponseActFuture<Self, Vec<DiscordID>>;

    fn handle(&mut self, _msg: GetAdminAccess, _ctx: &mut Self::Context) -> Self::Result {
        let db = self.mongo_server.clone();
        Box::pin(
            async move {
                let test = db.get_admins().await.iter().map(|admin| admin.discord_id.clone()).collect::<Vec<DiscordID>>();
                test
            }
            .into_actor(self)
        )
    }
}


impl Handler<AddDiscordAccount> for AuthenticationServer {
    type Result = ResponseActFuture<Self, DiscordAccountStatus>;

    fn handle(&mut self, msg: AddDiscordAccount, _ctx: &mut Context<Self>) -> Self::Result {
        let db = self.mongo_server.clone();
        Box::pin(
            async move {
                if let Some(session) = db.find_user_session_with_discord(&msg.discord_data).await{
                    return DiscordAccountStatus::Updated(SessionRequest{
                        user_session_id: session.user_session_id,
                        session_token: session.session_token,
                    });
                }
                if !db.update_discord_data(&msg.user_session_id, &msg.discord_data).await {
                    return DiscordAccountStatus::NotAdded
                }
                DiscordAccountStatus::Added
            }
            .into_actor(self)
        )
        
    }
}






