use crate::event::{EventType, EventTransfer, EventQueue};
use crate::world::ChunkID;
use crate::datatype::{Position, ChunkUnit, Dimension};
use std::any::Any;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChunkEvents {
    NewChunk(Position<ChunkUnit>),  // TODO: generates a new chunk to the world and meshes
    LoadChunk(Position<ChunkUnit>),  // reads/generates a new chunk to the world and meshes
    OffloadChunk(ChunkID),  // saves/discards the selected chunk from the world and meshes
    ReloadChunks,  // reloads all the chunk, or basically reload all the world data
    ReloadChunk(ChunkID),  // reloads a specific chunk
    UpdateDimension(Dimension<u32>),  // updates chunk mesh's graphic pipeline display dimension

    EventFinal,  // emits when all the events has been consumed; or loaded by users if needed
}

impl EventType<ChunkEvents> for ChunkEvents {
    fn final_event() -> ChunkEvents {
        ChunkEvents::EventFinal
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WorldEvents {
    PlayerPosUpdate(Position<f32>),

    EventFinal,
}

impl EventType<WorldEvents> for WorldEvents {
    fn final_event() -> WorldEvents {
        WorldEvents::EventFinal
    }
}

impl EventTransfer<ChunkEvents> for EventQueue<WorldEvents> {
    fn transfer_into(&mut self) -> Vec<ChunkEvents> {
        unimplemented!()
    }

    fn transfer_copy(&self) -> Vec<ChunkEvents> {
        unimplemented!()
    }

    fn transfer_except(&mut self) -> Vec<ChunkEvents> {
        unimplemented!()
    }
}


// App events are also directly used by the UI system
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AppEvents {
    // Mouse pressed

    EventFinal,
}

impl EventType<AppEvents> for AppEvents {
    fn final_event() -> AppEvents {
        AppEvents::EventFinal
    }
}

impl EventTransfer<WorldEvents> for EventQueue<AppEvents> {
    fn transfer_into(&mut self) -> Vec<WorldEvents> {
        unimplemented!()
    }

    fn transfer_copy(&self) -> Vec<WorldEvents> {
        unimplemented!()
    }

    fn transfer_except(&mut self) -> Vec<WorldEvents> {
        unimplemented!()
    }
}
