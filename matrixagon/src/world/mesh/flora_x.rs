use crate::datatype::{Dimension, Position, ChunkUnit, BlockUnit};
use crate::world::mesh::{Mesh, MeshType, MeshDataTypeFull};
use crate::world::chunk::Chunk;
use crate::world::shader::{FloraVert, flora_vs, flora_fs};
use crate::world::ChunkID;
use crate::world::block::Block;
use crate::world::chunk_threadpool::{ChunkThreadPool, ThreadPoolOutput};
use crate::world::player::camera::Camera;

use vulkano::pipeline::viewport::Viewport;
use vulkano::framebuffer::{Subpass, RenderPassAbstract};
use vulkano::device::Device;
use vulkano::pipeline::{GraphicsPipelineAbstract, GraphicsPipeline};
use vulkano::command_buffer::DynamicState;
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage, CpuBufferPool};
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};
use vulkano::descriptor::DescriptorSet;
use vulkano::buffer::cpu_pool::{CpuBufferPoolSubbuffer, CpuBufferPoolChunk};
use vulkano::memory::pool::StdMemoryPool;
use vulkano::image::ImmutableImage;
use vulkano::format::Format;

use rayon::prelude::*;

use std::sync::Arc;
use std::iter;


pub enum Side {
    Positive,  // facing (+x,+y)
    Negative,  // facing (-x,-y)
}

// flora mesh is basically two diagonal textures corssing each other
pub struct FloraX {
    textures: Arc<ImmutableImage<Format>>,
    // chunks: Chunk Reference, Chunk Cullling, Chunk Vertices, Chunk Indices
    chunks: Vec<(ChunkID, bool, Vec<<Self as Mesh>::Vertex>, Vec<<Self as Mesh>::Index>)>,
    grph_pipe: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    dimensions: Dimension<u32>,

    vert_shd: flora_vs::Shader,
    frag_shd: flora_fs::Shader,
    vrtx_buf: CpuBufferPool<<Self as Mesh>::Vertex>,
    indx_buf: CpuBufferPool<<Self as Mesh>::Index>,

    persp_mat: CpuBufferPool<flora_vs::ty::MVP>,
    persp_buf: Option<CpuBufferPoolSubbuffer<flora_vs::ty::MVP, Arc<StdMemoryPool>>>,
    sampler: Arc<Sampler>,

    vertices: Vec<<Self as Mesh>::Vertex>,  // aggregated vertices (stored to optimize)
    indices: Vec<<Self as Mesh>::Index>,  // aggregated indices (stored to optimize)
}

impl FloraX {
    pub fn new(device: Arc<Device>,
               texture: Arc<ImmutableImage<Format>>,
               render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
               dimensions: Dimension<u32>,
               cam: &Camera,
    ) -> Self {
        let vs = flora_vs::Shader::load(device.clone()).expect("failed to create flora vertex shaders module");
        let fs = flora_fs::Shader::load(device.clone()).expect("failed to create flora fragment shaders module");

        let mut s = Self {
            textures: texture.clone(),
            chunks: Vec::new(),
            grph_pipe: FloraX::pipeline(
                &vs, &fs, device.clone(),
                render_pass.clone(), dimensions.into(),
            ),
            dimensions: dimensions,

            vert_shd: vs,
            frag_shd: fs,
            vrtx_buf: CpuBufferPool::new(device.clone(), BufferUsage {
                transfer_destination: true,
                vertex_buffer: true,
                ..BufferUsage::none()
            }),
            indx_buf: CpuBufferPool::new(device.clone(), BufferUsage {
                transfer_destination: true,
                index_buffer: true,
                ..BufferUsage::none()
            }),

            persp_mat: CpuBufferPool::uniform_buffer(device.clone()),
            persp_buf: None,
            sampler: Sampler::new(device.clone(), Filter::Nearest, Filter::Nearest,
                                  MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
                                  SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 8.0).unwrap(),

            vertices: Vec::new(),
            indices: Vec::new(),
        };
        println!("FX-VB Resv: {:?}", s.vrtx_buf.capacity());
        println!("FX-IB Resv: {:?}", s.vrtx_buf.capacity());
        s.vrtx_buf.reserve(4);
        s.indx_buf.reserve(4);
        println!("FX-VB Resv Aft: {:?}", s.vrtx_buf.capacity());
        println!("FX-IB Resv Aft: {:?}", s.vrtx_buf.capacity());
        s.updt_world(Some(dimensions), Some(cam));
        s
    }

