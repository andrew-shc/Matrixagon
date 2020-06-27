use crate::player::{Player, CHUNK_RADIUS};
use crate::terrain::Terrain;
use crate::mesh::{Meshes, MeshesExt};
use crate::chunk::{Chunk, ChunkError, ChunkUpdate, CHUNK_SIZE};
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
    chunk_flags: ChunkUpdate,  // chunk flags will usually be read by update/render call; not on instance
}

impl<'c> World<'c> {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dimensions: Dimension<u32>
    ) -> Self {  // creates a new world
        println!("WORLD - INITIALIZED");

        // TODO: textures: use macro expansion to automatically add the textures AT COMPILE TIME
        // TODO: or implement somw ways to add textures at RUN TIME
        let mut texture = Texture::new(queue.clone());

        texture.add(include_bytes!("../resource/texture/blocks/test.png").to_vec(), "test_old");
        texture.add(include_bytes!("../resource/texture/blocks/test2.png").to_vec(), "test");
        texture.add(include_bytes!("../resource/texture/blocks/air.png").to_vec(), "air");

        texture.add(include_bytes!("../resource/texture/blocks/east.png").to_vec(), "east");
        texture.add(include_bytes!("../resource/texture/blocks/south.png").to_vec(), "south");
        texture.add(include_bytes!("../resource/texture/blocks/west.png").to_vec(), "west");
        texture.add(include_bytes!("../resource/texture/blocks/north.png").to_vec(), "north");
        texture.add(include_bytes!("../resource/texture/blocks/zenith.png").to_vec(), "zenith");
        texture.add(include_bytes!("../resource/texture/blocks/nadir.png").to_vec(), "nadir");

        Self {
            player: Player::new(),
            terrain: Terrain::new(&texture),
            meshes: Meshes::new(device.clone(), &texture, renderpass.clone(), dimensions.clone()),
            chunks: Vec::new(),

            texture: texture,
            chunk_counter: 0,
            chunk_flags: ChunkUpdate::default(),
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
        // println!("WORLD - UPDATE");
        // println!("Player {:?}", self.player);

        self.chunk_flags = ChunkUpdate::default();

        // player position in chunk position
        let chunk_pos: Position<i64> = Position::new(
            (self.player.camera.position.coords.data[0] % CHUNK_SIZE as f32) as i64,
            (self.player.camera.position.coords.data[1] % CHUNK_SIZE as f32) as i64,
            (self.player.camera.position.coords.data[2] % CHUNK_SIZE as f32) as i64,
        );

        let mut chunk_loaded = 0;
        for x in -(CHUNK_RADIUS as i64)..CHUNK_RADIUS as i64*2 {
            for y in -(CHUNK_RADIUS as i64)..CHUNK_RADIUS as i64*2 {
                for z in -(CHUNK_RADIUS as i64)..CHUNK_RADIUS as i64*2 {
                    if self.load_chunk(Position::new(
                        (chunk_pos.x+x),
                        (chunk_pos.y+y),
                        (chunk_pos.z+z),
                    )) {
                        chunk_loaded += 1;
                    }
                }
            }
        }

        println!("CHUNK LOADED: {:?}", chunk_loaded);

        if chunk_loaded > 0 {
            self.chunk_flags = self.chunk_flags | ChunkUpdate::BlockUpdate;
        }

        // let position = Position::new(0, 0, 0);
        // self.load_chunk(position);
    }

    // TODO: later we need to somehow load chunk from files
    // loads chunk to the world
    pub fn load_chunk(&mut self, chunk_pos: Position<i64>) -> bool {
        self.new_chunk(chunk_pos)
    }

    // offloads the chunk to either save or discard
    pub fn offload_chunk(&mut self, id: ChunkID) {

    }

    // TODO: might need to change chunk position to i128, to support "infinite" terrain generation
    // specifically to create a new chunk internally
    fn new_chunk(&mut self, chunk_pos: Position<i64>) -> bool {
        if let Ok(id) = self.chunk_id(chunk_pos) {
            let chunk1 = Rc::new(Chunk::new(id, chunk_pos, self.terrain.generate_chunk(chunk_pos)));
            self.chunks.push(chunk1.clone());
            self.meshes.load_chunk(chunk1.clone());
            true
        } else {
            false
        }
    }

    // specifically to read chunk from world files internally
    fn read_chunk(&mut self) {

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
        // println!("WORLD - RENDER");

        // the update function must be before the render call
        // this function call will be on the separate thread
        self.meshes.update(dimensions, &self.player);

        let mesh_data = self.meshes.render(device.clone(), renderpass.clone(), dimensions, rerender, self.chunk_flags);

        let mut cmd_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();

        cmd_builder
            .begin_render_pass(framebuffer.clone(), false, vec![[0.1, 0.3, 1.0, 1.0].into(), 1f32.into()]).unwrap()
            .draw_mesh(mesh_data[0].clone()).unwrap()
            .end_render_pass().unwrap();

        cmd_builder.build().unwrap()
    }

    // deterministic chunk id based off chunk position
    fn chunk_id(&mut self, position: Position<i64>) -> Result<ChunkID, ChunkError> {
        self.chunk_counter += 1;

        // checking if *all* of the chunks have *different* positions; no duplicate position
        if self.chunks.iter().all(|x| x.position != position) {
            Ok(ChunkID(self.chunk_counter))
        } else {
            Err(ChunkError::DuplicateChunkPos)
        }
    }
}
