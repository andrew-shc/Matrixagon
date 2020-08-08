use super::chunk::Chunk;
use crate::world::mesh::{Meshes, cube::Cube};
use crate::event::{EventQueue, types::ChunkEvents};
use super::world::WorldStateUpd;
use crate::world::world::ChunkID;
use crate::datatype::{Position, ChunkUnit};
use crate::world::player::CHUNK_RADIUS;
use crate::world::chunk::{ChunkError, CHUNK_SIZE};
use crate::world::terrain::Terrain;
use crate::world::mesh::{MeshesExt, MeshDataType, MeshesStructType, MeshesDataType};
use crate::world::shader::VertexType;

use std::sync::{mpsc, Arc};
use std::thread;

use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, Queue};
use bitflags::_core::time::Duration;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::descriptor::DescriptorSet;
use vulkano::pipeline::input_assembly::Index;


pub type ThreadInput = (Vec<ChunkEvents>, WorldStateUpd);
pub type ThreadOutput<'b> = (MeshesDataType<'static>, ChunkStatusInfo);


#[derive(Clone, Debug)]
pub struct ChunkStatusInfo {
    pub chunks: Vec<(ChunkID, Position<ChunkUnit>)>,
    pub total_chunks_loaded: u32,
    pub chunks_loaded: u32,
    pub chunks_offloaded: u32,
    pub chunks_updated: u32,
}

impl ChunkStatusInfo {
    fn from_chunk_handler(handler: &ChunkHandler, chunks_ld: u32, chunks_offld: u32, chunks_upd: u32) -> Self {
        Self {
            chunks: handler.chunks.iter().map(|c| (c.id, c.position)).collect::<Vec<_>>(),
            total_chunks_loaded: handler.chunks.len() as u32,
            chunks_loaded: chunks_ld,
            chunks_offloaded: chunks_offld,
            chunks_updated: chunks_upd,
        }
    }
}


pub struct ChunkHandler {
    device: Arc<Device>,
    queue: Arc<Queue>,

    event: EventQueue<ChunkEvents>,  // event queue
    chunks: Vec<Chunk>,  // vectors of chunks
    meshes: MeshesStructType<'static>,  // world meshes
    terrain: Terrain,  // terrain of the world

    cid_counter: u32,  // chunk id counter

    // channels
    chunk_chan_inp_rx: mpsc::Receiver<ThreadInput>,  // chunk thread input receiving channel
    chunk_chan_out_tx: mpsc::Sender<ThreadOutput<'static>>,  // chunk thread output sending channel
}

