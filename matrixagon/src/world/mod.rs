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


use crate::world::player::Player;
use crate::world::terrain::Terrain;
use crate::world::mesh::{Meshes, MeshesExt, MeshesDataType};
use crate::datatype::Dimension;
use crate::world::texture::Texture;
use crate::world::chunk_handler::{ChunkHandler, ChunkStatusInfo};
use crate::event::types::{ChunkEvents, WorldEvents};
use crate::world::block::registry::BlockRegistry;
use crate::event::EventQueue;
use crate::world::commands::WorldCommandExecutor;

use vulkano::device::{Queue, Device};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferExecFuture};
use vulkano::framebuffer::{RenderPassAbstract, FramebufferAbstract};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::sync::{GpuFuture, NowFuture};

use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time;
use std::mem;


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
    pub player: Player,
    pub dimensions: Dimension<u32>,

    pub rerender: bool,
    pub renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,

    pub registry: Arc<BlockRegistry>,
}

impl WorldStateUpd {
    fn from_world(player: Player, registry: Arc<BlockRegistry>, dimn: Dimension<u32>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
                  rerender: bool,
    ) -> Self {
        Self {
            player: player,
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
            self.player != state.player ||
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
    pub event: EventQueue<WorldEvents>,
    registry: Arc<BlockRegistry>,  // a globalized way to hold all in-game block instance
    texture: Texture,
    texture_fut: Option<CommandBufferExecFuture<NowFuture, AutoCommandBuffer>>,
    player_ir: Player,  // player intermediate updates only 1 second instead of each frame instantaneously
    player_ir_signal: mpsc::Receiver<bool>,  // receives a signal whether to update the player state (not the player state for render)

    // multithreading
    world_state: Option<WorldStateUpd>,
    render_buffer: Option<MeshesDataType<'static>>,  // render data single buffer
    chunk_status_buffer: Option<ChunkStatusInfo>,  // chunk status info from Chunk Thread single buffer
    chunk_chan_inp_tx: mpsc::Sender<chunk_handler::ThreadInput>,  // chunk thread input sending channel
    chunk_chan_out_rx: mpsc::Receiver<chunk_handler::ThreadOutput<'static>>,  // chunk thread output receiving channel
}

impl World {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
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

        let (inp_tx, inp_rx) = mpsc::channel();  // new chunk events/world state -> chunk handler channel
        let (out_tx, out_rx) = mpsc::channel();  // chunk handler -> render data/chunk statuses channel

        ChunkHandler::new(
            device.clone(), queue.clone(),
            inp_rx, out_tx, inp_tx.clone(),
            Meshes::new(device.clone(), txtr_dt.clone(), renderpass.clone(), dimensions.clone()),
            Terrain::new(24)
        ).instantiate();

        let player = Player::new();

        let (ply_sgn_tx, ply_sgn_rx) = mpsc::channel();

        // a simple async timer for a time to update player state to not spam the channel to the chunk handler
        thread::spawn(move || {
            loop {
                if let Ok(_) = ply_sgn_tx.send(true) {

                } else {
                    println!("Async update timer for player state has been disconnected");
                    break;
                }
                thread::sleep(time::Duration::from_secs_f32(1.5));
            }
        });

        let mut cmd = WorldCommandExecutor::new();
        cmd.load_file_bytc("resource/commands/test00.wcb".into());

        Self {
            player: player.clone(),
            command: cmd,

            event: EventQueue::new(),
            registry: Arc::new(BlockRegistry::new(&texture)),
            texture: texture,
            texture_fut: Some(txtr_future),
            player_ir: player.clone(),
            player_ir_signal: ply_sgn_rx,

            render_buffer: None,  // render data single buffer
            chunk_status_buffer: None,  // chunk status info single buffer
            chunk_chan_inp_tx: inp_tx,  // chunk thread input sending channel
            chunk_chan_out_rx: out_rx,  // chunk thread output receiving channel
            world_state: None,
        }
    }

    pub fn bind_texture( &mut self, gpu_future: Box<dyn GpuFuture>, ) -> Box<dyn GpuFuture> {
        let txtr_fut = mem::replace(&mut self.texture_fut, None);
        Box::new(gpu_future.join(txtr_fut.expect("Texture future has already been taken"))) as Box<dyn GpuFuture>
    }

    // update function on SEPARATE UPDATE THREAD
    pub fn update(&mut self, dimensions: Dimension<u32>, events: Vec<WorldEvents>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
                  rerender: bool,) {
        // println!("WORLD - UPDATE");
        self.event.merge_events(events);

        if let Ok(_) = self.player_ir_signal.try_recv() {
            self.player_ir = self.player.clone();
            self.player_ir_signal.try_iter();  // flushes the buffer
        }

        // submitting new chunk events to the Chunk Thread
        let mut chunk_events: Vec<ChunkEvents> = Vec::new();

        if let Some(stat) = &self.chunk_status_buffer {
            if stat.chunks_loaded > 0 || stat.chunks_offloaded > 0 {
                // println!("C: {:?}, L: {:?}, O: {:?}, U: X", stat.total_chunks_loaded, stat.chunks_loaded, stat.chunks_offloaded);
            }
        }

        if let Some(state) = &self.world_state {
            let new_state = WorldStateUpd::from_world(self.player_ir.clone(), self.registry.clone(), dimensions, renderpass.clone(), framebuffer.clone(), rerender);
            let update_state = state.update(&new_state);
            if !chunk_events.is_empty() || update_state != ChunkUpdateState::Consistent {
                let send = self.chunk_chan_inp_tx.send(
                    (chunk_events, new_state.clone(), update_state)
                );
                if let Err(e) = send {
                    println!("[WORLD] Send Error: {:?}", e);
                }

                self.world_state = Some(new_state);
            }
        } else {
            let new_state = WorldStateUpd::from_world(self.player_ir.clone(), self.registry.clone(), dimensions, renderpass.clone(), framebuffer.clone(), rerender);

            let send = self.chunk_chan_inp_tx.send(
                (chunk_events, new_state.clone(), ChunkUpdateState::Immediate)
            );
            if let Err(e) = send {
                println!("[WORLD] Initial Send Error: {:?}", e);
            }

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

        match self.chunk_chan_out_rx.try_recv() {
            Ok(buf) => {
                self.render_buffer = Some(buf.0);
                self.chunk_status_buffer = Some(buf.1);
            },
            Err(mpsc::TryRecvError::Empty) => {
                if let None = self.render_buffer {
                    if let Ok(buf) = self.chunk_chan_out_rx.recv() {
                        self.render_buffer = Some(buf.0);
                        self.chunk_status_buffer = Some(buf.1);
                    } else {
                        // TODO: HANDLE THIS CHANNEL DISCONNECTED CASE
                    }
                }
            },
            Err(mpsc::TryRecvError::Disconnected) => {
                // TODO: IMPORTANT: HANDLE THIS CHANNEL DISCONNECTED CASE
            },
        }
        // println!("[WORLD] Recv results");

        // the render buffer should always have value taken care by the code above
        let mut mesh_datas = self.render_buffer.clone().unwrap();
        mesh_datas.update_camera(device.clone(), &self.player.camera, dimensions);
        // TODO: Maybe add a method for a way to recompute the descriptors without touching the vert/indx buffer

        let mut cmd_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();

        cmd_builder
            .begin_render_pass(framebuffer.clone(), false, vec![[0.1, 0.3, 1.0, 1.0].into(), 1f32.into()]).unwrap()
            .draw_mesh(mesh_datas.cube.clone()).unwrap()
            .draw_mesh(mesh_datas.flora_x.clone()).unwrap()
            .end_render_pass().unwrap();

        cmd_builder.build().unwrap()
    }
}
