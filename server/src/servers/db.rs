use std::collections::HashSet;
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use cult_common::wasm_lib::hashs::filechunk::FileChunkHash;
use cult_common::wasm_lib::hashs::validate::ValidateHash;
use cult_common::wasm_lib::ids::discord::DiscordID;
use cult_common::wasm_lib::{FileData};
use futures::{AsyncWriteExt, StreamExt};
use mongodb::action::gridfs::OpenUploadStream;
use mongodb::bson::{doc, to_bson};
use mongodb::gridfs::GridFsBucket;
use mongodb::options::{ClientOptions, GridFsBucketOptions, ServerApi, ServerApiVersion, WriteConcern};
use mongodb::{Client, Collection, IndexModel};
use ritelinked::LinkedHashSet;
use strum::{Display, EnumIter};
use cult_common::wasm_lib::ids::usersession::UserSessionId;
use crate::data::{CFile, FileChunk};
use crate::servers::db::DBDatabase::CultPardy;
use crate::servers::game::UserSession;
use crate::ws::session;

use super::authentication::{self, Admin};
use super::game::{DiscordData, SessionToken};


#[derive(Clone,Display, Debug, Default)]
pub enum DBDatabase{
    CultPardy(UserCollection),
    #[default]
    None,
}


#[derive(Clone, Debug)]
pub struct UserCollection {
    pub user_sessions: Collection<UserSession>,
    pub admins: Collection<Admin>,
    pub file_data: Collection<FileData>,
    pub file_chunks: Collection<FileChunk>,
    pub file_bucket: GridFsBucket,
    pub file_bucket_files: Collection<FileData>,
    pub file_bucket_chunks: Collection<FileChunk>,

}


#[derive(Clone, Debug)]
pub struct MongoServer {	
    pub mongo_client: Client,
    pub collections:  UserCollection,
}

//GridFsMetadata for fle data

pub struct FileMetadata {
    pub file_name: String,
    pub file_type: String,
    pub files_size: u64,
    pub validate_hash: ValidateHash,
    pub uploader: DiscordID,
}




impl MongoServer {

    pub async fn new() -> Self{
        let url = std::env::var("MONGODB_URI").expect("Cant get MONGODB_URI");
        let mongo_client = Client::with_uri_str(&url).await.expect("Can´t connect to Mongodb");
        mongo_client.database("admin").run_command(doc! {"ping": 1}).await.expect("Cant ping");
        println!("Pinged your deployment. You successfully connected to MongoDB!");
    


        
        let wc = WriteConcern::builder().w_timeout(Duration::new(5, 0)).build();
        let opts = GridFsBucketOptions::builder()
            .bucket_name("FileBucket".to_string())
            .write_concern(wc)
            .build();



        let db = mongo_client.database("CultPardy");
        let bucket = db.gridfs_bucket(opts);




        let collections = UserCollection{
            user_sessions: db.collection("UserSessions"),
            admins: db.collection("Admins"),
            file_data: db.collection("FileData"),
            file_chunks : db.collection("FileChunks"),
            file_bucket: bucket,
            file_bucket_files: db.collection("FileBucket.chunks"),
            file_bucket_chunks: db.collection("FileBucket.chunks"),
        };

        collections.user_sessions.create_index(IndexModel::builder().keys(doc! {"user_session_id.id": 1}).build()).await.expect("Failed to create index");
        collections.admins.create_index(IndexModel::builder().keys(doc! {"discord_id.id": 1}).build()).await.expect("Failed to create index");
        collections.file_data.create_index(IndexModel::builder().keys(doc! {"file_name": 1}).build()).await.expect("Failed to create index");
        collections.file_chunks.create_index(IndexModel::builder().keys(doc! {"file_name": 1}).build()).await.expect("Failed to create index");
        collections.file_chunks.create_index(IndexModel::builder().keys(doc! {"index": 1}).build()).await.expect("Failed to create index");
        collections.file_chunks.create_index(IndexModel::builder().keys(doc! {"filechunk_hash.hash": 1}).build()).await.expect("Failed to create index");
        collections.file_data.create_index(IndexModel::builder().keys(doc! {"uploader.id": 1}).build()).await.expect("Failed to create index");

        MongoServer{
            mongo_client,
            collections,
        }
    }


    

    pub async fn get_user_session_with_token(&self, user_session_id: &UserSessionId, session_token:&SessionToken) -> UserSession {
        println!("Getting UserSession with token check");
        let result = self.collections.user_sessions.find_one(doc! {"user_session_id.id": &user_session_id.id, "session_token.token": &session_token.token}).await;
        let optional_session = match result {
            Err(_) => {
                println!("Something went wrong");
                return self.new_user_session().await;
            }
            Ok(data) => data,
        };
        return match optional_session {
            None => {
                self.new_user_session().await
            }
            Some(session) => session
        };
    }

