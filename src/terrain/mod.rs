use crate::block::Block;
use crate::mesh::MeshType;
use crate::block::state::BlockState;
use crate::chunk::{CHUNK_SIZE, CHUNK_BLOCKS};
use crate::datatype::Position;
use crate::texture::Texture;

use oorandom::Rand32;

use std::collections::HashMap;


pub struct Terrain {
    blocks: HashMap<String, Block>,
}

impl Terrain {
    pub fn new(texture: &Texture) -> Self {
        println!("TERRAIN - INITIALIZED");

        let mut blocks = HashMap::new();

        blocks.insert("air".into(),
                      Block::new(
                          "air".into(),
                          MeshType::cube_all(texture.id_name("air".into()).unwrap()),
                          BlockState {transparent: true, ..Default::default()}
                      )
        );
        blocks.insert("dirt".into(),
                      Block::new(
                          "dirt".into(),
                          MeshType::cube_all(texture.id_name("test".into()).unwrap()),
                          BlockState {..Default::default()}
                      )
        );
        blocks.insert("grass".into(),
                      Block::new(
                          "grass".into(),
                          MeshType::cube_all(texture.id_name("test".into()).unwrap()),
                          BlockState {..Default::default()}
                      )
        );
        blocks.insert("stone".into(),
                      Block::new(
                          "stone".into(),
                          MeshType::Cube {
                              top: texture.id_name("zenith".into()).unwrap(),
                              bottom: texture.id_name("nadir".into()).unwrap(),
                              left: texture.id_name("west".into()).unwrap(),
                              right: texture.id_name("east".into()).unwrap(),
                              front: texture.id_name("south".into()).unwrap(),
                              back: texture.id_name("north".into()).unwrap(),
                          },
                          BlockState {..Default::default()}
                      )
        );

        Self {
            blocks: blocks,
        }
    }

    pub fn generate_chunk(&self, position: Position<i64>) -> Box<[Block; CHUNK_BLOCKS]> {
        println!("Terrain size allocated: {:?} Blocks", CHUNK_BLOCKS);

        let blocks = (0..CHUNK_BLOCKS).map(|n| {
            self.blocks["stone".into()]
        }).collect::<Vec<_>>().into_boxed_slice();

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
}
