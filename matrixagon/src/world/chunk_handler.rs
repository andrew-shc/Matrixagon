use crate::world::chunk::Chunk;
use crate::event::{EventQueue, types::ChunkEvents, ControlFlow};
use crate::world::{WorldStateUpd, ChunkUpdateState};
use crate::world::ChunkID;
use crate::datatype::{Position, ChunkUnit};
use crate::world::player::CHUNK_RADIUS;
use crate::world::chunk::{ChunkError, CHUNK_SIZE};
use crate::world::terrain::Terrain;
use crate::world::mesh::{MeshesStructType, MeshesDataType};
use crate::world::chunk_threadpool::ChunkThreadPool;

use vulkano::device::{Device, Queue};

use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::thread;
use std::thread::JoinHandle;

pub type ThreadInput = (Vec<ChunkEvents>, WorldStateUpd, ChunkUpdateState);
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
    meshes: MeshesStructType,  // world meshes
    terrain: Terrain,  // terrain of the world

    timer_counter: JoinHandle<()>,  // timer to prevent the event executing too long
    timer_channel: mpsc::Receiver<bool>,

    cid_counter: u32,  // chunk id counter
    chunk_threadpool: ChunkThreadPool,

    // channels
    chunk_chan_inp_rx: mpsc::Receiver<ThreadInput>,  // chunk thread input receiving channel
    chunk_chan_out_tx: mpsc::Sender<ThreadOutput<'static>>,  // chunk thread output sending channel
    chunk_chan_inp_tx: mpsc::Sender<ThreadInput>,  // uses the sender to transfer the result from "peeking" in the result
}

