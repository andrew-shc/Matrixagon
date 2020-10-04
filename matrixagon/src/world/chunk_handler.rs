use crate::world::chunk::Chunk;
use crate::event::{EventDispatcher, EventName};
use crate::world::WorldStateUpd;
use crate::world::ChunkID;
use crate::datatype::{Position, ChunkUnit, Dimension};
use crate::world::player::CHUNK_RADIUS;
use crate::world::chunk::{ChunkError, CHUNK_SIZE};
use crate::world::terrain::Terrain;
use crate::world::mesh::{MeshesStructType, MeshesDataType};
use crate::world::chunk_threadpool::ChunkThreadPool;
use crate::world::player::camera::Camera;

use vulkano::device::{Device, Queue};

use std::sync::Arc;
use std::rc::Rc;


pub type ThreadInput = WorldStateUpd;
pub type ThreadOutput<'b> = (MeshesDataType, ChunkStatusInfo);

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

    event: Rc<EventDispatcher>,  // event queue
    chunks: Vec<Chunk>,  // vectors of chunks
    meshes: MeshesStructType,  // world meshes
    terrain: Terrain,  // terrain of the world

    cid_counter: u32,  // chunk id counter
    chunk_threadpool: ChunkThreadPool,
    reload_chunks: bool,

    chunks_loaded: u32,
    chunks_offloaded: u32,
}

impl ChunkHandler {
    // creating a chunk handler requires you to communicate through mspc's
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, evd: Rc<EventDispatcher>,
               meshes: MeshesStructType, terrain: Terrain) -> Self {
        // chunk_obsv.subscribe(String::from("a"), a, a);

        Self {
            device: device.clone(),
            queue: queue.clone(),

            event: evd.clone(),
            chunks: Vec::new(),
            meshes: meshes,
            terrain: terrain,

            cid_counter: 0,
            // high number: faster chunk generation but laggier across the whole computer
            // low number: slower chunk generation (maybe even stack overflow) but smoother across the whole computer
            chunk_threadpool: ChunkThreadPool::new(8),
            reload_chunks: false,

            chunks_loaded: 0,
            chunks_offloaded: 0,
        }
    }

    // updates every game tick, then returns the World Mesh Data
    pub fn update(&mut self, state: WorldStateUpd) -> (MeshesDataType, ChunkStatusInfo) {
        /*
        // once for receive_once();
        event_receiver!(self.event){
            "MeshEvent/NewChunk" => once |mut param| {

            },
            "OtherEvents" => |mut param| {

            },
        }
        self.event.clone().receive_event()
            .receive(EventName(), mut || {
            })
            .receive_once(name, closure)

         */

        self.event.clone().receive(EventName("MeshEvent/NewChunk"),  |mut param| {
            let pos = param.pop::<Position<ChunkUnit>>();

            if let Ok(id) = self.chunk_id(pos) {
                let new_chunk = Chunk::new(id, pos, self.terrain.generate_chunk(pos));
                self.meshes.add_chunk(new_chunk.id);
                self.chunks.push(new_chunk);
                self.chunks_loaded += 1;
            }
            self.reload_chunks = true;
        });
        self.event.clone().receive(EventName("MeshEvent/LoadChunk"), |mut param| {
            let _ = param.pop::<u32>();
            self.reload_chunks = true;
        });
        self.event.clone().receive(EventName("MeshEvent/OffloadChunk"), |mut param| {
            let id = param.pop::<ChunkID>();

            self.meshes.remv_chunk(id);

            for ind in 0..self.chunks.len() {
                if self.chunks[ind].id == id {
                    self.chunks.swap_remove(ind);
                    break;
                }
            }
            self.chunks_offloaded += 1;
            self.reload_chunks = true;
        });
        self.event.clone().receive(EventName("MeshEvent/ReloadChunks"), |mut param| {
            self.reload_chunks = true;
        });
        self.event.clone().receive(EventName("MeshEvent/ReloadChunk"), |mut param| {
            let _ = param.pop::<ChunkID>();

            self.reload_chunks = true;
        });
        // Updates mesh with reloading all necessary chunks
        self.event.clone().receive(EventName("MeshEvent/UpdateMesh"), |mut param| {
            println!("begn");
            self.meshes.load_chunks(self.chunks.clone(), &mut self.chunk_threadpool);
            println!("endn");
            self.reload_chunks = true;
        });
        self.event.clone().receive(EventName("MeshEvent/UpdateDimensions"), |mut param| {
            let dimn = param.pop::<Dimension<u32>>();
            self.meshes.update(Some(dimn), None);
        });
        self.event.clone().receive(EventName("MeshEvent/UpdateWorldStates"), |mut param| {
            let cam = param.pop::<Camera>();

            self.meshes.update(None, Some(&cam));
        });

        let mut chunk_loaded = 0;
        let mut chunk_offloaded = 0;

        // world.player position in chunk position
        let chunk_pos: Position<i64> = Position::new(
            (state.cam.position.coords.data[0] / CHUNK_SIZE as f32).floor() as i64,
            (state.cam.position.coords.data[1] / CHUNK_SIZE as f32).floor() as i64,
            (state.cam.position.coords.data[2] / CHUNK_SIZE as f32).floor() as i64,
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
                            self.event.clone().emit(EventName("MeshEvent/NewChunk"), event_data![new_pos]);
                            chunk_loaded += 1;
                        }
                    }
                }
            }
        }

        // due to high numbers of chunk, there will be brutally optimized code here
        // lower bound
        let lb_x = ChunkUnit((chunk_pos.x-(CHUNK_RADIUS as i64)) as f32);
        let lb_y = ChunkUnit((chunk_pos.y-(CHUNK_RADIUS as i64)) as f32);
        let lb_z = ChunkUnit((chunk_pos.z-(CHUNK_RADIUS as i64)) as f32);
        // upper bound
        let ub_x = ChunkUnit((chunk_pos.x+(CHUNK_RADIUS as i64)) as f32);
        let ub_y = ChunkUnit((chunk_pos.y+(CHUNK_RADIUS as i64)) as f32);
        let ub_z = ChunkUnit((chunk_pos.z+(CHUNK_RADIUS as i64)) as f32);

        for chunk in &self.chunks {
            if  lb_x > chunk.position.x || chunk.position.x > ub_x &&
                lb_y > chunk.position.y || chunk.position.y > ub_y &&
                lb_z > chunk.position.z || chunk.position.z > ub_z {
                self.event.clone().emit(EventName("MeshEvent/OffloadChunk"), event_data![chunk.id]);
                chunk_offloaded += 1;
            }
        }

        if chunk_loaded > 0 || chunk_offloaded > 0 {
            println!("L {:?} O {:?}", chunk_loaded, chunk_offloaded);
            self.event.clone().emit(EventName("MeshEvent/UpdateMesh"), event_data![]);
        }

        // TODO: TEMPORARY: To be handled by events
        self.meshes.update(Some(state.dimensions), Some(&state.cam));

        // TODO: Calling this is really slow, once threadpool is completed, use threadpool
        let mesh_datas = self.meshes.render(self.device.clone(), state.renderpass.clone(), state.rerender, self.reload_chunks);
        (mesh_datas, ChunkStatusInfo::from_chunk_handler(&self, chunk_loaded, chunk_offloaded, 0))
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
