use super::block::Block;
use super::mesh::MeshType;
use super::block::state::{BlockState, Matter};
use super::chunk::{CHUNK_SIZE, CHUNK_BLOCKS};
use crate::datatype::{Position, ChunkUnit};
use super::texture::Texture;

use oorandom::{Rand32, Rand64};

use std::collections::HashMap;
use std::ops::Range;


#[derive(Clone)]
pub struct Terrain {
    pub blocks: HashMap<String, Block>,  // TODO: make it as a vector of enums
    random: Rand64,
}

impl Terrain {
    pub fn new(texture: &Texture, seed: u128) -> Self {
        println!("TERRAIN - INITIALIZED");

        let mut blocks = HashMap::new();

        blocks.insert("air".into(),
                      Block::new(
                          "air".into(),
                          MeshType::cube_all(texture.id_name("air".into()).unwrap()),
                          BlockState {matter: Matter::Gas, transparent: true, ..Default::default()}
                      )
        );
        blocks.insert("dirt".into(),
                      Block::new(
                          "dirt".into(),
                          MeshType::cube_all(texture.id_name("dirt".into()).unwrap()),
                          BlockState {..Default::default()}
                      )
        );
        blocks.insert("grass".into(),
                      Block::new(
                          "grass".into(),
                          MeshType::Cube {
                              top: texture.id_name("grass_top".into()).unwrap(),
                              bottom: texture.id_name("dirt".into()).unwrap(),
                              left: texture.id_name("grass_side".into()).unwrap(),
                              right: texture.id_name("grass_side".into()).unwrap(),
                              front: texture.id_name("grass_side".into()).unwrap(),
                              back: texture.id_name("grass_side".into()).unwrap()
                          },
                          BlockState {..Default::default()}
                      )
        );
        blocks.insert("stone".into(),
                      Block::new(
                          "stone".into(),
                          MeshType::cube_all(texture.id_name("stone".into()).unwrap()),
                          BlockState {..Default::default()}
                      )
        );

        Self {
            blocks: blocks,

            // Rand64 from oorandom is deterministic random number generator which is really REALLY useful
            // in deterministic natueral world.terrain generation like this sandbox game. Which is why it must
            // be instanced once, or else it would return the same result for each new instance created.
            random: Rand64::new(seed),
        }
    }

    pub fn generate_chunk(&mut self, chunk_pos: Position<ChunkUnit>) -> Box<[Block; CHUNK_BLOCKS]> {
        println!("Terrain size allocated: {:?} Blocks", CHUNK_BLOCKS);

        let ground_level = 20i64;

        let hmap = self.generate_heightmap();
        // let mut num = self.random.rand_range(Range {start: 0, end: 5}) as i64;

        // println!("N: {:?}", num);
        // println!("hmap: {:?}", hmap);

        // the global chunk coordinate in blocks
        let gx = chunk_pos.x.into_inner() as i64*CHUNK_SIZE as i64;
        let gy = chunk_pos.y.into_inner() as i64*CHUNK_SIZE as i64;
        let gz = chunk_pos.z.into_inner() as i64*CHUNK_SIZE as i64;

        let blocks = (0..CHUNK_BLOCKS).map(|n| {
            // local world.block coordinates
            let lx = ((n / (CHUNK_SIZE*CHUNK_SIZE)) % CHUNK_SIZE) as i64;
            let ly = ((n / CHUNK_SIZE) % CHUNK_SIZE) as i64;
            let lz = (n % CHUNK_SIZE) as i64;

            // global world.block coordinates
            let x = lx+gx;
            let y = ly+gy;
            let z = lz+gz;

            // 2D-3D: X to X, Y to Z, Z
            let num = hmap[lx as usize][lz as usize] as i64;

            if ground_level-num > y && y >= ground_level-num-1 {
                self.blocks["grass".into()]
            } else if ground_level-num-1 > y && y >= ground_level-num-3 {
                self.blocks["dirt".into()]
            } else if y < ground_level-num-3 {
                self.blocks["stone".into()]
            } else {
                self.blocks["air".into()]
            }
        }).collect::<Vec<_>>().into_boxed_slice();

        // this converts the slice type to an actual statically defined length array
        let block_data;
        unsafe {
            block_data = Box::from_raw(Box::into_raw(blocks) as *mut [Block; CHUNK_BLOCKS]);
        }
        block_data

        // let ground_level = 120;
        //
        // let mut block_data = [[[self.blocks["air".into()]; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        //
        // let some_seed = 4;
        // let mut num = Rand32::new(some_seed).rand_range(Range {start: 0, end: 5});
        //
        // for x in position.x..position.x+CHUNK_SIZE as u32 {
        //     for y in position.y..position.y+CHUNK_SIZE as u32 {
        //         for z in position.z..position.z+CHUNK_SIZE as u32 {
        //             if y >= ground_level-num-2 {
        //                 block_data[x as usize][y as usize][z as usize] = self.blocks["grass"];
        //             } else if y >= ground_level-num-5 {
        //                 block_data[x as usize][y as usize][z as usize] = self.blocks["dirt"];
        //             } else {
        //                 block_data[x as usize][y as usize][z as usize] = self.blocks["stone"];
        //             }
        //         }
        //     }
        // }
    }

    // TERRAIN GENERATION STAGE 1: Generating the basic heightmap
    fn generate_heightmap(&mut self) -> [[u8; CHUNK_SIZE]; CHUNK_SIZE] {
        let mut height_map = [[0; CHUNK_SIZE]; CHUNK_SIZE];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                height_map[x][y] = self.random.rand_range(Range {start: 0, end: 5}) as u8;
            }
        }

        height_map
    }
}
