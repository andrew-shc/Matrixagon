use super::block::Block;
use super::world::ChunkID;
use crate::datatype::{Position, LocalBU, ChunkUnit, BlockUnit};

use std::ops::Rem;

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

    pub fn blocks(&self, x: BlockUnit, y: BlockUnit, z: BlockUnit) -> &Block {
        &self.block_data[(x.0 as usize%CHUNK_SIZE)*CHUNK_SIZE*CHUNK_SIZE+(y.0 as usize%CHUNK_SIZE)*CHUNK_SIZE+(z.0 as usize%CHUNK_SIZE)]
    }
}

pub enum ChunkError {
    Invalid,  // TODO invalid chunk when reading

    // related to Chunk ID's
    DuplicateID,  // if there were multiple same ID
    DuplicateChunkPos,  // cannot generate chunk ID: duplicate chunk position
}


// TODO: Both New_ChunkUpdate enum and bitflag are discarded in favor of a new event system

// TODO: just use enums
pub enum New_ChunkUpdate {
    // chunk load (world.player moving around loading new chunks to be loaded)
    ChunkLoad,
    // world.block update (world.block manipulation, world.player breaking/placing blocks, etc.)
    // requires the chunk location to reload the whole chunk TODO
    BlockUpdate,
    // lighting update (light level, placed a illuminate blocks, etc.)
    LightingUpdate,

    // TODO: Theoretical implementation
    RedstoneUpdate,
}

bitflags! {
    #[derive(Default)]
    pub struct ChunkUpdate: u32 {
        // chunk load (world.player moving around loading new chunks to be loaded)
        const ChunkLoad = 0b00000001;
        // world.block update (world.block manipulation, world.player breaking/placing blocks, etc.)
        const BlockUpdate = 0b00000010;
        // lighting update (light level, placed a illuminate blocks, etc.)
        const LightingUpdate = 0b00000100;

        // TODO: Theoreticl implementation
        const RedstoneUpdate = 0b10000000;
    }
}
