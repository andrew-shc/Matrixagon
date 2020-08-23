use crate::world::mesh::MeshType;
use crate::world::block::state::BlockState;
use crate::world::block::registry::BlockID;

pub mod state;
pub mod registry;


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Block {
    pub id: BlockID,
    pub name: &'static str,  // world.block name as world.block id
    pub mesh: MeshType,  // the parent world.mesh
    pub state: BlockState,  // world.block state info TODO
}

impl Block {
    pub fn new(id: BlockID, name: &'static str, mesh: MeshType, state: BlockState) -> Self {  // create new world.block
        Self {
            id,
            name,
            mesh,
            state,
        }
    }

    // creates a new, temporary world.block for placeholder usages in static arrays
    // NOTE: NULL BLOCK MUST BE IMMEDIATELY REPLACED AFTER ITS CREATION; IT MUST NOT BE IN THE FINAL RESULT
    pub fn null() -> Self {
        Self {
            id: BlockID(0),
            name: "null",
            mesh: MeshType::Null,
            state: BlockState::default(),
        }
    }
}

/*
Block::new(BlockName, BlockID, BlockMesh, BlockTexture, BlockState)
 */