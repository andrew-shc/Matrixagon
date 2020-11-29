use crate::world::mesh::cube::Cube;
use crate::world::mesh::cube::Side;
use crate::world::chunk::Chunk;
use crate::world::shader::{VertexType, IndexType, cube_vs};
use crate::world::ChunkID;
use crate::world::texture::TextureID;
use crate::datatype::Dimension;
use crate::world::player::camera::Camera;
use crate::world::chunk_threadpool::ChunkThreadPool;
use crate::world::mesh::flora_x::FloraX;

use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DrawIndexedError, DynamicState};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::buffer::{BufferAccess, TypedBufferAccess, CpuAccessibleBuffer, CpuBufferPool};
use vulkano::pipeline::input_assembly::Index;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::DescriptorSet;

use std::sync::Arc;
use vulkano::image::ImmutableImage;
use vulkano::format::Format;
use vulkano::buffer::cpu_pool::{CpuBufferPoolChunk, CpuBufferPoolSubbuffer};
use vulkano::memory::pool::{StdMemoryPool, StdMemoryPoolAlloc};


pub mod air;
pub mod cube;
pub mod flora_x;

// MeshType denotes what type of meshes the object uses with the object's texture info
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MeshType {  // all TextureID has a same lifetime
    // a null block for temporary placeholder or default block for unknown blocks
    Null,
    // air is just an absence of block of an area
    Air,
    // an texture id for each of the 6 side of the cube
    Cube {top: TextureID, bottom: TextureID, left: TextureID,
        right: TextureID, front: TextureID, back: TextureID},
    // a x-shaped mesh using two textures; positive means facing (+x,+y) while negative means facing towards (-x,-y)
    FloraX {positive: TextureID, negative: TextureID},
    // A #-symbol shape
    // FloraH {north, south, east, west}
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

    pub fn cube_individual(top: TextureID, bottom: TextureID,
                           left: TextureID, right: TextureID,
                           front: TextureID, back: TextureID,
    ) -> Self {
        MeshType::Cube {
            top: top.clone(),
            bottom: bottom.clone(),
            left: left.clone(),
            right: right.clone(),
            front: front.clone(),
            back: back.clone(),
        }
    }
}

// a more generalized type alias for MeshDataType below
pub type MeshDataTypeFull<V: VertexType + Send + Sync, I: IndexType + Send + Sync, Pc> = (
    Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
    DynamicState,  // dynamic state for display
    // Arc<CpuAccessibleBuffer<[V]>>,   // vertex buffer
    // Arc<CpuAccessibleBuffer<[I]>>,  // index buffer
    CpuBufferPoolChunk<V, Arc<StdMemoryPool>>,  // vertex buffer
    CpuBufferPoolChunk<I, Arc<StdMemoryPool>>,  // index buffer
    Vec<Arc<dyn DescriptorSet+Send+Sync>>,   // sets (aka uniforms) buffer
    Pc,   // push-down constants TODO: A Generic Return of PushDown Constants
);

// note: generic types here is only just for a reference, compiler does not enforce it
pub type MeshDataType<V: VertexType + Send + Sync, I: IndexType + Send + Sync> = MeshDataTypeFull<V, I, ()>;

pub type MeshesDataType = Meshes<
    MeshDataType<<Cube as Mesh>::Vertex, <Cube as Mesh>::Index>,
    MeshDataType<<FloraX as Mesh>::Vertex, <FloraX as Mesh>::Index>,
>;

pub type MeshesStructType = Meshes<
    Cube,
    FloraX,
>;

// this struct is merely used to organized each individual meshes
pub struct Meshes<C, Fx> {
    pub cube: C,
    pub flora_x: Fx, // an x-shaped mesh for flora
    // flora_h: a tic-tac-toe shaped mesh for flora
    // liquid: a similar shape to cube, but transparent and slightly lower on the top
    // custom: a world.block-size bounded world.mesh
    // debug: a line based rendering to show chunk borders, hitboxed, and all those goodies
}

impl<C: Clone, Fx: Clone> Clone for Meshes<C, Fx> {
    fn clone(&self) -> Self {
        Self {
            cube: self.cube.clone(),
            flora_x: self.flora_x.clone(),
        }
    }
}

// a world mesh manager to manages multiple meshes at the same time
impl MeshesStructType {
    pub fn new(
        device: Arc<Device>,
        txtr: Arc<ImmutableImage<Format>>,
        renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dimensions: Dimension<u32>,
        cam: &Camera,
    ) -> Self {
        println!("MESHES - INITIALIZED");

        Self {
            cube: Cube::new(device.clone(), txtr.clone(), renderpass.clone(), dimensions, &cam),
            flora_x: FloraX::new(device.clone(), txtr.clone(), renderpass.clone(), dimensions, &cam),
        }
    }

    pub fn add_chunk(&mut self, chunk_id: ChunkID) {
        self.cube.add_chunk(chunk_id);
        self.flora_x.add_chunk(chunk_id);
    }

    pub fn load_chunks(&mut self, chunks: Vec<Chunk>, pool: &mut ChunkThreadPool) {
        self.cube.load_chunks(chunks.clone(), pool);
        self.flora_x.load_chunks(chunks, pool);
    }

    pub fn remv_chunk(&mut self, id: ChunkID) {
        self.cube.remv_chunk(id);
        self.flora_x.remv_chunk(id);
    }

    // update meshes
    pub fn update(&mut self, dimensions: Option<Dimension<u32>>, cam: Option<&Camera>) {
        self.cube.updt_world(dimensions, cam);
        self.flora_x.updt_world(dimensions, cam);
    }

