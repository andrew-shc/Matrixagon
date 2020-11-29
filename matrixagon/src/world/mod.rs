use crate::world::player::Player;
use crate::world::terrain::Terrain;
use crate::world::mesh::{Meshes, MeshesExt, MeshesDataType};
use crate::datatype::Dimension;
use crate::world::texture::Texture;
use crate::world::chunk_handler::{ChunkHandler, ChunkStatusInfo};
use crate::world::block::registry::BlockRegistry;
use crate::event::{EventDispatcher, EventName};
use crate::world::commands::WorldCommandExecutor;
use crate::world::player::camera::Camera;

use vulkano::device::{Queue, Device};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture};
use vulkano::framebuffer::{RenderPassAbstract, FramebufferAbstract};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::sync::{GpuFuture, NowFuture};

use std::sync::Arc;
use std::mem;
use std::rc::Rc;

pub mod mesh;
pub mod terrain;
pub mod player;
pub mod block;
pub mod commands;

pub mod shader;
pub mod chunk;
pub mod chunk_handler;
pub mod texture;
pub mod chunk_threadpool;


#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ChunkID(pub u32);

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ChunkUpdateState {
    Consistent,  // the result is similar enough to not needed to be update
    Update,  // the result is different enough that it needs to be updated
    Immediate,  // the result must immediately send to chunk handler to be updated
}

// World State Update struct
// This strut is used to format the required states from the world to generate a World Mesh Data
#[derive(Clone)]
pub struct WorldStateUpd {
    pub cam: Camera,
    pub dimensions: Dimension<u32>,

    pub rerender: bool,
    pub renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,

    pub registry: Arc<BlockRegistry>,
}

impl WorldStateUpd {
    fn from_world(camera: Camera, registry: Arc<BlockRegistry>, dimn: Dimension<u32>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
                  rerender: bool,
    ) -> Self {
        Self {
            cam: camera,
            dimensions: dimn,

            rerender: rerender,
            renderpass: renderpass.clone(),
            framebuffer: framebuffer.clone(),

            registry: registry.clone(),
        }
    }

    // returns a bool on whether it should discards the new state update if most of the field remains
    // same as the state passed in through the argument
    fn update(&self, state: &WorldStateUpd) -> ChunkUpdateState {
        if self.dimensions != state.dimensions {
            ChunkUpdateState::Immediate
        } else if
            self.cam != state.cam ||
            self.dimensions != state.dimensions ||
            self.rerender != state.rerender {
            ChunkUpdateState::Update
        } else {
            ChunkUpdateState::Consistent
        }
    }
}

pub struct World {
    // world entities/components
    // TODO: This can privatized once the world events has been fully added
    pub player: Player,
    command: WorldCommandExecutor,

    // world structure and manager
    event: Rc<EventDispatcher>,
    registry: Arc<BlockRegistry>,  // a globalized way to hold all in-game block instance
    texture: Texture,
    texture_fut: Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,

    // multithreading
    world_state: Option<WorldStateUpd>,
    render_buffer: Option<MeshesDataType>,  // render data single buffer
    chunk_status_buffer: Option<ChunkStatusInfo>,  // chunk status info from Chunk Thread single buffer

    temp_chunkhandler: ChunkHandler,
}

