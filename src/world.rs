use crate::player::Player;
use crate::terrain::Terrain;
use crate::mesh::{Meshes, MeshesExt};
use crate::chunk::{Chunk, ChunkError};
use crate::datatype::{Position, Dimension};
use crate::texture::Texture;

use vulkano::device::{Queue, Device};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder};
use vulkano::framebuffer::{RenderPassAbstract, FramebufferAbstract};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;

use std::sync::Arc;
use vulkano::sync::GpuFuture;
use std::rc::Rc;


pub struct ChunkID(u32);

pub struct World<'c> {
    // world entities
    pub player: Player,
    terrain: Terrain,
    meshes: Meshes<'c>,
    chunks: Vec<Rc<Chunk>>,  // TODO: Rc can affect mass chunk loading performance

    // weather:

    // world structure and manager
    texture: Texture,
    chunk_counter: u32,  // chunk ID counter
}

impl<'c> World<'c> {
    pub fn new(
        device: Arc<Device>,
        mut texture: Texture,
        renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dimensions: Dimension<u32>
    ) -> Self {  // creates a new world
        println!("WORLD - INITIALIZED");

        texture.add(include_bytes!("../resource/texture/blocks/test.png").to_vec(), "test_old");
        texture.add(include_bytes!("../resource/texture/blocks/test2.png").to_vec(), "test");

        Self {
            player: Player::new(),
            terrain: Terrain::new(&texture),
            meshes: Meshes::new(device.clone(), &texture, renderpass.clone(), dimensions.clone()),
            chunks: Vec::new(),

            texture: texture,
            chunk_counter: 0,
        }
    }

    pub fn bind_texture(
        &mut self,
        mut gpu_future: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        let futures = self.texture.futures();

        for fut in futures.into_iter() {
            gpu_future = Box::new(gpu_future.join(fut)) as Box<dyn GpuFuture>;
        }
        gpu_future
    }

    // update function on SEPARATE UPDATE THREAD
    pub fn update(&mut self) {
        println!("WORLD - UPDATE");
        println!("Player {:?}", self.player);

        let position = Position::new(0, 0, 0);
        if let Ok(id) = self.chunk_id(position) {
            let chunk1 = Rc::new(Chunk::new(id, position, self.terrain.generate_chunk(position)));
            self.chunks.push(chunk1.clone());
            self.meshes.load_chunk(chunk1.clone())
        }
    }

    // returns command pipeline
    pub fn render(&mut self,
                  device: Arc<Device>,
                  queue: Arc<Queue>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
                  dimensions: Dimension<u32>,
                  rerender: bool,
    ) -> AutoCommandBuffer<StandardCommandPoolAlloc> {
        println!("WORLD - RENDER");

        // the update function must be before the render call
        // this function call will be on the separate thread
        self.meshes.update(dimensions, &self.player);

        let mesh_data = self.meshes.render(device.clone(), renderpass.clone(), dimensions, rerender);

        AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            .begin_render_pass(framebuffer.clone(), false, vec![[0.1, 0.3, 1.0, 1.0].into(), 1f32.into()]).unwrap()
            .draw_mesh(mesh_data[0].clone()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap()
    }

    // deterministic chunk id based off chunk position
    fn chunk_id(&mut self, position: Position<u32>) -> Result<ChunkID, ChunkError> {
        self.chunk_counter += 1;

        // checking if *all* of the chunks have *different* positions; no duplicate position
        if self.chunks.iter().all(|x| x.position != position) {
            Ok(ChunkID(self.chunk_counter))
        } else {
            Err(ChunkError::DuplicateChunkPos)
        }
    }
}