    // re-renders the vertex and index data
    pub fn render(
        &mut self,
        device: Arc<Device>,
        renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
        rerender: bool,
        reload_chunk: bool,
    ) -> MeshesDataType {
        Meshes {
            cube: self.cube.render(device.clone(), renderpass.clone(), rerender, reload_chunk),
            flora_x: self.flora_x.render(device.clone(), renderpass.clone(), rerender, reload_chunk),
        }
    }
}

impl MeshesDataType {
    // a way to intercept the buffer mesh datas to quickly update player position and rotation without
    // the slowness from the chunk updates
    pub fn update_camera(&mut self, device: Arc<Device>, cam: &Camera, dimensions: Dimension<u32>) {
        let (proj, view, world) = cam.gen_mvp(dimensions);

        let persp_mat = CpuBufferPool::uniform_buffer(device.clone());

        let persp_buf = Some(persp_mat.next(
            cube_vs::ty::MVP {proj: proj, view: view, world: world}
        ).unwrap());

        let sets = Meshes {
            cube: {
                let layout = self.cube.0.descriptor_set_layout(1).unwrap();
                Arc::new(PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(persp_buf.as_ref().unwrap().clone()).unwrap()
                    .build().unwrap()
                )
            },
            flora_x: {
                let layout = self.flora_x.0.descriptor_set_layout(1).unwrap();
                Arc::new(PersistentDescriptorSet::start(layout.clone())
                    .add_buffer(persp_buf.as_ref().unwrap().clone()).unwrap()
                    .build().unwrap()
                )
            },
        };

        self.cube.4 = vec![self.cube.4[0].clone(), sets.cube];
        self.flora_x.4 = vec![self.flora_x.4[0].clone(), sets.flora_x];
    }
}

// all meshes must be implemented by the world.mesh trait
pub trait Mesh {
    type Vertex: VertexType + 'static;
    type Index: IndexType + 'static;

    type PushConstants; // optional pushdown constants

    // Mesh trait functionalities description:
    // add_chunk(); when you want to add chunks
    // load_chunk(); to load all the render data of the chunk to the world.mesh
    // updt_chunk(); reloads all the render data of the chunk to the world.mesh TODO: whats the point?
    // remv_chunk(); to remove the chunk reference to the world.mesh
    // updt_world(); calls this when the world information needs to be updated
    // render(); to return the graphic pipeline from the world.mesh to the main renderer

    fn add_chunk(&mut self, chunk_id: ChunkID);  // adds the reference of the chunk to the chunk database of the world.mesh
    fn load_chunks(&mut self,
                   chunks: Vec<Chunk>,
                   pool: &mut ChunkThreadPool,
    );  // loads all the chunks' data to the world.mesh's main vertices and indices vector
    fn updt_chunks(&mut self, id: ChunkID);  // updates the chunk (blocks, lighting, other chunk-bound info)
    fn remv_chunk(&mut self, id: ChunkID);  // remove the chunk from the chunk database of the world.mesh
    fn updt_world(&mut self, dimensions: Option<Dimension<u32>>, player: Option<&Camera>);  // updates world-bound info
    fn render<'b>(&mut self,
                  device: Arc<Device>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  rerender: bool,
                  reload_chunk: bool,
    ) -> MeshDataTypeFull<Self::Vertex, Self::Index, Self::PushConstants>;
        //                (
        //     Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
        //     DynamicState,  // dynamic state for display
        //     CpuBufferPoolChunk<Vec<Self::Vertex>, Arc<StdMemoryPool>>,   // vertex buffer
        //     CpuBufferPoolChunk<Vec<Self::Index>, Arc<StdMemoryPool>>,  // index buffer
        //     Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>,   // sets (aka uniforms) buffer
        //     Self::PushConstants,   // push-down constants
        // );
    //                    (
    //         Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
    //         DynamicState,  // dynamic state for display
    //         Arc<CpuAccessibleBuffer<[Self::Vertex]>>,   // vertex buffer
    //         Arc<CpuAccessibleBuffer<[Self::Index]>>,  // index buffer
    //         Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>,   // sets (aka uniforms) buffer
    //         Self::PushConstants,   // constants
    // );  // retrieve the render data in the form of (vertices, indices)
}

// NOTE: THIS IS AN EXTENSION TRAIT
// ...for the Vulkano's AutoCommandBufferBuilder to easily add meshes to the world
// the only things this was needed is for convenience and future implication on adding meshes
pub trait MeshesExt {
    fn draw_mesh<V, I>(
        &mut self,
        mesh_data: MeshDataType<V, I>
    ) -> Result<&mut Self, DrawIndexedError>
        where Self: Sized,
              V: VertexType + Send + Sync + 'static,
              I: Index + Send + Sync + 'static,
              // CpuAccessibleBuffer<[V]>: BufferAccess+TypedBufferAccess;
    ;
}

impl MeshesExt for AutoCommandBufferBuilder {
    fn draw_mesh<V, I>(
        &mut self,
        mesh_data: MeshDataType<V, I>,
    ) -> Result<&mut Self, DrawIndexedError>
            where Self: Sized,
                  V: VertexType + Send + Sync + 'static,
                  I: Index + Send + Sync + 'static,
                  // CpuAccessibleBuffer<[V]>: BufferAccess+TypedBufferAccess,
    {
        self.draw_indexed(mesh_data.0, &mesh_data.1, vec![Arc::new(mesh_data.2)], mesh_data.3, mesh_data.4, mesh_data.5)
    }
}