impl ChunkHandler {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>,
               inp_rx: mpsc::Receiver<ThreadInput>, out_tx: mpsc::Sender<ThreadOutput>,
               meshes: MeshesStructType<'static>, terrain: Terrain) -> Self {
        Self {
            device: device.clone(),
            queue: queue.clone(),

            event: EventQueue::new(),
            chunks: Vec::new(),
            meshes: meshes,
            terrain: terrain,

            cid_counter: 0,
            chunk_chan_inp_rx: inp_rx,
            chunk_chan_out_tx: out_tx,
        }
    }

    // starts the update loop
    pub fn instantiate(mut self) {
        thread::spawn( move || {
            loop {
                match self.chunk_chan_inp_rx.try_recv() {
                    Ok(buf) => {
                        self.update(buf.0, buf.1);
                    },
                    Err(mpsc::TryRecvError::Empty) => {
                        continue;
                    },
                    Err(mpsc::TryRecvError::Disconnected) => {
                        break;
                    },
                }
                // println!("[CHUNK THREAD] Recv results");

                // thread::sleep(Duration::from_millis(10));
            }
        });
    }

    // updates every game tick, then returns the World Mesh Data
    fn update(&mut self, mut events: Vec<ChunkEvents>, state: WorldStateUpd) {
        let mut chunk_loaded = 0;
        let mut chunk_offloaded = 0;

        println!("Rerendering State WorldStUpd: {:?}; Dimn: {:?}", state.rerender, state.dimensions);

        // world.player position in chunk position
        let chunk_pos: Position<i64> = Position::new(
            (state.player.camera.position.coords.data[0] / CHUNK_SIZE as f32).floor() as i64,
            (state.player.camera.position.coords.data[1] / CHUNK_SIZE as f32).floor() as i64,
            (state.player.camera.position.coords.data[2] / CHUNK_SIZE as f32).floor() as i64,
        );

        for x in -(CHUNK_RADIUS as i64)..CHUNK_RADIUS as i64 {
            for y in -(CHUNK_RADIUS as i64)..CHUNK_RADIUS as i64 {
                for z in -(CHUNK_RADIUS as i64)..CHUNK_RADIUS as i64 {
                    events.push(ChunkEvents::LoadChunk(
                        Position::new(
                            ChunkUnit((chunk_pos.x+x) as f32),
                            ChunkUnit((chunk_pos.y+y) as f32),
                            ChunkUnit((chunk_pos.z+z) as f32),
                        )
                    ));
                }
            }
        }

        for chunk in self.chunks.clone() {
            if  chunk_pos.x-(CHUNK_RADIUS as i64) > chunk.position.x.0 as i64 || chunk.position.x > ChunkUnit((chunk_pos.x+(CHUNK_RADIUS as i64)) as f32) &&
                chunk_pos.y-(CHUNK_RADIUS as i64) > chunk.position.y.0 as i64 || chunk.position.y > ChunkUnit((chunk_pos.y+(CHUNK_RADIUS as i64)) as f32) &&
                chunk_pos.z-(CHUNK_RADIUS as i64) > chunk.position.z.0 as i64 || chunk.position.z > ChunkUnit((chunk_pos.z+(CHUNK_RADIUS as i64)) as f32) {

                events.push(ChunkEvents::OffloadChunk(chunk.id));
            }
        }

        // println!("[Chunk Thread] Events: {:?}", events);

        // TODO: Maybe change the run_event closure, because returns error of multiple mutable ref to self
        let mut event = self.event.clone();

        event.merge_events(events.clone());
        event.run_event(|e| {
            match e {
                // TODO: might need to change ChunkUnit to i128, to support "infinite" world.terrain generation
                // TODO: sometime later we need to deserialize/load chunk from save files
                // creates/loads a new chunk at the `pos`
                ChunkEvents::LoadChunk(pos) => {
                    if let Ok(id) = self.chunk_id(pos) {
                        let new_chunk = Chunk::new(id, pos, self.terrain.generate_chunk(pos));
                        self.meshes.add_chunk(new_chunk.id);
                        self.chunks.push(new_chunk);
                        chunk_loaded += 1;
                    }
                },
                // removes/saves a chunk from the world
                ChunkEvents::OffloadChunk(id) => {
                    self.meshes.remv_chunk(id);

                    for ind in 0..self.chunks.len() {
                        if self.chunks[ind].id == id {
                            self.chunks.swap_remove(ind);
                            break;
                        }
                    }
                    chunk_offloaded += 1;
                },
                // reloads all the chunk vertex data and index data
                ChunkEvents::ReloadChunks => {

                },
                // reloads a specific chunk (via ChunkID) vertex data and index data
                ChunkEvents::ReloadChunk(id) => {

                },
                // the final event emitted
                ChunkEvents::EventFinal => {
                    self.meshes.load_chunks(&self.chunks);
                }
            }
        });

        // we need world state update: rerender, framebuffer, renderpass

        // TODO: will remove this later after the issue above is fixed

        self.meshes.update(state.dimensions, &state.player);

        let mesh_datas = self.meshes.render(self.device.clone(), state.renderpass.clone(), state.dimensions, state.rerender, events);

        let send = self.chunk_chan_out_tx.send(
            (mesh_datas, ChunkStatusInfo::from_chunk_handler(&self, chunk_loaded, chunk_offloaded, 0))
        );
        // println!("[CHUNK THREAD] Send results: {:?}", send);
    }

    fn chunk_id(&mut self, position: Position<ChunkUnit>) -> Result<ChunkID, ChunkError> {
        // checking if *all* of the chunks have *different* positions; no duplicate position
        if self.chunks.iter().all(|x| x.position != position) {
            self.cid_counter += 1;
            Ok(ChunkID(self.cid_counter))
        } else {
            Err(ChunkError::DuplicateChunkPos)
        }
    }
}