    // internal function for building pipeline
    fn pipeline(
        vert: &flora_vs::Shader,
        frag: &flora_fs::Shader,
        device: Arc<Device>,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dimensions: Dimension<u32> )
        -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {

        // note: flora-x must not need to be culled because it will be viewed on both sides
        Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<<Self as Mesh>::Vertex>()
            .vertex_shader(vert.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                dimensions: dimensions.into(),
                depth_range: 0.0 .. 1.0,
            }))
            .fragment_shader(frag.main_entry_point(), ())
            .alpha_to_coverage_enabled()  // to enable transparency
            .depth_stencil_simple_depth()  // to enable depth buffering
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone()).unwrap()
        )
    }

    fn mesh_data(chunk_list: Vec<(ChunkID, bool, Vec<FloraVert>, Vec<u32>)>, chunks: Arc<Vec<Chunk>>, chunk: Chunk) -> ThreadPoolOutput {
        let find_chunk = |cid: ChunkID| -> Option<&Chunk> {
            // don't parallelize this iterator: as this closure gets executed few thousands time,
            // the overhead of even threadpool can largely affect negatively
            if let Some(ind) = chunks.iter().position(|x| x.id == cid) {
                Some(&chunks[ind])
            } else {
                None
            }
        };

        let start = Position::new(
            chunk.position.x.into_block(),
            chunk.position.y.into_block(),
            chunk.position.z.into_block(),
        );

        let end = Position::new(
            (chunk.position.x + ChunkUnit(1.0)).into_block() - BlockUnit(1.0),
            (chunk.position.y + ChunkUnit(1.0)).into_block() - BlockUnit(1.0),
            (chunk.position.z + ChunkUnit(1.0)).into_block() - BlockUnit(1.0),
        );

        // println!("Chunk Start: {:?}", start);
        // println!("Chunk End: {:?}", end);

        // Chunk Border Culling:
        // first locate any adjacent chunks in the world.mesh
        // then locate the world.block data to find any transparent blocks
        //  ^- IF NOT: do not add the vertices and indices
        //  ^- ELSE: add the vertices and indices to the vector
        //  ^- CASE: when there are no adjacent chunks: do not add the vertices and indices

        // listing out all the theoretical adjacent chunks this Chunk has
        let adjc_chunks: [Position<ChunkUnit>; 6] = [
            Position::new(chunk.position.x+ChunkUnit(1.0), chunk.position.y  , chunk.position.z  ),  // LEFT
            Position::new(chunk.position.x-ChunkUnit(1.0), chunk.position.y  , chunk.position.z  ),  // RIGHT
            Position::new(chunk.position.x  , chunk.position.y+ChunkUnit(1.0), chunk.position.z  ),  // UP
            Position::new(chunk.position.x  , chunk.position.y-ChunkUnit(1.0), chunk.position.z  ),  // DOWN
            Position::new(chunk.position.x  , chunk.position.y  , chunk.position.z+ChunkUnit(1.0)),  // BACK
            Position::new(chunk.position.x  , chunk.position.y  , chunk.position.z-ChunkUnit(1.0)),  // FRONT
        ];

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // NOTE: FOR THE HECK SAKE, the bug was I have to change the u32 to i32 because of negative position. sigh.
        for x in start.x.into()..=i32::from(end.x) {
            let x = BlockUnit(x as f32);
            for y in start.y.into()..=i32::from(end.y) {
                let y = BlockUnit(y as f32);
                for z in start.z.into()..=i32::from(end.z) {
                    let z = BlockUnit(z as f32);
                    let block: &Block = chunk.blocks(x, y, z);

                    /*
                        1 -- 3
                        | \  |
                        |  \ |
                        0 -- 2
                     */

                    if let MeshType::FloraX {positive, negative} = &block.mesh {
                        // positive face
                        /*
                        |\--|
                        | \ |
                        |__\|
                         */
                        vertices.push(FloraVert { pos: [ 1.1+x.inner(),-0.1+y.inner(), 0.0+z.inner()], txtr: 1 | (positive.0 << 16)});
                        vertices.push(FloraVert { pos: [ 1.1+x.inner(), 1.1+y.inner(), 0.0+z.inner()], txtr: 0 | (positive.0 << 16)});
                        vertices.push(FloraVert { pos: [ 0.0+x.inner(), 1.1+y.inner(), 1.1+z.inner()], txtr: 2 | (positive.0 << 16)});
                        vertices.push(FloraVert { pos: [ 0.0+x.inner(),-0.1+y.inner(), 1.1+z.inner()], txtr: 3 | (positive.0 << 16)});

                        // negative face
                        /*
                        |--/|
                        | / |
                        |/__|
                         */
                        vertices.push(FloraVert { pos: [ 0.0+x.inner(),-0.1+y.inner(), 0.0+z.inner()], txtr: 1 | (negative.0 << 16)});
                        vertices.push(FloraVert { pos: [ 0.0+x.inner(), 1.1+y.inner(), 0.0+z.inner()], txtr: 0 | (negative.0 << 16)});
                        vertices.push(FloraVert { pos: [ 1.1+x.inner(), 1.1+y.inner(), 1.1+z.inner()], txtr: 2 | (negative.0 << 16)});
                        vertices.push(FloraVert { pos: [ 1.1+x.inner(),-0.1+y.inner(), 1.1+z.inner()], txtr: 3 | (negative.0 << 16)});

                        if indices.is_empty() {
                            indices.append(
                                &mut vec![
                                    // first shape
                                    0, 1, 2,  // 1st triangle
                                    0, 2, 3,  // 2nd triangle
                                    // second shape
                                    4, 5, 6,
                                    4, 6, 7,
                                ]
                            );
                        } else {
                            let ofs = *indices.last().unwrap() as u32+1;  // offset
                            indices.append(
                                &mut vec![
                                    // first shape
                                    0+ofs, 1+ofs, 2+ofs,  // triangle 1
                                    0+ofs, 2+ofs, 3+ofs,  // triangle 2
                                    // second shape
                                    4+ofs, 5+ofs, 6+ofs,
                                    4+ofs, 6+ofs, 7+ofs,
                                ]
                            );
                        }
                    }
                }
            }
        }

        (Box::new(vertices), Box::new(indices))
    }
}

