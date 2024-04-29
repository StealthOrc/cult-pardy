use actix::{Actor, Addr};
use serde::Serialize;
use crate::servers::authentication::AuthenticationServer;
use crate::servers::game::GameServer;

pub(crate) mod game;
pub(crate) mod input;
pub(crate) mod authentication;


pub(crate) struct Services {
    pub authentication_server:Addr<AuthenticationServer>,
    pub game_server:Addr<GameServer>
}


impl Services {
    pub fn init() -> Self {
        Services{
            authentication_server:AuthenticationServer::new().start(),
            game_server :GameServer::new().start(),
        }
    }

}