    //upload file to and file bucket with metadata
    pub async fn upload_file_to_file_bucket(&self, file_bytes:Bytes, file_metadata:FileMetadata) -> bool {
        let file_name = file_metadata.file_name.clone();
        let file_type = file_metadata.file_type.clone();
        let file_size = file_metadata.files_size.clone();
        let validate_hash = file_metadata.validate_hash.clone();
        let uploader = file_metadata.uploader.clone();
        let mut upload_stream = self.collections.file_bucket.open_upload_stream(file_name.clone()).await.expect("Failed to open upload stream");

       let write =  match upload_stream.write_all(&file_bytes).await {
            Err(_) => {
                return false;
            }
            Ok(_) => true,
        };
        upload_stream.close().await.expect("Failed to close upload stream");

        write
        
        
    }







    pub async fn get_user_session_with_token_check(&self, user_session_id: &UserSessionId, session_token:&SessionToken) -> UserSession {
        let result = self.collections.user_sessions.find_one(doc! {"user_session_id.id": &user_session_id.id, "session_token.token": &session_token.token}).await;
        let optional_session = match result {
            Err(_) => {
                println!("Something went wrong");
                return self.new_user_session().await;
            }
            Ok(data) => data,
        };
        return match optional_session {
            None => return self.new_user_session().await,
            Some(session) => self.check_token(session, &session_token).await
        
        };
    }


    async fn check_token(&self, db_user_session:  UserSession, current_token:&SessionToken) -> UserSession {
        if db_user_session.session_token.token.eq(&current_token.token) {
            if db_user_session.session_token.is_expired() {
                println!("Token is expired");
                let mut cloned: UserSession = db_user_session.clone();
                let token = cloned.session_token.update();
                println!("New Token {:?}", token);
                if self.update_user_session_token(&cloned.user_session_id, &token).await {
                    let clone: UserSession = cloned.clone();
                    println!("Saved new Token {:?}", clone.session_token.token);
                    return clone.clone()
                } else {
                    println!("Failed to save new Token");
                    return self.new_user_session().await;
                }
            }
            println!("Token is the same");
            return db_user_session.clone()
        } else{
            print!("Token is not the same");
            return self.new_user_session().await;
         }
    }


