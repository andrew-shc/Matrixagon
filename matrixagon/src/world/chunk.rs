use crate::world::block::Block;
use crate::world::ChunkID;
use crate::datatype::{Position, LocalBU, ChunkUnit, BlockUnit};


pub const CHUNK_SIZE: usize = 32;  // note: attempting to change this might cause a lot of problems
pub const CHUNK_BLOCKS: usize = CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE;  // blocks in a chunk


#[derive(Clone)]
pub struct Chunk {
    pub id: ChunkID,
    pub visible: bool,  // is it visible for frustum culling
    pub position: Position<ChunkUnit>,  // position by chunk sizes
    pub block_data: Box<[Block; CHUNK_BLOCKS]>,  // abstract block data
    // each bit of u32 represent each vertical layer of the chunk that all have opaque blocks
    // (32) higher y-level < > lower y-level (0)
    // 1 == opaque; 0 == at least one is transparent
    pub layers: u32,
}

impl Chunk {
    pub fn new(id: ChunkID, position: Position<ChunkUnit>, block_data: Box<[Block; CHUNK_BLOCKS]>) -> Self {
        let chunk_size = CHUNK_SIZE as u32;
        let mut layers = 0u32;

        for l in 0..32u32 {
            let mut opaque = true;  // if all of that layer is opaque
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    if block_data[((x as u32%chunk_size)*chunk_size*chunk_size+(l%chunk_size)*chunk_size+(z as u32%chunk_size)) as usize].state.transparent {
                        opaque = false;
                        break;
                    }
                }
            }
            if opaque {
                layers = layers | (1 << l)
            }
        }

        // print!("L: ");
        // for l in 0..32u32 {
        //     print!("{}", (layers & (1 << l)) >> l);
        // }
        // println!("");

        Self {
            id: id,
            visible: true,  // TODO: Conditional; For frustum culling
            position: position,
            block_data: block_data,
            layers: layers,
        }
    }

    // TODO: temporary; will create a proper chunk interface; or use the event system
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
