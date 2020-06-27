use crate::datatype::{Dimension, Position};
use crate::mesh::{Mesh, MeshType};
use crate::chunk::{Chunk, CHUNK_SIZE, ChunkUpdate};
use crate::shader::{CubeVert, cube_vs, cube_fs};
use crate::world::ChunkID;
use crate::block::Block;
use crate::texture::Texture;
use crate::player::Player;

use vulkano::pipeline::viewport::Viewport;
use vulkano::framebuffer::{Subpass, RenderPassAbstract};
use vulkano::device::Device;
use vulkano::pipeline::{GraphicsPipelineAbstract, GraphicsPipeline};
use vulkano::command_buffer::DynamicState;
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage, CpuBufferPool};
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet, PersistentDescriptorSetBuilderArray};
use vulkano::descriptor::DescriptorSet;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::memory::pool::{StdMemoryPool, PotentialDedicatedAllocation, StdMemoryPoolAlloc};
use vulkano::image::ImmutableImage;
use vulkano::format::Format;

use std::sync::Arc;
use std::iter;
use std::rc::Rc;
use std::marker::PhantomData;
use std::any::Any;


const CUBE_FACES: u32 = 6;  // 6 faces in a cube (duh)
const VERT_FACES: u32 = 4;  // 4 vert in a face of a cube
const IND_FACES: u32 = 6;  // 3 vert triangles x 2

pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

// only stores an immutable reference to a vector of meshes
// the cube mesh will only re-render the render data when the is update
pub struct Cube<'c> {
    textures: Vec<Arc<ImmutableImage<Format, PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>>,
    chunks: Vec<(Rc<Chunk>, Vec<<Self as Mesh>::Vertex>, Vec<<Self as Mesh>::Index>)>,
    grph_pipe: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,

    vert_shd: cube_vs::Shader,
    frag_shd: cube_fs::Shader,

    persp_mat: CpuBufferPool<cube_vs::ty::MVP>,
    persp_buf: Option<CpuBufferPoolSubbuffer<cube_vs::ty::MVP, Arc<StdMemoryPool>>>,
    sampler: Arc<Sampler>,

    vertices: Vec<<Self as Mesh>::Vertex>,  // aggregated vertices (stored to optimize)
    indices: Vec<<Self as Mesh>::Index>,  // aggregated indices (stored to optimize)

    TEMP_LIFETIME: PhantomData<&'c str>, // TODO: we may revert back to references or change req. lifetime again
}

impl<'c> Cube<'c> {
    pub fn new(device: Arc<Device>,
               texture: &Texture,
               render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
               dimensions: Dimension<u32>
    ) -> Self {
        let vs = cube_vs::Shader::load(device.clone()).expect("failed to create cube vertex shaders module");
        let fs = cube_fs::Shader::load(device.clone()).expect("failed to create cube fragment shaders module");

        Self {
            textures: texture.texture_array(),
            chunks: Vec::new(),
            grph_pipe: Cube::pipeline(
                &vs, &fs, device.clone(),
                render_pass.clone(), dimensions.into(),
            ),

            vert_shd: vs,
            frag_shd: fs,

            persp_mat: CpuBufferPool::uniform_buffer(device.clone()),
            persp_buf: None,
            sampler: Sampler::new(device.clone(), Filter::Nearest, Filter::Nearest,
                                  MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
                                  SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 8.0).unwrap(),

            vertices: Vec::new(),
            indices: Vec::new(),

            TEMP_LIFETIME: PhantomData,
        }
    }

    // internal function for building pipeline
    fn pipeline(
        vert: &cube_vs::Shader,
        frag: &cube_fs::Shader,
        device: Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dimensions: Dimension<u32> )
        -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {

        // specialization constants will not be used
        // let spec_consts = cube_fs::SpecializationConstants {
        //     TEXTURES: texture.texture_len() as u32
        // };

        Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<<Cube as Mesh>::Vertex>()
            .vertex_shader(vert.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                dimensions: dimensions.into(),
                depth_range: 0.0 .. 1.0,
            }))
            .fragment_shader(frag.main_entry_point(), ())
            .cull_mode_front()  // face culling for optimization
            .alpha_to_coverage_enabled()  // to enable transparency
            .depth_stencil_simple_depth()  // to enable depth buffering
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone()).unwrap()
        )
    }
}

impl<'c> Mesh for Cube<'c> {
    type Vertex = CubeVert;
    type Index = u32;

    type PushConstants = ();