    pub async fn update_user_session_token(&self, user_session_id:  &UserSessionId, session_token: &SessionToken) -> bool {
        let status = self.collections.user_sessions.update_one(
            doc! {"user_session_id.id": &user_session_id.id},
            doc! {"$set": {"session_token.token": &session_token.token}},
        ).await;
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




    pub async fn find_user_session(&self, user_session_id: &UserSessionId) -> Option<UserSession> {
        let result = self.collections.user_sessions.find_one(doc! {"user_session_id.id": &user_session_id.id}).await;
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }




    pub async fn new_user_session(&self) -> UserSession {
        let mut session= UserSession::random();
        while self.has_user_session(&session.user_session_id).await {
            session = UserSession::random();
        }
        println!("Added User-session {:?}", session.clone());
        let result = self.collections.user_sessions.insert_one(&session).await;
        result.expect("Failed to insert UserSession");
        session
    }

    pub async fn update_discord_data(&self, user_session_id: &UserSessionId, discord_data: &DiscordData) -> bool {
        println!("Updating Discord Data, session id {:?}", user_session_id);
        let status = self.collections.user_sessions.update_one(
            doc! {"user_session_id.id": &user_session_id.id},
            doc! {"$set": {"discord_auth": &discord_data}},
        ).await;
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






    pub async fn find_admin(&self, discord_id: &DiscordID) -> Option<Admin> {
        let result = self.collections.admins.find_one(doc! {"discord_id.id": &discord_id.id}).await;
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }

    pub async fn is_admin(&self, discord_id: &DiscordID) -> bool {
        match self.find_admin(discord_id).await {
            None => false,
            Some(_) => true,
        }
    }


    pub async fn add_admin(&self, admin: Admin) -> bool {
        let result = self.collections.admins.insert_one(admin).await;
        match result {
            Err(_) => {
                return false;
            }
            Ok(_) => {
                return true;
            }
        }
    }

    pub async fn get_admins(&self) -> HashSet<Admin> {
        let result = self.collections.admins.find(doc! {}).await;
        match result {
            Err(_) => {
                return HashSet::new();
            }
            Ok(mut data) => {
                let mut set = HashSet::new();
                while let Some(doc) = data.next().await {
                    if let Ok(doc) = doc {
                        set.insert(doc);
                    }
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
        let result = self.collections.file_data.insert_one(file).await;
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
        let result = self.collections.file_chunks.insert_one(file_chunk).await;
        match result {
            Err(er) => {
                println!("Error {:?}", er);
                return false;
            }   
            Ok(e) => {
                print!("Added FileChunk {:?}", e);
                return true;
            }
        }
    }





    pub async fn update_file_data_hash(&self, file: FileData) -> bool {
        let status = self.collections.file_data.update_one(
            doc! {"validate_hash.hash": &file.validate_hash.get_hash(), "file_name": &file.file_name, "uploader.id": &file.uploader.id},
                 doc! {"$set": {"filedata_hash.hash": &file.filedata_hash.get_hash()}},
          ).await;
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



    pub async fn is_file_valide(&self, name: &str) -> bool {
        let file = match self.get_cfile_from_name(name).await {
            None => return false,
            Some(file) => file,
        };
        file.is_valid()
    }


    pub async fn is_file_chunk_valide(&self, name:&str, hash: &ValidateHash) -> bool {   
        let file_data = match self.get_file_data_from_name(&name).await {
            None => {
                println!("No FileData");
                return false;
            },
            Some(file_chunk) => file_chunk,
        };
        file_data.containts_file_chunk_hash(hash)
    }

    pub async fn is_last_file_chunk(&self, name: &str) -> bool {
        let file_data: FileData = match self.get_file_data_from_name(&name).await {
            None => return false,
            Some(file_chunk) => file_chunk,
        };

        let count = match self.get_file_chunks_count(name).await {
            None => return false,
            Some(count) => count,
        };
        file_data.total_chunks.clone() as u64  == count
    }


    pub async fn get_file_chunks_count(&self, name: &str) -> Option<u64> {
        let result = self.collections.file_chunks.count_documents(doc! {"file_name": &name}).await;
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => Some(data),
        }
    }





    pub async  fn get_file_data_from_user_session(&self, user_session_id: &UserSessionId) -> HashSet<FileData> {
        let result = self.collections.file_data.find(doc! {"uploader.id": &user_session_id.id}).await;
        match result {
            Err(_) => {
                return HashSet::new();
            }
            Ok(mut data) => {
                let mut set = HashSet::new();
                while let Some(doc) = data.next().await {
                    if let Ok(doc) = doc {
                        set.insert(doc);
                    }
                }
                return set;
            }
        }
    }

    pub async fn get_file_chunks_from_user_session(&self, user_session_id: &UserSessionId) -> HashSet<FileChunk> {
        let result = self.collections.file_chunks.find(doc! {"uploader.id": &user_session_id.id}).await;
        match result {
            Err(_) => {
                return HashSet::new();
            }
            Ok(mut data) => {
                let mut set = HashSet::new();
                while let Some(doc) = data.next().await {
                    if let Ok(doc) = doc {
                        set.insert(doc);
                    }
                    
                }
                return set;
            }
        }
    }
    

    pub async fn get_file_chunks(&self, file_name: &str) -> Vec<FileChunk> {
        let result = self.collections.file_chunks.find(doc! {"file_name": &file_name}).await;
        match result {
            Err(_) => {
                return Vec::new();
            }
            Ok(mut data) => {
                let mut set = Vec::new();
                while let Some(doc) = data.next().await {
                    if let Ok(doc) = doc {
                        set.push(doc);
                    }
                }
                return set;
            }
        }
    }

    pub async fn get_file_chunks_by_index(&self, file_name: &str, index: &usize) -> Option<FileChunk> {
        let index = index.clone() as i64;
        let result = self.collections.file_chunks.find_one(doc! {"file_name": &file_name.to_string(), "index": &index}).await;
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }


    pub async fn get_file_chunk_by_hash(&self, hash: &FileChunkHash) -> Option<FileChunk> {
        let result = self.collections.file_chunks.find_one(doc! {"filechunk_hash.hash": &hash.get_hash()}).await;
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }
    
    
    pub async fn get_cfile_from_name(&self, name: &str) -> Option<CFile> {
        let file_data = match self.get_file_data_from_name(name).await {
            None =>  return None,
            Some(file_data) => file_data,
        };
        let file_chunks = self.get_file_chunks(name).await;
        Some(CFile{
            file_data,
            file_chunks,
        })
    }

    pub async fn is_file_data(&self, name: &str) -> bool {
        let result = self.collections.file_data.find_one(doc! {"file_name": &name}).await;
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



    pub async fn get_file_data_from_name(&self, name: &str) -> Option<FileData> {
        let result = self.collections.file_data.find_one(doc! {"file_name": &name}).await;
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }







    pub async fn find_user_session_with_discord(&self, discord_data: &DiscordData) -> Option<UserSession> {
        println!("Finding UserSession with Discord Data {:?}", discord_data.discord_user.clone().unwrap().discord_id.id);
        let optianal_discord_user = discord_data.discord_user.clone();
        if optianal_discord_user.is_none() {
            return None;
        }
        let id : String = optianal_discord_user.unwrap().discord_id.id.clone();
        let test = doc! {"discord_auth.discord_user.discord_id.id": &id};
        let result = self.collections.user_sessions.find_one(test).await;
        match result {
            Err(_) => {
                return None;
            }
            Ok(data) => data,
        }
    }



    pub async fn has_user_session(&self, user_session_id: &UserSessionId) -> bool {
         match self.find_user_session(&user_session_id).await{
            None => false,
            Some(_) => true,
        }
    }



}