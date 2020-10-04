use crate::world::block::Block;
use crate::world::mesh::MeshType;
use crate::world::texture::Texture;
use crate::world::block::state::{BlockState, Matter};

use std::collections::HashMap;
use std::ops::Index;


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct BlockID(pub u32);

pub struct BlockRegistry {
    blocks: HashMap<BlockID, Block>,
    id_counter: u32,
}

impl BlockRegistry {
    // initiates the block registry
    // adds all the basic blocks of matrixagon
    pub fn new(texture: &Texture) -> Self {
        let mut reg = Self {
            blocks: HashMap::new(),
            id_counter: 1,  // 0 BlockID is null
        };

        reg.add_block(
            "air".into(),
            MeshType::Air,
            BlockState {matter: Matter::Gas, transparent: true, ..Default::default()},
        );
        reg.add_block("dirt".into(),
                      MeshType::cube_all(texture.id_name("dirt".into()).unwrap()),
                      BlockState {..Default::default()},
        );
        reg.add_block("grass_block".into(),
                      MeshType::Cube {
                              top: texture.id_name("grass_top".into()).unwrap(),
                              bottom: texture.id_name("dirt".into()).unwrap(),
                              left: texture.id_name("grass_side".into()).unwrap(),
                              right: texture.id_name("grass_side".into()).unwrap(),
                              front: texture.id_name("grass_side".into()).unwrap(),
                              back: texture.id_name("grass_side".into()).unwrap()
                      },
                      BlockState {..Default::default()}
        );
        reg.add_block("stone".into(),
                      MeshType::cube_all(texture.id_name("stone".into()).unwrap()),
                      BlockState {..Default::default()}
        );
        reg.add_block("sand".into(),
                      MeshType::cube_all(texture.id_name("sand".into()).unwrap()),
                      BlockState {..Default::default()}
        );
        reg.add_block("grass".into(),
                      MeshType::FloraX {
                          positive: texture.id_name("grass_flora".into()).unwrap(),
                          negative: texture.id_name("grass_flora".into()).unwrap(),
                      },
                      BlockState {transparent: true, ..Default::default()}
        );
        reg.add_block("flower".into(),
                      MeshType::FloraX {
                          positive: texture.id_name("flower".into()).unwrap(),
                          negative: texture.id_name("flower".into()).unwrap(),
                      },
                      BlockState {transparent: true, ..Default::default()}
        );

        reg
    }

    #[inline(always)]
    pub fn add_block(&mut self, name: String, mesh: MeshType, state: BlockState) {
        self.blocks.insert(BlockID(self.id_counter),
                           Block::new(
                               BlockID(self.id_counter),
                               Box::leak(name.into_boxed_str()),
                               mesh,
                               state,
                           )
        );
        self.id_counter += 1;
    }

    #[inline(always)]
    pub fn block(&self, name: String) -> Block {
        self.blocks[&self.block_id(name).unwrap()]
    }

    #[inline(always)]
    pub fn block_id(&self, name: String) -> Option<BlockID> {
        for hmap in self.blocks.iter() {
            if hmap.1.name == name {
                return Some(*hmap.0);
            }
        }
        None
    }
}

impl Index<String> for BlockRegistry {
    type Output = Block;

    fn index(&self, index: String) -> &Self::Output {
        &self.blocks[&self.block_id(index).unwrap()]
    }
}

impl Index<&str> for BlockRegistry {
    type Output = Block;

    fn index(&self, index: &str) -> &Self::Output {
        &self.blocks[&self.block_id(String::from(index)).unwrap()]
    }
}
