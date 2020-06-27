use crate::block::Block;
use crate::world::ChunkID;
use crate::datatype::Position;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_BLOCKS: usize = CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE;  // blocks in a chunk


pub struct Chunk {
    pub id: ChunkID,
    pub visible: bool,  // is it visible for frustum culling
    pub position: Position<i64>,  // position by chunk sizes
    pub block_data: Box<[Block; CHUNK_BLOCKS]>,  // abstract block data
}

impl Chunk {
    pub fn new(id: ChunkID, position: Position<i64>, block_data: Box<[Block; CHUNK_BLOCKS]>) -> Self {
        Self {
            id: id,
            visible: true,  // TODO: Conditional; For frustum culling
            position: position,
            block_data: block_data,
        }
    }
}

pub enum ChunkError {
    Invalid,  // TODO invalid chunk when reading

    // related to Chunk ID's
    DuplicateID,  // if there were multiple same ID
    DuplicateChunkPos,  // cannot generate chunk ID: duplicate chunk position
}

bitflags! {
    #[derive(Default)]
    pub struct ChunkUpdate: u32 {
        // block update (block manipulation, player breaking/placing blocks, etc.)
        const BlockUpdate = 0b00000001;
        // lighting update (light level, placed a illuminate blocks, etc.)
        const LightingUpdate = 0b00000010;

        // TODO: Theoreticl implementation
        const RedstoneUpdate = 0b10000000;
    }
}

// pub enum ChunkUpdate {
//     None,  // no chunk update
//     BlockUpdate,  // block update (block manipulation, player breaking/placing blocks, etc.)
//     LightingUpdate,  // lighting update (light level, placed a illuminate blocks, etc.)
//
//     // TODO theoretical
//     RedstoneUpdate,  // maybe redstone update
// }
