use std::collections::HashMap;
use crate::world::block::Block;
use crate::world::mesh::MeshType;
use crate::world::texture::Texture;
use crate::world::block::state::{BlockState, Matter};

pub struct BlockRegistry {
    blocks: HashMap<String, Block>,
}

impl BlockRegistry {
    // initiates the block registry
    pub fn new(texture: &Texture) -> Self {
        let mut reg = Self {
            blocks: HashMap::new(),
        };

        reg.add_block(
            "air".into(),
            MeshType::cube_all(texture.id_name("air".into()).unwrap()),
            BlockState {matter: Matter::Gas, transparent: true, ..Default::default()},
        );
        reg.add_block("dirt".into(),
                      MeshType::cube_all(texture.id_name("dirt".into()).unwrap()),
                      BlockState {..Default::default()},
        );
        reg.add_block("grass".into(),
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

        reg
    }

    #[inline(always)]
    pub fn add_block(&mut self, name: String, mesh: MeshType, state: BlockState) {
        self.blocks.insert(name.clone(),
                           Block::new(
                               Box::leak(name.into_boxed_str()),
                               mesh,
                               state,
                           )
        );
    }

    #[inline(always)]
    pub fn get_block(&self, name: String) -> Block {
        self.blocks[&name]
    }
}
