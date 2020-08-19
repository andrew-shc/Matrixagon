use crate::world::block::Block;
use crate::world::ChunkID;
use crate::datatype::{Position, LocalBU, ChunkUnit, BlockUnit};


pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_BLOCKS: usize = CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE;  // blocks in a chunk


#[derive(Clone)]
pub struct Chunk {
    pub id: ChunkID,
    pub visible: bool,  // is it visible for frustum culling
    pub position: Position<ChunkUnit>,  // position by chunk sizes
    pub block_data: Box<[Block; CHUNK_BLOCKS]>,  // abstract world.block data
}

impl Chunk {
    pub fn new(id: ChunkID, position: Position<ChunkUnit>, block_data: Box<[Block; CHUNK_BLOCKS]>) -> Self {
        Self {
            id: id,
            visible: true,  // TODO: Conditional; For frustum culling
            position: position,
            block_data: block_data,
        }
    }

    // TODO: temporary; will create a proper chunk interface
    pub fn update(&mut self, remove_block_pos: Position<LocalBU>, block_to_be_replaced: Block) {
        (*self.block_data)[remove_block_pos.into_vec_pos()] = block_to_be_replaced;
    }

    #[inline(always)]
    pub fn blocks(&self, x: BlockUnit, y: BlockUnit, z: BlockUnit) -> &Block {
        &self.block_data[(usize::from(x)%CHUNK_SIZE)*CHUNK_SIZE*CHUNK_SIZE+(usize::from(y)%CHUNK_SIZE)*CHUNK_SIZE+(usize::from(z)%CHUNK_SIZE)]
    }
}

pub enum ChunkError {
    Invalid,  // TODO invalid chunk when reading

    // related to Chunk ID's
    DuplicateID,  // if there were multiple same ID
    DuplicateChunkPos,  // cannot generate chunk ID: duplicate chunk position
}
