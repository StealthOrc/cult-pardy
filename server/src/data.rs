use bson::{oid::ObjectId, DateTime};
use bytes::Bytes;
use cult_common::wasm_lib::{ids::usersession::UserSessionId, NumberScope};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::services::game::{FileMetadata, SessionToken};

/*
{
  "_id": {
    "$oid": "66bf784c2748b00f79839f06"
  },
  "length": {
    "$numberLong": "352313"
  },
  "chunkSize": 261120,
  "uploadDate": {
    "$date": "2024-08-16T16:03:24.124Z"
  },
  "filename": "Loading Symbol.gif",
  "metadata": "Test"
}
 */

#[derive(Debug, Clone,Serialize,Deserialize, Hash,Eq, PartialEq, ToSchema)]
pub struct FileData {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    pub length : usize,
    #[serde(rename = "chunkSize")]
    pub chunk_size: usize,
    #[serde(rename = "uploadDate")]
    pub upload_date: DateTime,
    #[serde(rename = "filename")]
    pub file_name: String,
    pub metadata: Option<FileMetadata>,
}



impl FileData {

  pub fn get_chunk_range(&self, range:NumberScope) -> NumberScope {
    let start_chunk = range.start / self.chunk_size;
    let end_chunk = range.end / self.chunk_size;
    NumberScope {
      start: start_chunk,
      end: end_chunk,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FileChunk {
    pub files_id:Option<ObjectId>,
    pub n: usize,
    pub data: Bytes,
}


#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default, ToSchema)]
pub struct SessionRequest {
    pub user_session_id: UserSessionId,
    pub session_token: SessionToken,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct FileDataRequest {
    pub file_data: Bytes
}


#[derive(Debug, Clone, ToSchema)]
pub struct BasicTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
}