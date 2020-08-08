use super::player::{Player, CHUNK_RADIUS};
use super::terrain::Terrain;
use super::mesh::{Meshes, MeshesExt};
use super::chunk::{Chunk, ChunkError, ChunkUpdate, CHUNK_SIZE};
use crate::datatype::{Position, Dimension, LocalBUIntermediate, BlockUnit, LocalBU, ChunkUnit};
use super::texture::Texture;
use super::chunk_handler;
use super::chunk_handler::ChunkHandler;
use crate::event::types::ChunkEvents;
use crate::world::chunk_handler::ChunkStatusInfo;
use crate::world::shader::VertexType;
use crate::world::mesh::{MeshDataType, MeshesDataType};

use vulkano::device::{Queue, Device};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::framebuffer::{RenderPassAbstract, FramebufferAbstract};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::sync::GpuFuture;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::descriptor::DescriptorSet;
use vulkano::pipeline::input_assembly::Index;

use na::{
    Point3,
};

use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::mem;


#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ChunkID(pub u32);

// World State Update struct
// This strut is used to format the required states from the world to generate a World Mesh Data
#[derive(Clone)]
pub struct WorldStateUpd {
    pub player: Player,
    pub dimensions: Dimension<u32>,

    pub rerender: bool,
    pub renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>
}

impl WorldStateUpd {
    fn from_world(world: &World, dimn: Dimension<u32>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
                  rerender: bool,
    ) -> Self {
        Self {
            player: world.player.clone(),
            dimensions: dimn,

            rerender: rerender,
            renderpass: renderpass.clone(),
            framebuffer: framebuffer.clone()
        }
    }
}

pub struct World {
    // world entities/components
    pub player: Player,
    terrain: Terrain,

    // world.weather:

    // world structure and manager
    texture: Texture,

    // multithreading
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

        // TODO: textures: use macro expansion to automatically add the textures AT COMPILE TIME
        // TODO: or implement somw ways to add textures at RUN TIME
        let mut texture = Texture::new(queue.clone());

        texture.add(include_bytes!("../../resource/texture/blocks/air.png").to_vec(), "air");
        texture.add(include_bytes!("../../resource/texture/blocks/grass_side.png").to_vec(), "grass_side");
        texture.add(include_bytes!("../../resource/texture/blocks/grass_top.png").to_vec(), "grass_top");
        texture.add(include_bytes!("../../resource/texture/blocks/dirt.png").to_vec(), "dirt");
        texture.add(include_bytes!("../../resource/texture/blocks/sand.png").to_vec(), "sand");
        texture.add(include_bytes!("../../resource/texture/blocks/stone.png").to_vec(), "stone");

        let (inp_tx, inp_rx) = mpsc::channel();  // new chunk events/world state -> chunk handler channel
        let (out_tx, out_rx) = mpsc::channel();  // chunk handler -> render data/chunk statuses channel

        ChunkHandler::new(
            device.clone(), queue.clone(),
            inp_rx, out_tx,
            Meshes::new(device.clone(), &texture, renderpass.clone(), dimensions.clone()),
            Terrain::new(&texture, 24)
        ).instantiate();

        Self {
            player: Player::new(),
            terrain: Terrain::new(&texture, 24),

            texture: texture,

            render_buffer: None,  // render data single buffer
            chunk_status_buffer: None,  // chunk status info single buffer
            chunk_chan_inp_tx: inp_tx,  // chunk thread input sending channel
            chunk_chan_out_rx: out_rx,  // chunk thread output receiving channel
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
    pub fn update(&mut self, dimensions: Dimension<u32>,
                  renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,
                  rerender: bool,) {
        // println!("WORLD - UPDATE");
        // println!("Player {:?}", world.world.player);

        // sumbitting new chunk events to the Chunk Thread
        let mut chunk_events: Vec<ChunkEvents> = Vec::new();

        if let Some(stat) = &self.chunk_status_buffer {
            if stat.chunks_loaded > 0 || stat.chunks_offloaded > 0 {
                // println!("C: {:?}, L: {:?}, O: {:?}, U: X", stat.total_chunks_loaded, stat.chunks_loaded, stat.chunks_offloaded);
            }
        }

        // println!("Pre Rerendering State: {:?}; Dimn: {:?}", rerender, dimensions);

        let send = self.chunk_chan_inp_tx.send(
            (chunk_events, WorldStateUpd::from_world(&self, dimensions, renderpass.clone(), framebuffer.clone(), rerender))
        );
        // println!("[WORLD] Send results: {:?}", send);
    }

    // TODO: moving this to a proper event at chunk_handler
    // pub fn do_block(&mut self, breaking: bool, placing: bool) {
    //     let block_break = self.player.camera.raycast_break(&self.chunks);
    //
    //     if let Some((bpos, block)) = block_break {
    //         let mut chunk_edited = 0;  // TODO: temporary chunk info logger
    //         // TODO: thinking of a stack machine logger
    //         // TODO: same thing for event system
    //
    //         let bchunk = Position::new(
    //             bpos.x.into_chunk_int(),
    //             bpos.y.into_chunk_int(),
    //             bpos.z.into_chunk_int(),
    //         );
    //
    //         println!("Crosshair Ray: {:?}", block_break);
    //
    //         // TODO: convert chunk position type into ChunkPosition
    //         if self.chunks.iter().any(|c| c.position == bchunk) {
    //             let chunks_cloned = self.chunks.clone();
    //
    //             for (ind, mut c) in self.chunks.iter_mut().enumerate() {
    //                 if c.position == bchunk {
    //                     let pos = Position::new(
    //                         if let LocalBUIntermediate::UBound(px) | LocalBUIntermediate::Ok(px) = bpos.x.into_local_bu() {px} else {LocalBU(0f32)},
    //                         if let LocalBUIntermediate::UBound(py) | LocalBUIntermediate::Ok(py) = bpos.y.into_local_bu() {py} else {LocalBU(0f32)},
    //                         if let LocalBUIntermediate::UBound(pz) | LocalBUIntermediate::Ok(pz) = bpos.z.into_local_bu() {pz} else {LocalBU(0f32)},
    //                     );
    //                     if let Some(c_ind) = chunks_cloned.iter().position(|int_c| int_c.id == c.id) {
    //                         c.update(pos, self.terrain.blocks["air"]);
    //                         self.meshes.load_chunks(&vec![c.clone()]);
    //                         chunk_edited += 1;
    //                     }
    //                 }
    //             }
    //         }
    //         self.chunk_flags = self.chunk_flags | ChunkUpdate::BlockUpdate;
    //     }
    // }

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
            .end_render_pass().unwrap();

        cmd_builder.build().unwrap()
    }
}