impl World {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        evd: Rc<EventDispatcher>,
        renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
        dimensions: Dimension<u32>
    ) -> Self {  // creates a new world
        println!("WORLD - INITIALIZED");

        let mut texture = Texture::new(queue.clone());

        texture.add_texture("resource/texture/blocks/air.png", "air");
        texture.add_texture("resource/texture/blocks/grass_side.png", "grass_side");
        texture.add_texture("resource/texture/blocks/grass_top.png", "grass_top");
        texture.add_texture("resource/texture/blocks/dirt.png", "dirt");
        texture.add_texture("resource/texture/blocks/sand.png", "sand");
        texture.add_texture("resource/texture/blocks/stone.png", "stone");
        texture.add_texture("resource/texture/blocks/grass_flora.png", "grass_flora");
        texture.add_texture("resource/texture/blocks/flower.png", "flower");

        let (txtr_dt, txtr_future) = texture.texture_future();
        let block_registry = Arc::new(BlockRegistry::new(&texture));

        let player = Player::new();

        // TODO: Use global work threads instead
        // chunk handler will create a new separate chunk threadpools
        // we only just need the channels
        let temp_chunkhandler = ChunkHandler::new(
            device.clone(), queue.clone(), evd.clone(),
            Meshes::new(device.clone(), txtr_dt.clone(), renderpass.clone(), dimensions.clone(), &player.camera),
            Terrain::new(24, block_registry.clone()),
        );

        let mut cmd = WorldCommandExecutor::new();
        cmd.load_file_bytc("resource/commands/test00.wcb".into());

        Self {
            player: player.clone(),
            command: cmd,

            event: evd.clone(),
            registry: block_registry.clone(),
            texture: texture,
            texture_fut: Some(txtr_future),

            render_buffer: None,  // render data single buffer
            chunk_status_buffer: None,  // chunk status info single buffer
            world_state: None,

            temp_chunkhandler: temp_chunkhandler,
        }
    }

    pub fn bind_texture( &mut self, gpu_future: Box<dyn GpuFuture>, ) -> Box<dyn GpuFuture> {
        let txtr_fut = mem::replace(&mut self.texture_fut, None);
        Box::new(gpu_future.join(txtr_fut.expect("Texture future has already been taken"))) as Box<dyn GpuFuture>
    }

    // update function on SEPARATE UPDATE THREAD
    pub fn update(&mut self, dimensions: Dimension<u32>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
                  rerender: bool,) {
        // println!("WORLD - UPDATE");



        if let Some(stat) = &self.chunk_status_buffer {
            if stat.chunks_loaded > 0 || stat.chunks_offloaded > 0 {
                // println!("C: {:?}, L: {:?}, O: {:?}, U: X", stat.total_chunks_loaded, stat.chunks_loaded, stat.chunks_offloaded);
            }
        }

        println!("Nooo");
        if let Some(state) = &self.world_state {
            let new_state = WorldStateUpd::from_world(self.player.camera.clone(), self.registry.clone(), dimensions, renderpass.clone(), framebuffer.clone(), rerender);
            let update_state = state.update(&new_state);
            if update_state != ChunkUpdateState::Consistent {
                println!("ChunkUpdateState no Consistent");
                // TODO: temp
                self.event.clone().emit(EventName("MeshEvent/UpdateDimensions"), event_data![dimensions]);
                let (rb, csb) = self.temp_chunkhandler.update(new_state.clone());

                self.render_buffer = Some(rb);
                self.chunk_status_buffer = Some(csb);

                self.world_state = Some(new_state);
            }
        } else {
            println!("OFC");
            let new_state = WorldStateUpd::from_world(self.player.camera.clone(), self.registry.clone(), dimensions, renderpass.clone(), framebuffer.clone(), rerender);
            let (rb, csb) = self.temp_chunkhandler.update(new_state.clone());

            self.render_buffer = Some(rb);
            self.chunk_status_buffer = Some(csb);

            self.world_state = Some(new_state);
        }
    }

    // returns command pipeline
    pub fn render(&mut self,
                  device: Arc<Device>,
                  queue: Arc<Queue>,
                  framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
                  dimensions: Dimension<u32>,
    ) -> AutoCommandBuffer<StandardCommandPoolAlloc> {
        // println!("WORLD - RENDER");

        println!(".__.");
        // the render buffer should always have value taken care by the code above
        println!("DT?: {:?}", if let Some(_) = self.render_buffer {true} else {false});
        // self.render_buffer.clone();
        let mut mesh_datas = self.render_buffer.clone().expect("Render buffer missing");
        mesh_datas.update_camera(device.clone(), &self.player.camera, dimensions);
        // self.render_buffer.unwrap().update_camera(device.clone(), &self.player.camera, dimensions);

        // destructuring instead of cloning each field out has a noticeable rendering performance improvements
        let MeshesDataType {cube, flora_x} = mesh_datas;

        // TODO: Maybe add a method for a way to recompute the descriptors without touching the vert/indx buffer

        let mut cmd_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();

        cmd_builder
            .begin_render_pass(framebuffer.clone(), false, vec![[0.1, 0.3, 1.0, 1.0].into(), 1f32.into()]).unwrap()
            .draw_mesh(cube).unwrap()
            .draw_mesh(flora_x).unwrap()
            .end_render_pass().unwrap();

        cmd_builder.build().unwrap()
    }
}