impl Mesh for FloraX {
    type Vertex = FloraVert;
    type Index = u32;

    type PushConstants = ();

    fn add_chunk(&mut self, chunk_id: ChunkID) {
        // ( chunk reference, vertices vector, indices vector )
        self.chunks.push((chunk_id, false, Vec::new(), Vec::new()));
    }

    fn load_chunks(&mut self, chunks: Vec<Chunk>, pool: &mut ChunkThreadPool) {
        let mut chunk_cloned = self.chunks.clone();
        let chunks = Arc::new(chunks);

        for (chunk_id, _cull, _vert, _indx) in chunk_cloned.iter_mut() {
            // println!("Chunk loop");
            // println!("Chunks Arc Ref: {}", Arc::strong_count(&chunks));

            let cloned_chunks = chunks.clone();

            let find_chunk = |cid: ChunkID| -> Option<Chunk> {
                if let Some(ind) = cloned_chunks.par_iter().position_first(|x| x.id == cid) {
                    Some(cloned_chunks[ind].clone())
                } else {
                    None
                }
            };

            if let Some(chunk) = find_chunk(*chunk_id) {
                let chunk_list = self.chunks.clone();
                let chunks = chunks.clone();

                // println!("Adding a chunk thread");
                let chunk = chunk.clone();
                pool.add_work( ( chunk_id.clone(), Box::new(move || {
                    Self::mesh_data(chunk_list, chunks, chunk)
                })));  // end for adding work to the thread pool
            }
        }

        let output = pool.join();

        for (id, (mut vert, mut indx)) in output {
            // as long the self.chunks doesn't get changed in between, it should never panic
            let ind = self.chunks.iter().position(|c| c.0 == id).unwrap();

            let mut vertices: &mut Vec<FloraVert> = (*vert).downcast_mut().unwrap();
            let mut indices: &mut Vec<u32> = (*indx).downcast_mut().unwrap();

            // .2: vertex dt of that chunk; .3 index dt of that chunk

            // just in case if there are any vertices/indices data this chunk has previously
            // which can cause some rendering issues
            self.chunks[ind].2.clear();
            self.chunks[ind].3.clear();

            self.chunks[ind].2.append(&mut vertices);
            self.chunks[ind].3.append(&mut indices);
        }


    }

    // TODO: Will probably be removed in future
    // re-evaluates the vertex and index data buffer
    fn updt_chunks(&mut self, id: ChunkID) {
        unimplemented!()
    }