impl ChunkHandler {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>,
               inp_rx: mpsc::Receiver<ThreadInput>, out_tx: mpsc::Sender<ThreadOutput>, inp_tx: mpsc::Sender<ThreadInput>,
               meshes: MeshesStructType, terrain: Terrain) -> Self {
        let (timer_tx, timer_rx) = mpsc::channel();

        Self {
            device: device.clone(),
            queue: queue.clone(),

            event: EventQueue::new(),
            chunks: Vec::new(),
            meshes: meshes,
            terrain: terrain,

            timer_counter: thread::spawn(move || {
                loop {
                    let res = timer_tx.send(true);
                    if let Err(e) = res {
                        println!("Timer of chunk handler has the sender returned an error");
                        break;
                    }
                    thread::sleep(Duration::from_secs(2));
                }
            }),
            timer_channel: timer_rx,

            cid_counter: 0,
            // high number: faster chunk generation but laggier across the whole computer
            // low number: slower chunk generation (maybe even stack overflow) but smoother across the whole computer
            chunk_threadpool: ChunkThreadPool::new(8),

            chunk_chan_inp_rx: inp_rx,
            chunk_chan_out_tx: out_tx,
            chunk_chan_inp_tx: inp_tx,
        }
    }

    // starts the update loop
    pub fn instantiate(mut self) {
        let thread_builder = thread::Builder::new()
            .name("Chunk Handler".into());
        let res = thread_builder.spawn( move || {
            loop {
                match self.chunk_chan_inp_rx.try_recv() {
                    Ok(buf) => {
                        println!("Res filled");
                        self.update(buf.0, buf.1);
                    },
                    Err(mpsc::TryRecvError::Empty) => {
                        // println!("Res empty");
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
        if let Err(e) = res {
            println!("The 'Chunk Handler' thread has failed to spawn correctly: {}", e);
        }
    }

    // updates every game tick, then returns the World Mesh Data
    fn update(&mut self, mut events: Vec<ChunkEvents>, state: WorldStateUpd) {
        let mut chunk_loaded = 0;
        let mut chunk_offloaded = 0;

        // world.player position in chunk position
        let chunk_pos: Position<i64> = Position::new(
            (state.player.camera.position.coords.data[0] / CHUNK_SIZE as f32).floor() as i64,
            (state.player.camera.position.coords.data[1] / CHUNK_SIZE as f32).floor() as i64,
            (state.player.camera.position.coords.data[2] / CHUNK_SIZE as f32).floor() as i64,
        );

        for x in -(CHUNK_RADIUS as i64)..=CHUNK_RADIUS as i64 {
            for y in -(CHUNK_RADIUS as i64)..=CHUNK_RADIUS as i64 {
                for z in -(CHUNK_RADIUS as i64)..=CHUNK_RADIUS as i64 {
                    // prevent chunk generation below y-level 0
                    if (chunk_pos.y+y) >= 0 {
                        let new_pos = Position::new(
                            ChunkUnit((chunk_pos.x+x) as f32),
                            ChunkUnit((chunk_pos.y+y) as f32),
                            ChunkUnit((chunk_pos.z+z) as f32),
                        );


                        // checks for duplicated position before submitting an event
                        if self.chunks.iter().all(|x| x.position != new_pos) {
                            events.push(ChunkEvents::LoadChunk(new_pos));
                        }
                    }
                }
            }
        }

        for chunk in self.chunks.clone() {
            if  chunk_pos.x-(CHUNK_RADIUS as i64) > i64::from(chunk.position.x) || chunk.position.x > ChunkUnit((chunk_pos.x+(CHUNK_RADIUS as i64)) as f32) &&
                chunk_pos.y-(CHUNK_RADIUS as i64) > i64::from(chunk.position.y) || chunk.position.y > ChunkUnit((chunk_pos.y+(CHUNK_RADIUS as i64)) as f32) &&
                chunk_pos.z-(CHUNK_RADIUS as i64) > i64::from(chunk.position.z) || chunk.position.z > ChunkUnit((chunk_pos.z+(CHUNK_RADIUS as i64)) as f32) {

                events.push(ChunkEvents::OffloadChunk(chunk.id));
            }
        }

        println!("[Chunk Thread] Events: {:?}", events);

        let mut event = self.event.clone();
        event.merge_events(events.clone());

        // running through empty events does cost some computation times
        if event.event_count() > 0 {
            // TODO: add event interrupt to pause the event before rerunning
            event.run_event(|e, c| {
                // if let Ok(_) = self.timer_channel.try_recv() {
                //     *c = ControlFlow::Halt;
                // }
                if let Ok(transf) = self.chunk_chan_inp_rx.try_recv() {
                    if transf.2 == ChunkUpdateState::Immediate {
                        self.chunk_chan_inp_rx.try_iter();  // cleans input buffer for the priority
                        if let Err(e) = self.chunk_chan_inp_tx.send(transf) {
                            println!("The chunk channel receiver peeker had failed to transfer the results to the chunk channel sender");
                        }
                        *c = ControlFlow::Halt;
                        println!("Priority incoming!");
                    }
                }

                // println!("EEEEEEEEEEEEE {:?}", e);
                match e {
                    // TODO: might need to change ChunkUnit to i128, to support more "infinite" world.terrain generation
                    // TODO: sometime later we need to deserialize/load chunk from save files
                    // creates/loads a new chunk at the `pos`
                    ChunkEvents::LoadChunk(pos) => {
                        // The id grabber will automatically check for any dupe position
                        // println!("LDC - A");
                        if let Ok(id) = self.chunk_id(*pos) {
                            // println!("LDC - B");
                            let new_chunk = Chunk::new(id, *pos, self.terrain.generate_chunk(state.registry.clone(), *pos));
                            self.meshes.add_chunk(new_chunk.id);
                            self.chunks.push(new_chunk);
                            chunk_loaded += 1;
                        }
                    },
                    // removes/saves a chunk from the world
                    ChunkEvents::OffloadChunk(id) => {
                        self.meshes.remv_chunk(*id);

                        for ind in 0..self.chunks.len() {
                            if self.chunks[ind].id == *id {
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
                        self.meshes.load_chunks(self.chunks.clone(), &mut self.chunk_threadpool);
                    },
                    _ => {},
                }
            });
            self.event = event;
        }

        // TODO: will remove this later after the issue above is fixed

        self.meshes.update(state.dimensions, &state.player);

        let mesh_datas = self.meshes.render(self.device.clone(), state.renderpass.clone(), state.dimensions, state.rerender, events);

        let send = self.chunk_chan_out_tx.send(
            (mesh_datas, ChunkStatusInfo::from_chunk_handler(&self, chunk_loaded, chunk_offloaded, 0))
        );
        if let Err(e) = send {
            println!("[CHUNK THREAD] Send Error: {:?}", e);
        }
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
