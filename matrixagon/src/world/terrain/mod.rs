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
    registry: Arc<BlockRegistry>,

    random: Rand64,
    perlin2d: PerlinNoise2D,
}

impl Terrain {
    pub fn new(seed: u128, block_reg: Arc<BlockRegistry>) -> Self {
        println!("TERRAIN - INITIALIZED");

        Self {
            registry: block_reg.clone(),

            // Rand64 from oorandom is deterministic random number generator which is really REALLY useful
            // in deterministic natural terrain generation like this sandbox game. Which is why it must
            // be instanced once, or else it would return the same result for each new instance created.
            random: Rand64::new(seed),

            perlin2d: PerlinNoise2D::new(seed),
        }
    }

    // TODO: Make registry implement slicing
    pub fn generate_chunk(&mut self, chunk_pos: Position<ChunkUnit>) -> Box<[Block; CHUNK_BLOCKS]> {
        // println!("Terrain size allocated: {:?} Blocks", CHUNK_BLOCKS);

        let ground_level = 64i64;

        // the global chunk coordinate in blocks
        let gx = chunk_pos.x.into_block().inner().round() as i64;
        let gy = chunk_pos.y.into_block().inner().round() as i64;
        let gz = chunk_pos.z.into_block().inner().round() as i64;

        let hmap = self.generate_heightmap(gx, gy, gz);

        let blocks = vec![0;CHUNK_BLOCKS].iter().enumerate().map(|i|i.0).map(|n| {
            // local world.block coordinates
            let lx = (n / (CHUNK_SIZE*CHUNK_SIZE)) & (CHUNK_SIZE-1);
            let ly = (n / CHUNK_SIZE) & (CHUNK_SIZE-1);
            let lz = n & (CHUNK_SIZE-1);

            // global world.block coordinates
            let x = lx as i64+gx;
            let y = ly as i64+gy;  // y should never be below 0
            let z = lz as i64+gz;

            // 2D-3D: X to X, Y to Z, Z
            let num = hmap[lx as usize][lz as usize] as i64 - 10;  // todo: conversion error on indexes when negative form i to u?

            if ground_level+num+1 > y && y >= ground_level+num {
                if self.random.rand_range(0..10) == 0 {
                    self.registry["grass"]
                } else if self.random.rand_range(0..50) == 0 {
                    self.registry["flower"]
                } else {
                    self.registry["air"]
                }
            } else if ground_level+num > y && y >= ground_level+num-1 {
                if ground_level+num < ground_level {
                    self.registry["sand"]
                } else {
                    self.registry["grass_block"]
                }
            } else if ground_level+num-1 > y && y >= ground_level+num-3 {
                if ground_level+num < ground_level {
                    self.registry["sand"]
                } else {
                    self.registry["dirt"]
                }
            } else if y < ground_level+num-3 {
                self.registry["stone"]
            } else {
                self.registry["air"]
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
    fn generate_heightmap(&mut self, gx: i64, gy: i64, gz: i64) -> [[u32; CHUNK_SIZE]; CHUNK_SIZE] {
        // let mut height_map = (0..CHUNK_SIZE*CHUNK_SIZE).enumerate().map(|i|i.0).map(|n| {
        //     let x = (n as f32/CHUNK_SIZE as f32).floor() as isize;
        //     let z = (n%CHUNK_SIZE) as isize;
        //
        //     // Favorite Math Noise: ((((gx+x as isize) as f32/3 as f32).sin()+((gz+z as isize) as f32/3 as f32).cos())*32.0) as i32;
        //     // height_map[x][z] = ((((gx+x as isize) as f32/3 as f32).sin()+((gz+z as isize) as f32/3 as f32).cos())*32.0) as i32;
        //
        //     // let pn0 = self.perlin2d.perlin((gx as f64+x as f64)/100.0, (gz as f64+z as f64)/100.0);
        //     // let pn1 = self.perlin2d.perlin((gx+x as isize) as f64/40.0, (gz+z as isize) as f64/40.0);
        //     let pn2 = self.perlin2d.perlin((gx+x) as f64/10.0, (gz+z) as f64/10.0);
        //
        //     // try multiplication too
        //     let octaves = pn2/15.0;
        //
        //     octaves.round() as i32
        // }).collect::<Vec<_>>();
        //
        // let map;
        // unsafe {
        //     map = Box::from_raw(Box::into_raw(Box::new(height_map)) as *mut [i32; CHUNK_SIZE*CHUNK_SIZE])
        // }
        // *map

        let mut height_map = [[0; CHUNK_SIZE]; CHUNK_SIZE];

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Favorite Math Noise: ((((gx+x as isize) as f32/3 as f32).sin()+((gz+z as isize) as f32/3 as f32).cos())*32.0) as i32;
                // height_map[x][z] = ((((gx+x as isize) as f32/3 as f32).sin()+((gz+z as isize) as f32/3 as f32).cos())*32.0) as i32;

                // let pn0 = self.perlin2d.perlin((gx as f64+x as f64)/100.0, (gz as f64+z as f64)/100.0);
                // let pn1 = self.perlin2d.perlin((gx+x as isize) as f64/40.0, (gz+z as isize) as f64/40.0);
                let pn2 = self.perlin2d.perlin((gx+x as i64) as f64/10.0, (gz+z as i64) as f64/10.0);

                // try multiplication too
                let octaves = (pn2/15.0).round() + 16.0;
                height_map[x][z] = if octaves >= 0.0 {octaves as u32} else {0u32};
            }
        }

        height_map
    }
}