    // removes the buffer and its reference
    fn remv_chunk(&mut self, id: ChunkID) {
        for ind in 0..self.chunks.len() {
            if self.chunks[ind].0 == id {
                self.chunks.swap_remove(ind);
                break;
            }
        }
    }

    fn updt_world(&mut self, dimensions: Option<Dimension<u32>>, cam: Option<&Camera>) {
        if let Some(new_dimn) = dimensions {
            self.dimensions = new_dimn;

            if let Some(new_cam) = cam {
                let (proj, view, world) = new_cam.gen_mvp(self.dimensions);

                self.persp_buf = Some(self.persp_mat.next(
                    flora_vs::ty::MVP {proj: proj, view: view, world: world}
                ).unwrap());
            }
        }
    }

    // renders the buffers and pipeline; only merges the vertex and index data into a one large buffer
    // called for each frame
    fn render<'b>(&mut self,
                  device: Arc<Device>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  rerender: bool,
                  reload_chunk: bool,
    ) -> MeshDataTypeFull<Self::Vertex, Self::Index, Self::PushConstants>
       // (
       //     Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
       //     DynamicState,  // dynamic state for display
       //     CpuBufferPoolChunk<Self::Vertex, Arc<StdMemoryPool>>,   // vertex buffer
       //     CpuBufferPoolChunk<Self::Index, Arc<StdMemoryPool>>,  // index buffer
       //     Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>,   // sets (aka uniforms) buffer
       //     Self::PushConstants,   // push-down constants
       // )
    //                    (
    //     Arc<dyn GraphicsPipelineAbstract + Send + Sync>,  // graphic pipeline
    //     DynamicState,  // dynamic state for display
    //     Arc<CpuAccessibleBuffer<[Self::Vertex]>>,   // vertex buffer
    //     Arc<CpuAccessibleBuffer<[Self::Index]>>,  // index buffer
    //     Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>,   // sets (aka uniforms) buffer
    //     Self::PushConstants,   // push-down constants
    // )
    {
        println!("vvvvvvvvvvvvvvvv P");

        if rerender {
            self.grph_pipe = Self::pipeline(
                &self.vert_shd, &self.frag_shd,
                device.clone(), renderpass.clone(), self.dimensions
            );
        }

        if self.vertices.is_empty() || self.indices.is_empty() || reload_chunk {
            self.vertices.clear();
            self.indices.clear();

            for (_chunk, cull, vertices, _indices) in self.chunks.iter() {
                if !*cull {  // check if the chunk is visible to be loaded (using frustum culling)
                    self.vertices.extend(vertices.iter());
                }
            }

            for (_chunk, cull, _vertices, indices) in self.chunks.iter() {
                if !*cull {
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

            // println!("Vertex length: {:?}", self.vertices.len());
            // println!("Index length: {:?}", self.indices.len());
        }

        // TODO: Dynamically add new texture buffers
        let layout0 = self.grph_pipe.descriptor_set_layout(0).unwrap();
        let set0 = Arc::new(PersistentDescriptorSet::start(layout0.clone())
            .add_sampled_image(self.textures.clone(), self.sampler.clone()).unwrap()
            .build().unwrap()
        );

        let layout1 = self.grph_pipe.descriptor_set_layout(1).unwrap();
        let set1 = Arc::new(PersistentDescriptorSet::start(layout1.clone())
            .add_buffer(self.persp_buf.as_ref().unwrap().clone()).unwrap()
            .build().unwrap()
        );

        println!("^^^^^^^^^^^^^^^^ P");
        let vrtx_sb = self.vrtx_buf.chunk(self.vertices.clone()).unwrap();
        let indx_sb = self.indx_buf.chunk(self.indices.clone()).unwrap();


        (
            self.grph_pipe.clone(),
            DynamicState::none(),
            vrtx_sb,
            indx_sb,
            vec![set0, set1],
            (),
        )

        // (
        //     self.grph_pipe.clone(),
        //     DynamicState::none(),
        //     CpuAccessibleBuffer::from_iter(
        //         device.clone(), BufferUsage::vertex_buffer(),
        //         false, self.vertices.clone().into_iter()
        //     ).unwrap(),
        //     CpuAccessibleBuffer::from_iter(
        //         device.clone(), BufferUsage::index_buffer(),
        //         false, self.indices.clone().into_iter()
        //     ).unwrap(),
        //     vec![set0, set1],
        //     (),
        // )
    }
}
