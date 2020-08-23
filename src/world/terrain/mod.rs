use crate::world::block::Block;
use crate::world::chunk::{CHUNK_SIZE, CHUNK_BLOCKS};
use crate::datatype::{Position, ChunkUnit};
use crate::world::block::registry::BlockRegistry;
use crate::world::terrain::noise::PerlinNoise2D;

use oorandom::Rand64;

use std::sync::Arc;

mod noise;


#[derive(Clone)]
pub struct Terrain {
    random: Rand64,

    perlin2d: PerlinNoise2D,
}

impl Terrain {
    pub fn new(seed: u128) -> Self {
        println!("TERRAIN - INITIALIZED");

        Self {
            // Rand64 from oorandom is deterministic random number generator which is really REALLY useful
            // in deterministic natueral world.terrain generation like this sandbox game. Which is why it must
            // be instanced once, or else it would return the same result for each new instance created.
            random: Rand64::new(seed),

            perlin2d: PerlinNoise2D::new(seed),
        }
    }

    pub fn generate_chunk(&mut self, registry: Arc<BlockRegistry>, chunk_pos: Position<ChunkUnit>) -> Box<[Block; CHUNK_BLOCKS]> {
        // println!("Terrain size allocated: {:?} Blocks", CHUNK_BLOCKS);

        let ground_level = 64i64;

        // the global chunk coordinate in blocks
        let gx = chunk_pos.x.into_inner() as i64*CHUNK_SIZE as i64;
        let gy = chunk_pos.y.into_inner() as i64*CHUNK_SIZE as i64;
        let gz = chunk_pos.z.into_inner() as i64*CHUNK_SIZE as i64;

        let hmap = self.generate_heightmap(gx as isize, gy as isize, gz as isize);

        let blocks = vec![0;CHUNK_BLOCKS].iter().enumerate().map(|i|i.0).map(|n| {
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

            if ground_level-num+1 > y && y >= ground_level-num {
                if self.random.rand_range(0..10) == 0 {
                    registry.block("grass".into())
                } else if self.random.rand_range(0..50) == 0 {
                    registry.block("flower".into())
                } else {
                    registry.block("air".into())
                }
            } else if ground_level-num > y && y >= ground_level-num-1 {
                if ground_level-num < ground_level {
                    registry.block("sand".into())
                } else {
                    registry.block("grass_block".into())
                }
            } else if ground_level-num-1 > y && y >= ground_level-num-3 {
                if ground_level-num < ground_level {
                    registry.block("sand".into())
                } else {
                    registry.block("dirt".into())
                }
            } else if y < ground_level-num-3 {
                registry.block("stone".into())
            } else {
                registry.block("air".into())
            }
        }).collect::<Vec<_>>().into_boxed_slice();

        // this converts the slice type to an actual statically defined length array
        let block_data;
        unsafe {
            block_data = Box::from_raw(Box::into_raw(blocks) as *mut [Block; CHUNK_BLOCKS]);
        }
        block_data
    }

    // TERRAIN GENERATION STAGE 1: Generating the basic heightmap
    fn generate_heightmap(&mut self, gx: isize, gy: isize, gz: isize) -> [[i32; CHUNK_SIZE]; CHUNK_SIZE] {
        let mut height_map = [[0; CHUNK_SIZE]; CHUNK_SIZE];

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Favorite Math Noise: ((((gx+x as isize) as f32/3 as f32).sin()+((gz+z as isize) as f32/3 as f32).cos())*32.0) as i32;
                // height_map[x][z] = ((((gx+x as isize) as f32/3 as f32).sin()+((gz+z as isize) as f32/3 as f32).cos())*32.0) as i32;

                // let pn0 = self.perlin2d.perlin((gx as f64+x as f64)/100.0, (gz as f64+z as f64)/100.0);
                // let pn1 = self.perlin2d.perlin((gx+x as isize) as f64/40.0, (gz+z as isize) as f64/40.0);
                let pn2 = self.perlin2d.perlin((gx+x as isize) as f64/10.0, (gz+z as isize) as f64/10.0);

                // try multiplication too
                let octaves = pn2/15.0;

                height_map[x][z] = octaves.round() as i32;
            }
        }

        height_map
    }
}
