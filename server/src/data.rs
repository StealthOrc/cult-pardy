
use bytes::Bytes;
use cult_common::{dto::DTOFileChunk, wasm_lib::{hashs::{filechunk::FileChunkHash, validate::ValidateHash}, ids::usersession::UserSessionId, FileData}};
use serde::{Deserialize, Serialize};
use twox_hash::XxHash;
use crate::servers::game::SessionToken;


#[derive(Debug, Clone,Serialize,Deserialize, Hash,Eq, PartialEq, Default)]
pub struct FileChunk {
    pub file_name : String,
    pub index: usize,
    pub chunk: Bytes,
    pub validate_hash: ValidateHash,
}



impl FileChunk {


   

    pub fn to_file_chunk(dto_file_chunk:DTOFileChunk) -> Option<FileChunk> {
        let hash = dto_file_chunk.to_chunk_hash();
        if !dto_file_chunk.validate_hash.validate_file_chunk(&hash) {
            return None;
        }
        Some(FileChunk {
            file_name: dto_file_chunk.file_name,
            index: dto_file_chunk.index,
            chunk: dto_file_chunk.chunk,
            validate_hash: dto_file_chunk.validate_hash,
        })
    }

}


#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct CFile {
    pub file_chunks: Vec<FileChunk>,
    pub file_data: FileData,
}

impl CFile {


    pub fn current_chunks(&self) -> usize {
        self.file_chunks.len()
    }
    
    pub fn is_valid(&self) -> bool {
        self.file_chunks.len() == self.file_data.total_chunks && self.file_data.validate_hash.validate_file_data(&self.file_data.filedata_hash)
    }

    pub fn get_chunk(&self, index: usize) -> Option<&FileChunk> {
        self.file_chunks.iter().find(|x| x.index == index)
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct SessionRequest {
    pub user_session_id: UserSessionId,
    pub session_token: SessionToken,
}
