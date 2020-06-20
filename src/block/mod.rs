use crate::mesh::MeshType;
use crate::block::state::BlockState;

pub mod state;


#[derive(Copy, Clone)]
pub struct Block {
    pub name: &'static str,  // block name as block id
    pub mesh: MeshType,  // the parent mesh
    pub state: BlockState,  // block state info TODO
}

impl Block {
    pub fn new(name: &'static str, mesh: MeshType, state: BlockState) -> Self {  // create new block
        Self {
            name,
            mesh,
            state,
        }
    }
}

/*
Block::new(BlockName, BlockID, BlockMesh, BlockTexture, BlockState)
 */