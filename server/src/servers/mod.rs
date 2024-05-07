use actix::{Actor, Addr};
use crate::authentication::discord::{GrantDiscordAuth, LoginDiscordAuth};
use crate::servers::authentication::AuthenticationServer;
use crate::servers::game::GameServer;

pub(crate) mod game;
pub(crate) mod input;
pub(crate) mod authentication;
pub(crate) mod gamestructure;


#[derive(Clone)]
pub(crate) struct Services{
    pub authentication_server:Addr<AuthenticationServer>,
    pub game_server:Addr<GameServer>,
    pub grant_client : GrantDiscordAuth,
    pub login_client :LoginDiscordAuth,
}


impl Services {
    pub fn init() -> Self {
        let login_client = LoginDiscordAuth::init();
        let auth_server = AuthenticationServer::new().start();
        Services{
            grant_client : GrantDiscordAuth::init(),
            login_client: login_client.clone(),
            authentication_server:auth_server.clone(),
            game_server : GameServer::new(login_client,auth_server).start(),
        }
    }

}