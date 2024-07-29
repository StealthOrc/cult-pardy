use actix::{Actor, Addr};





use crate::authentication::discord::{GrantDiscordAuth, LoginDiscordAuth};
use crate::servers::authentication::AuthenticationServer;
use crate::servers::db::MongoServer;
use crate::servers::game::GameServer;

pub(crate) mod game;
pub(crate) mod input;
pub(crate) mod authentication;
pub(crate) mod gamestructure;
pub(crate) mod db;


#[derive(Clone, Debug)]
pub(crate) struct StartingServices{
    pub authentication_server:Addr<AuthenticationServer>,
    pub grant_client : GrantDiscordAuth,
    pub login_client :LoginDiscordAuth,
    pub mongo_server : MongoServer,
}

#[derive(Clone, Debug)]
pub(crate) struct Services {
    pub authentication_server:Addr<AuthenticationServer>,
    pub game_server: Addr<GameServer>,
    pub grant_client : GrantDiscordAuth,
    pub login_client :LoginDiscordAuth,
    pub mongo_server : MongoServer,
}


impl Services {
    pub async fn init() -> Self {

        let mongo_server = MongoServer::new();

        let login_client = LoginDiscordAuth::init();
        let auth_server = AuthenticationServer::new().start();
        let discord_auth = GrantDiscordAuth::init();

        let services = StartingServices {
            authentication_server: auth_server.clone(),
            grant_client: discord_auth.clone(),
            login_client: login_client.clone(),
            mongo_server: mongo_server.clone(),
        };
        Services {
            authentication_server: auth_server,
            game_server: GameServer::new(services.clone()).start(),
            grant_client: discord_auth,
            login_client,
            mongo_server,
        }
    }

}