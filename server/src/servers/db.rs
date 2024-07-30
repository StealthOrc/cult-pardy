use mongodb::bson::doc;
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use mongodb::sync::{Client, Collection};
use strum::{Display, EnumIter};
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use crate::servers::db::DBDatabase::CultPardy;
use crate::servers::db::UserCollection::UserSessions;
use crate::servers::game::UserSession;

use super::game::DiscordData;


#[derive(Copy,Clone,EnumIter,Display, Debug, Default)]
pub enum DBDatabase{
    CultPardy(UserCollection),
    #[default]
    None,
}



#[derive(Copy,Clone,EnumIter,Display,Debug, Default)]
pub enum UserCollection{
    #[default]
    UserSessions,
}


#[derive(Clone, Debug)]
pub struct MongoServer{
    pub mongo_client: Client,
}


impl MongoServer{

    pub fn new() -> Self{
        let url = std::env::var("MONGODB_URI").expect("Cant get MONGODB_URI");
        let mut client_options = ClientOptions::parse(url).expect("Can´t connect to Mongodb");
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);
        let mongo_client = Client::with_options(client_options).expect("failed to connect");
        mongo_client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .expect("Cant ping");
        println!("Pinged your deployment. You successfully connected to MongoDB!");
        MongoServer{
            mongo_client,
        }
    }


    pub fn collection<T>(&self, dbdatabase: DBDatabase) -> Collection<T> {
        match dbdatabase {
            DBDatabase::CultPardy(collection) => self.mongo_client.database(&dbdatabase.to_string()).collection::<T>(&collection.to_string()),
            _ => {todo!()}
        }

    }

    pub fn find_user_session(&self, user_session_id: &UserSessionId) -> Option<UserSession> {
        let result = self.collection::<UserSession>(CultPardy(UserSessions)).find_one(doc! {"user_session_id.id": &user_session_id.id}, None);
        match result {
            Err(e) => {
                return None;
            }
            Ok(data) => data,
        }
    }


    pub fn find_user_session_with_discord(&self, discord_data: &DiscordData) -> Option<UserSession> {
        let optianal_discord_user = discord_data.discord_user.clone();
        if optianal_discord_user.is_none() {
            return None;
        }
        let id : String = optianal_discord_user.unwrap().discord_id.id.clone();
        let test = doc! {"discord_auth.discord_user.discord_id": &id};
        let result = self.collection::<UserSession>(CultPardy(UserSessions)).find_one(test, None);
        match result {
            Err(e) => {
                return None;
            }
            Ok(data) => data,
        }
    }



    pub fn has_user_session(&self, user_session_id: &UserSessionId) -> bool {
         match self.find_user_session(&user_session_id){
            None => false,
            Some(_) => true,
        }
    }



}