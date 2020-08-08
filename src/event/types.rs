use crate::event::EventType;
use crate::world::world::ChunkID;
use crate::datatype::{Position, ChunkUnit};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChunkEvents {
    LoadChunk(Position<ChunkUnit>),  // reads/generates a new chunk to the world and meshes
    OffloadChunk(ChunkID),  // saves/discards the selected chunk from the world and meshes
    ReloadChunks,  // reloads all the chunk, or basically reload all the world data
    ReloadChunk(ChunkID),  // reloads a specific chunk

    EventFinal,  // emits when all the events has been consumed; or loaded by users if needed
}

impl EventType<ChunkEvents> for ChunkEvents {

    fn final_event() -> ChunkEvents {
        ChunkEvents::EventFinal
    }
}