    fn load_chunk(&mut self, chunk: Rc<Chunk>) {
        let start = Position::new(
            chunk.position.x * CHUNK_SIZE as i64,
            chunk.position.y * CHUNK_SIZE as i64,
            chunk.position.z * CHUNK_SIZE as i64,
        );

        let end = Position::new(
            (chunk.position.x+1) * CHUNK_SIZE as i64 - 1,
            (chunk.position.y+1) * CHUNK_SIZE as i64 - 1,
            (chunk.position.z+1) * CHUNK_SIZE as i64 - 1,
        );

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let get_block = |x, y, z| {
            &chunk.block_data[(x as usize%CHUNK_SIZE)*CHUNK_SIZE*CHUNK_SIZE+(y as usize%CHUNK_SIZE)*CHUNK_SIZE+(z as usize%CHUNK_SIZE)]
        };

        println!("x {} {}", start.x, end.x);
        println!("y {} {}", start.y, end.y);
        println!("z {} {}", start.z, end.z);

        for x in start.x..=end.x {
            for y in start.y..=end.y {
                for z in start.z..=end.z {
                    let block: &Block = get_block(x, y, z);

                    if let MeshType::Cube {top, bottom, left, right, front, back} = &block.mesh {
                        let mut faces = 0;

                        // if if (1st: checks chunk border) {true} else {2nd: checks for nearby transparent block}
                        if if start.x == x {true} else {get_block(x-1, y, z).state.transparent && !block.state.transparent} {  // left face
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,0.0+y as f32,1.0+z as f32], ind: left.0, txtr: [0, 1]});
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,1.0+y as f32,1.0+z as f32], ind: left.0, txtr: [0, 0]});
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,1.0+y as f32,0.0+z as f32], ind: left.0, txtr: [1, 0]});
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,0.0+y as f32,0.0+z as f32], ind: left.0, txtr: [1, 1]});
                            faces += 1;
                        }
                        if if start.y == y {true} else {get_block(x, y-1, z).state.transparent && !block.state.transparent} {  // bottom face
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,0.0+y as f32,0.0+z as f32], ind: bottom.0, txtr: [0, 0]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,0.0+y as f32,0.0+z as f32], ind: bottom.0, txtr: [1, 0]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,0.0+y as f32,1.0+z as f32], ind: bottom.0, txtr: [1, 1]});
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,0.0+y as f32,1.0+z as f32], ind: bottom.0, txtr: [0, 1]});
                            faces += 1;
                        }
                        if if start.z == z {true} else {get_block(x, y, z-1).state.transparent && !block.state.transparent} {  // front face
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,1.0+y as f32,0.0+z as f32], ind: front.0, txtr: [1, 1]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,1.0+y as f32,0.0+z as f32], ind: front.0, txtr: [0, 1]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,0.0+y as f32,0.0+z as f32], ind: front.0, txtr: [0, 0]});
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,0.0+y as f32,0.0+z as f32], ind: front.0, txtr: [1, 0]});
                            faces += 1;
                        }
                        if if end.x == x {true} else {get_block(x+1, y, z).state.transparent && !block.state.transparent} {  // right face
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,0.0+y as f32,0.0+z as f32], ind: right.0, txtr: [0, 0]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,1.0+y as f32,0.0+z as f32], ind: right.0, txtr: [0, 1]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,1.0+y as f32,1.0+z as f32], ind: right.0, txtr: [1, 1]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,0.0+y as f32,1.0+z as f32], ind: right.0, txtr: [1, 0]});
                            faces += 1;
                        }
                        if if end.y == y {true} else {get_block(x, y+1, z).state.transparent && !block.state.transparent} {  // top face
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,1.0+y as f32,1.0+z as f32], ind: top.0, txtr: [0, 0]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,1.0+y as f32,1.0+z as f32], ind: top.0, txtr: [1, 0]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,1.0+y as f32,0.0+z as f32], ind: top.0, txtr: [1, 1]});
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,1.0+y as f32,0.0+z as f32], ind: top.0, txtr: [0, 1]});
                            faces += 1;
                        }
                        if if end.z == z {true} else {get_block(x, y, z+1).state.transparent && !block.state.transparent} {  // back face
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,0.0+y as f32,1.0+z as f32], ind: back.0, txtr: [0, 0]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,0.0+y as f32,1.0+z as f32], ind: back.0, txtr: [1, 0]});
                            vertices.push(Self::Vertex { pos: [1.0+x as f32,1.0+y as f32,1.0+z as f32], ind: back.0, txtr: [1, 1]});
                            vertices.push(Self::Vertex { pos: [0.0+x as f32,1.0+y as f32,1.0+z as f32], ind: back.0, txtr: [0, 1]});
                            faces += 1;
                        }

                        for _ in 0..faces {
                            if indices.is_empty() {
                                indices.append(
                                    &mut vec![
                                        0, 1, 2,
                                        0, 2, 3,
                                    ]
                                )
                            } else {
                                let ofs = *indices.last().unwrap() as u32+1;  // offset
                                indices.append(
                                    &mut vec![
                                        0+ofs, 1+ofs, 2+ofs,  // triangle 1
                                        0+ofs, 2+ofs, 3+ofs,  // triangle 2
                                    ]
                                )
                            }
                        }
                    }
                }
            }
        }

        self.chunks.push((chunk, vertices, indices));
    }

    // TODO: Will probably be removed in future
    // re-evaluates the vertex and index data buffer
    fn updt_chunk(&mut self, id: &ChunkID) {
        unimplemented!()
    }

    // removes the buffer and its reference
    fn remv_chunk(&mut self, id: &ChunkID) {
        unimplemented!()
    }

    fn updt_world(&mut self, dimensions: Dimension<u32>, player: &Player) {
        let (proj, view, world) = player.camera.gen_mvp(dimensions);

        self.persp_buf = Some(self.persp_mat.next(
            cube_vs::ty::MVP {proj: proj, view: view, world: world}
        ).unwrap());
    }

    // renders the buffers and pipeline; only merges the vertex and index data into a one large buffer
    fn render<'b>(&mut self,
                  device: Arc<Device>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  dimensions: Dimension<u32>,
                  rerender: bool,
                  chunk_status: ChunkUpdate,
    ) -> (
            Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
            DynamicState,  // dynamic state for display
            Arc<CpuAccessibleBuffer<[Self::Vertex]>>,   // vertex buffer
            Arc<CpuAccessibleBuffer<[Self::Index]>>,  // index buffer
            Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>,   // sets (aka uniforms) buffer
            Self::PushConstants,   // push-down constants
    ) {
        if rerender {
            self.grph_pipe = Self::pipeline(
                &self.vert_shd, &self.frag_shd,
                device.clone(), renderpass.clone(), dimensions
            );
        }

        if  self.vertices.is_empty() ||
            self.indices.is_empty() ||
            (chunk_status & ChunkUpdate::BlockUpdate == ChunkUpdate::BlockUpdate)
        {
            self.vertices.clear();
            self.indices.clear();

            for (chunk, vertices, _indices) in self.chunks.iter() {
                if chunk.visible {  // check if the chunk is visible to be loaded (using frustum culling)
                    self.vertices.extend(vertices.iter());
                }
            }

            for (chunk, _vertices, indices) in self.chunks.iter() {
                if chunk.visible {
                    if self.indices.is_empty() {
                        self.indices.extend(
                            indices.iter()
                        );
                    } else {
                        // last number of the index is always that largest
                        // IF using the format: 0 1 2 0 2 3
                        let indx_max = self.indices.last().unwrap().clone();
                        self.indices.extend(
                            indices.iter().map(|&x| x+indx_max+1)
                        );
                    }
                }
            }

            println!("Vertex length: {:?}", self.vertices.len());
            println!("Index length: {:?}", self.indices.len());
        }

        // TODO: Automaticaly add texture buffers to sets
        let layout0 = self.grph_pipe.descriptor_set_layout(0).unwrap();
        let set0 = Arc::new(PersistentDescriptorSet::start(layout0.clone())
            .enter_array().unwrap()
            .add_sampled_image(self.textures[0].clone(), self.sampler.clone()).unwrap()
            .add_sampled_image(self.textures[1].clone(), self.sampler.clone()).unwrap()
            .add_sampled_image(self.textures[2].clone(), self.sampler.clone()).unwrap()
            .add_sampled_image(self.textures[3].clone(), self.sampler.clone()).unwrap()
            .add_sampled_image(self.textures[4].clone(), self.sampler.clone()).unwrap()
            .add_sampled_image(self.textures[5].clone(), self.sampler.clone()).unwrap()
            .add_sampled_image(self.textures[6].clone(), self.sampler.clone()).unwrap()
            .add_sampled_image(self.textures[7].clone(), self.sampler.clone()).unwrap()
            .add_sampled_image(self.textures[8].clone(), self.sampler.clone()).unwrap()
            .leave_array().unwrap()
            .build().unwrap()
        );

        let layout1 = self.grph_pipe.descriptor_set_layout(1).unwrap();
        let set1 = Arc::new(PersistentDescriptorSet::start(layout1.clone())
            .add_buffer(self.persp_buf.as_ref().unwrap().clone()).unwrap()
            .build().unwrap()
        );

        (
            self.grph_pipe.clone(),
            DynamicState::none(),
            CpuAccessibleBuffer::from_iter(
                device.clone(), BufferUsage::vertex_buffer(),
                false, self.vertices.clone().into_iter()
            ).unwrap(),
            CpuAccessibleBuffer::from_iter(
                device.clone(), BufferUsage::index_buffer(),
                false, self.indices.clone().into_iter()
            ).unwrap(),
            vec![set0, set1],
            (),
        )
    }
}
