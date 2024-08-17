use std::rc::Rc;
use std::sync::Arc;

use actix::{Actor, Addr};





use crate::authentication::discord::{GrantDiscordAuth, LoginDiscordAuth};
use crate::services::authentication::AuthenticationServer;
use crate::services::db::MongoServer;
use crate::services::game::GameServer;
use crate::settings::Settings;

pub(crate) mod game;
pub(crate) mod input;
pub(crate) mod authentication;
pub(crate) mod db;
pub(crate) mod lobby;


#[derive(Clone, Debug)]
pub(crate) struct StartingServices{
    pub authentication_server:Addr<AuthenticationServer>,
    pub grant_client : Arc<GrantDiscordAuth>,
    pub login_client :Arc<LoginDiscordAuth>,
    pub mongo_server : Arc<MongoServer>,
}

#[derive(Clone, Debug)]
pub(crate) struct Services {
    pub authentication_server:Addr<AuthenticationServer>,
    pub game_server: Addr<GameServer>,
    pub grant_client : Arc<GrantDiscordAuth>,
    pub login_client :Arc<LoginDiscordAuth>,
    pub mongo_server : Arc<MongoServer>,
}


impl Services {
    pub async fn init(settings:&Arc<Settings>) -> Self {

        let mongo_server = Arc::new(MongoServer::new(settings).await);
        let login_client = Arc::new(LoginDiscordAuth::init(settings));
        
        let auth_server = AuthenticationServer::new(mongo_server.clone()).start();


        let discord_auth =  Arc::new(GrantDiscordAuth::init(settings));

        let services =  Arc::new(StartingServices {
            authentication_server: auth_server.clone(),
            grant_client: discord_auth.clone(),
            login_client: login_client.clone(),
            mongo_server: mongo_server.clone()  ,
        });
        Services {
            authentication_server: auth_server,
            game_server: GameServer::new(services).start(),
            grant_client: discord_auth,
            login_client,
            mongo_server,
        }
    }

}

/*#[derive(Serialize, Deserialize, Debug)]
pub enum SessionMessageResult {
    Void,
    U64(u64),
}


#[derive(Serialize, Deserialize, Debug)]
pub enum GetSessionMessageType {
    GetPing,
}
    
#[derive(Message)]
#[rtype(result = "SessionMessageResult")]
pub enum SessionMessageType {
    Send(SendSessionMessageType),
    //Get(GetSessionMessageType),
}
*/
