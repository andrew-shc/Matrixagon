use self::cube::Cube;
use crate::mesh::cube::Side;
use crate::chunk::{Chunk, ChunkUpdate};
use crate::shader::{VertexType, IndexType};
use crate::world::ChunkID;
use crate::texture::{Texture, TextureID};
use crate::datatype::Dimension;
use crate::player::Player;

use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DrawIndexedError, DynamicState};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::buffer::{BufferAccess, TypedBufferAccess, CpuAccessibleBuffer};
use vulkano::pipeline::input_assembly::Index;

use std::sync::Arc;
use vulkano::descriptor::DescriptorSet;
use std::rc::Rc;


pub mod cube;

// MeshType denotes what type of meshes the object uses with the object's texture info
#[derive(Copy, Clone)]
pub enum MeshType {  // all TextureID has a same lifetime
    // an texture id for each of the 6 side of the cube
    Cube {top: TextureID, bottom: TextureID, left: TextureID,
        right: TextureID, front: TextureID, back: TextureID},
    // direction of each pane, since the Flora mesh uses 2 pane to create an x-shape
    // Flora {positive: u8, negative: u8}
}

impl MeshType {
    // all sides have uniform texture
    pub fn cube_all(name: TextureID) -> Self {
        MeshType::Cube {
            top: name.clone(),
            bottom: name.clone(),
            left: name.clone(),
            right: name.clone(),
            front: name.clone(),
            back: name.clone(),
        }
    }

    //  all sides have uniform texture except the chosen side using the `single` texture
    pub fn cube_except_one(name: TextureID, single: TextureID, side: Side) -> Self {
        MeshType::Cube {
            top: if let Side::Top = side {single.clone()} else {name.clone()},
            bottom: if let Side::Bottom = side {single.clone()} else {name.clone()},
            left: if let Side::Left = side {single.clone()} else {name.clone()},
            right: if let Side::Right = side {single.clone()} else {name.clone()},
            front: if let Side::Front = side {single.clone()} else {name.clone()},
            back: if let Side::Back = side {single.clone()} else {name.clone()},
        }
    }
}

// a mesh manager struct for managing meshes and contexts
pub struct Meshes<'c> {
    cube: Cube<'c>,
}

impl<'c> Meshes<'c> {
    pub fn new(
        device: Arc<Device>,
        txtr: &Texture,
        renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dimensions: Dimension<u32>,
    ) -> Self {
        println!("MESHES - INITIALIZED");

        Self {
            cube: Cube::new(device.clone(), txtr, renderpass.clone(), dimensions),
        }
    }

    pub fn load_chunk(&mut self, chunk: Rc<Chunk>) {
        self.cube.load_chunk(chunk.clone());
    }

    // update meshes
    pub fn update(&mut self, dimensions: Dimension<u32>, player: &Player) {
        self.cube.updt_world(dimensions, player);
    }

    // TODO: rendering using .draw_mesh() off the CommandBufferBUilder
    pub fn render<'d>(
        &mut self,
        device: Arc<Device>,
        renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dimensions: Dimension<u32>,
        rerender: bool,
        chunk_status: ChunkUpdate,
    ) -> Vec<(
        Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
        DynamicState,  // dynamic state for display
        Arc<CpuAccessibleBuffer<[impl VertexType]>>,   // vertex buffer
        Arc<CpuAccessibleBuffer<[impl Index]>>,  // index buffer
        Vec<Arc<dyn DescriptorSet+Send+Sync+'d>>,   // sets (aka uniforms) buffer
        (),   // push-down constants TODO: A Generic Return of PushDown Constants
    )> {
        let mut gp_data = Vec::new();
        gp_data.push(self.cube.render(device.clone(), renderpass.clone(), dimensions, rerender, chunk_status));
        gp_data
    }
}

// all meshes must be implemented by the mesh trait
pub trait Mesh {
    type Vertex: VertexType + 'static;
    type Index: IndexType + 'static;

    type PushConstants; // optional pushdown constants

    fn load_chunk(&mut self, chunk: Rc<Chunk>);  // add the chunk to the chunk database of the mesh
    fn updt_chunk(&mut self, id: &ChunkID);  // updates the chunk (blocks, lighting, other chunk-bound info)
    fn remv_chunk(&mut self, id: &ChunkID);  // remove the chunk from the chunk database of the mesh
    fn updt_world(&mut self, dimensions: Dimension<u32>, player: &Player);  // updates world-bound info
    fn render<'b>(&mut self,
                  device: Arc<Device>,
                  render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  dimensions: Dimension<u32>,
                  rerender: bool,
                  chunk_status: ChunkUpdate,
    ) -> (
            Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
            DynamicState,  // dynamic state for display
            Arc<CpuAccessibleBuffer<[Self::Vertex]>>,   // vertex buffer
            Arc<CpuAccessibleBuffer<[Self::Index]>>,  // index buffer
            Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>,   // sets (aka uniforms) buffer
            Self::PushConstants,   // constants
    );  // retrieve the render data in the form of (vertices, indices)
}

// NOTE: THIS IS AN EXTENSION TRAIT
// ...for the Vulkano's AutoCommandBufferBuilder to easily add meshes to the world
// the only things this was needed is for convenience and future implication on adding meshes
pub trait MeshesExt {
    fn draw_mesh<V, I>(
        &mut self,
        mesh_data: (
            Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
            DynamicState,  // dynamic state for display
            Arc<CpuAccessibleBuffer<[V]>>,   // vertex buffer
            Arc<CpuAccessibleBuffer<[I]>>,  // index buffer
            Vec<Arc<dyn DescriptorSet+Send+Sync>>,   // sets (aka uniforms) buffer
            (),   // constants TODO: generic type
    )) -> Result<&mut Self, DrawIndexedError>
        where Self: Sized,
              V: VertexType + Send + Sync + 'static,
              I: Index + Send + Sync + 'static,
              CpuAccessibleBuffer<[V]>: BufferAccess+TypedBufferAccess;
}

impl MeshesExt for AutoCommandBufferBuilder {
    fn draw_mesh<V, I>(
        &mut self,
        mesh_data: (
            Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
            DynamicState,  // dynamic state for display
            Arc<CpuAccessibleBuffer<[V]>>,   // vertex buffer
            Arc<CpuAccessibleBuffer<[I]>>,  // index buffer
            Vec<Arc<dyn DescriptorSet+Send+Sync>>,   // sets (aka uniforms) buffer
            (),   // push constants TODO: generic type
        )
    ) -> Result<&mut Self, DrawIndexedError>  // TODO: Vulkano 0.19.0 uses `&mut Self`
            where Self: Sized,
                  V: VertexType + Send + Sync + 'static,
                  I: Index + Send + Sync + 'static,
                  CpuAccessibleBuffer<[V]>: BufferAccess+TypedBufferAccess,
    {
        self.draw_indexed(mesh_data.0, &mesh_data.1, vec!(mesh_data.2), mesh_data.3, mesh_data.4, mesh_data.5)
    }
}
