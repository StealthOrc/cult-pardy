use std::collections::HashSet;

use cult_common::dto::FileChunk;
use cult_common::wasm_lib::hashs::filechunk::FileChunkHash;
use cult_common::wasm_lib::hashs::validate::ValidateHash;
use cult_common::wasm_lib::ids::discord::DiscordID;
use cult_common::wasm_lib::{CFile, FileData};
use mongodb::bson::{doc, to_bson};
use mongodb::options::{ClientOptions, ServerApi, ServerApiVersion};
use mongodb::sync::{Client, Collection};
use ritelinked::LinkedHashSet;
use strum::{Display, EnumIter};
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use crate::servers::db::DBDatabase::CultPardy;
use crate::servers::db::UserCollection::UserSessions;
use crate::servers::game::UserSession;

use super::authentication::{self, Admin};
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
    Admins,
    FileData,
    FileChunks
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
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }

    pub fn find_admin(&self, discord_id: &DiscordID) -> Option<Admin> {
        let result = self.collection::<authentication::Admin>(CultPardy(UserCollection::Admins)).find_one(doc! {"discord_id.id": &discord_id.id}, None);
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }

    pub fn is_admin(&self, discord_id: &DiscordID) -> bool {
        match self.find_admin(discord_id) {
            None => false,
            Some(_) => true,
        }
    }


    pub fn add_admin(&self, admin: Admin) -> bool {
        let result = self.collection::<authentication::Admin>(CultPardy(UserCollection::Admins)).insert_one(admin, None);
        match result {
            Err(_) => {
                return false;
            }
            Ok(_) => {
                return true;
            }
        }
    }

    pub fn get_admins(&self) -> HashSet<Admin> {
        let result = self.collection::<authentication::Admin>(CultPardy(UserCollection::Admins)).find(None, None);
        match result {
            Err(_) => {
                return HashSet::new();
            }
            Ok(data) => {
                let mut set = HashSet::new();
                for doc in data {
                    set.insert(doc.unwrap());
                }
                return set;
            }
        }
    }

    /*#[derive(Tsify, Debug,Serialize,Deserialize ,Clone ,Hash,Eq, PartialEq, Default)]
    pub struct FileData {
        chunks: Vec<FileChunkHash>,
        pub file_name: String,
        pub total_chunks: usize,
        pub file_type: String,
        pub filedata: FileDataHash,
        pub validate_hash: ValidateHash,
        pub upload_data: DateTime<Local>,
        pub uploader: UserSessionId,
}
 */

    pub async fn add_file_data(&self, file: FileData) -> bool {
        let result = self.collection::<FileData>(CultPardy(UserCollection::FileData)).insert_one(file, None);
        match result {
            Err(_) => {
                return false;
            }
            Ok(_) => {
                return true;
            }
        }
    }


    pub async fn add_file_chunk(&self, file_chunk: &FileChunk) -> bool {
        let result = self.collection::<FileChunk>(CultPardy(UserCollection::FileChunks)).insert_one(file_chunk, None);
        match result {
            Err(_) => {
                
                return false;
            }
            Ok(_) => {
                return true;
            }
        }
    }





    pub fn update_file_data_hash(&self, file: FileData) -> bool {
        let status = self.collection::<FileData>(CultPardy(UserCollection::FileData)).update_one(
            doc! {"validate_hash.hash": &file.validate_hash.get_hash(), "file_name": &file.file_name, "uploader.id": &file.uploader.id},
                 doc! {"$set": {"filedata_hash.hash": &file.filedata_hash.get_hash()}},
            None);
        match status {
            Err(_) => {
                return false;
            }
            Ok(update) => {
                match update.modified_count {
                    0 => return false,
                    _ => return true,
                }
            }
        }
    }



    pub fn is_file_valide(&self, name: &str) -> bool {
        let file = match self.get_cfile_from_name(name) {
            None => return false,
            Some(file) => file,
        };
        file.is_valid()
    }


    pub fn is_file_chunk_valide(&self, name:&str, hash: &FileChunkHash) -> bool {   
        let file_data = match self.get_file_data_from_name(&name) {
            None => {
                println!("No FileData");
                return false;
            },
            Some(file_chunk) => file_chunk,
        };
        file_data.containts_file_chunk_hash(hash)
    }

    pub fn is_last_file_chunk(&self, name: &str) -> bool {
        let file_data: FileData = match self.get_file_data_from_name(&name) {
            None => return false,
            Some(file_chunk) => file_chunk,
        };

        let count = match self.get_file_chunks_count(name) {
            None => return false,
            Some(count) => count,
        };
        file_data.total_chunks.clone() as u64  == count
    }


    pub fn get_file_chunks_count(&self, name: &str) -> Option<u64> {
        let result = self.collection::<FileChunk>(CultPardy(UserCollection::FileChunks)).count_documents(doc! {"file_name": &name}, None);
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => Some(data),
        }
    }





    pub  fn get_file_data_from_user_session(&self, user_session_id: &UserSessionId) -> HashSet<FileData> {
        let result = self.collection::<FileData>(CultPardy(UserCollection::FileData)).find(doc! {"uploader.id": &user_session_id.id}, None);
        match result {
            Err(_) => {
                return HashSet::new();
            }
            Ok(data) => {
                let mut set = HashSet::new();
                for doc in data {
                    if let Ok(doc) = doc {
                        set.insert(doc);
                    }
                }
                return set;
            }
        }
    }

    pub fn get_file_chunks_from_user_session(&self, user_session_id: &UserSessionId) -> HashSet<FileChunk> {
        let result = self.collection::<FileChunk>(CultPardy(UserCollection::FileChunks)).find(doc! {"uploader.id": &user_session_id.id}, None);
        match result {
            Err(_) => {
                return HashSet::new();
            }
            Ok(data) => {
                let mut set = HashSet::new();
                for doc in data {
                    if let Ok(doc) = doc {
                        set.insert(doc);
                    }
                }
                return set;
            }
        }
    }
    

    pub fn get_file_chunks(&self, file_name: &str) -> Vec<FileChunk> {
        let result = self.collection::<FileChunk>(CultPardy(UserCollection::FileChunks)).find(doc! {"file_name": &file_name}, None);
        match result {
            Err(_) => {
                return Vec::new();
            }
            Ok(data) => {
                let mut set = Vec::new();
                for doc in data {
                    if let Ok(doc) = doc {
                        set.push(doc);
                    }
                }
                return set;
            }
        }
    }

    pub fn get_file_chunks_by_index(&self, file_name: &str, index: &usize) -> Option<FileChunk> {
        let index = index.clone() as i64;
        let result = self.collection::<FileChunk>(CultPardy(UserCollection::FileChunks)).find_one(doc! {"file_name": &file_name.to_string(), "index": &index}, None);
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }


    pub fn get_file_chunk_by_hash(&self, hash: &FileChunkHash) -> Option<FileChunk> {
        let result = self.collection::<FileChunk>(CultPardy(UserCollection::FileChunks)).find_one(doc! {"filechunk_hash.hash": &hash.get_hash()}, None);
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }
    
    
    pub fn get_cfile_from_name(&self, name: &str) -> Option<CFile> {
        let file_data = match self.get_file_data_from_name(name) {
            None =>  return None,
            Some(file_data) => file_data,
        };
        let file_chunks = self.get_file_chunks(name);
        Some(CFile{
            file_data,
            file_chunks,
        })
    }

    pub fn is_file_data(&self, name: &str) -> bool {
        let result = self.collection::<FileData>(CultPardy(UserCollection::FileData)).find_one(doc! {"file_name": &name}, None);
        match result {
            Err(_) => {
                return false;
            }
            Ok(result) => {
                match result {
                    None =>  return false,
                    Some(_) =>  return true,
            
                }
            }
        }
    }



    pub fn get_file_data_from_name(&self, name: &str) -> Option<FileData> {
        let result = self.collection::<FileData>(CultPardy(UserCollection::FileData)).find_one(doc! {"file_name": &name}, None);
        match result {
            Err(_) => {
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
            Err(_) => {
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