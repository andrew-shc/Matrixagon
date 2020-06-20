use crate::block::Block;
use crate::world::ChunkID;
use crate::datatype::Position;

pub const CHUNK_SIZE: usize = 64;
pub const CHUNK_BLOCKS: usize = CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE;  // blocks in a chunk


pub struct Chunk {
    pub id: ChunkID,
    pub visible: bool,  // is it visible for frustum culling
    pub position: Position<u32>,  // chunk position
    pub block_data: Box<[Block; CHUNK_BLOCKS]>,  // abstract block data
}

impl Chunk {
    pub fn new(id: ChunkID, position: Position<u32>, block_data: Box<[Block; CHUNK_BLOCKS]>) -> Self {
        Self {
            id: id,
            visible: true,  // TODO: Conditional; For frustum culling
            position: position,
            block_data: block_data,
        }
    }
}

pub enum ChunkError {
    Invalid,  // TODO invalid chunk

    // related to Chunk ID's
    DuplicateID,  // if there were multiple same ID
    DuplicateChunkPos,  // cannot generate chunk ID: duplicate chunk position
}