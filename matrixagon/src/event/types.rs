use crate::event::EventName;
use crate::world::ChunkID;
use crate::datatype::{Position, ChunkUnit, Dimension};
use crate::world::player::camera::Camera;

use std::any::TypeId;
use std::collections::HashMap;


// pub fn global_enm() -> Vec<EventName> {
//     vec![
//         EventName("MeshEvent/NewChunk"),
//         EventName("MeshEvent/LoadChunk"),
//         EventName("MeshEvent/OffloadChunk"),
//         EventName("MeshEvent/ReloadChunks"),
//         EventName("MeshEvent/ReloadChunk"),
//         EventName("MeshEvent/UpdateMesh"),
//         EventName("MeshEvent/UpdateDimensions"),
//         EventName("MeshEvent/UpdateWorldStates"),
//         EventName("WorldEvent/NewChunk"),
//         EventName("WorldEvent/LoadChunk"),
//         EventName("WorldEvent/OffloadChunk"),
//         EventName("WorldEvent/ReloadChunks"),
//         EventName("WorldEvent/ReloadChunk"),
//         EventName("EventFinal"),
//     ]
// }
//
// // creates a hashmap of event nametypes for the EventInterchange
// macro_rules! ename_insert {
//     {$($key:literal => $val:ty,)*} => {
//         let mut map = HashMap::new();
//         $(map.insert(EventName($key), TypeId::of::<$val>());)*
//         map
//     }
// }
//
// // returns the event nametypes for this voxel applications
// pub fn global_enmtyp() -> HashMap<EventName, TypeId> {
//     ename_insert! {
//         "MeshEvent/NewChunk"            => Position<ChunkUnit>,
//         "MeshEvent/LoadChunk"           => u32,
//         "MeshEvent/OffloadChunk"        => ChunkID,
//         "MeshEvent/ReloadChunks"        => (),
//         "MeshEvent/ReloadChunk"         => ChunkID,
//         "MeshEvent/UpdateMesh"          => (),
//         "MeshEvent/UpdateDimensions"    => Dimension<u32>,
//         "MeshEvent/UpdateWorldStates"   => Camera,
//         "WorldEvent/NewChunk"           => Position<ChunkUnit>,
//         "WorldEvent/LoadChunk"          => u32,
//         "WorldEvent/OffloadChunk"       => ChunkID,
//         "WorldEvent/ReloadChunks"       => (),
//         "WorldEvent/ReloadChunk"        => ChunkID,
//         "EventFinal"                    => (),
//     }
// }


// creates a hashmap of event nametypes for the EventInterchange
macro_rules! ename_insert {
    {$($key:literal => [$($val:ty$(,)?)*],)*} => {
        let mut map = HashMap::new();
        $(map.insert(EventName($key), vec![$(TypeId::of::<$val>(),)*]);)*
        map
    }
}

// returns the event nametypes for this voxel applications
pub fn global_enmtyp() -> HashMap<EventName, Vec<TypeId>> {
    ename_insert! {
        "MeshEvent/NewChunk"            => [Position<ChunkUnit>],
        "MeshEvent/LoadChunk"           => [u32],
        "MeshEvent/OffloadChunk"        => [ChunkID],
        "MeshEvent/ReloadChunks"        => [],
        "MeshEvent/ReloadChunk"         => [ChunkID],
        "MeshEvent/UpdateMesh"          => [],
        "MeshEvent/UpdateDimensions"    => [Dimension<u32>],
        "MeshEvent/UpdateWorldStates"   => [Camera],
        "WorldEvent/NewChunk"           => [Position<ChunkUnit>],
        "WorldEvent/LoadChunk"          => [u32],
        "WorldEvent/OffloadChunk"       => [ChunkID],
        "WorldEvent/ReloadChunks"       => [],
        "WorldEvent/ReloadChunk"        => [ChunkID],
        "EventFinal"                    => [],
    }
}

// pub struct ChunkData {
//     pub pos: bool,
// }
//
// impl Observer for ChunkData {
//     fn event_name(&self) -> EventName {
//         EventName("@TEST/Testing")
//     }
//
//     fn receive(&mut self, mut evd: EventData) {
//         self.pos = evd.pop::<bool>();
//     }
//
//     fn retrieve_data(&self) -> EventData {
//         let mut ed = EventData::new();
//         ed.push(self.pos);
//         ed.pack();
//         ed
//     }
// }

//
// pub struct World {
//     inner0: u32,
//     obs_s: Vec<dyn Observer>,
// }
//
// impl World {
//     fn new() -> Self {
//         Self {
//             inner0: 0,
//             obs_s: vec![0]
//         }
//     }
// }
//
// pub trait Observer {
//     // the event name you are observing to
//     fn event_name() -> EventName;
//     fn notify(&mut self, evd: EventData) -> EventName;
//     fn retrieve_data(&self) -> EventData;
// }
//
// pub struct ChunkData {
//     pos: bool,
// };
//
// impl Observer for ChunkData {
//     fn event_name() -> EventName {
//         EventName("@TEST/Testing")
//     }
//
//     fn notify(&mut self, mut evd: EventData) {
//         self.pos = evd.pop::<bool>();
//     }
//
//     fn retrieve_data(&self) -> EventData {
//         let mut ed = EventData::new();
//         ed.push(self.pos);
//         ed.pack();
//         ed
//     }
// }
//

// struct EventDispatcherNEW {
//     obs: Vec<Box<dyn Observer>>,
// }
//
// impl EventDispatcherNEW {
//     fn new() -> Self {
//         Self {
//             obs: Vec::new(),
//         }
//     }
//
//     fn emit(&self, en: EventName, mut ed: EventData) {
//         ed.pack();
//         let edd = Rc::new(RefCell::new(ed));
//         for o in self.obs {
//
//             o.notify(ed.clone());
//         }
//     }
//
//     fn subscribe(&mut self, ob: impl Observer) {
//         self.obs.push(ob)
//     }
// }

/*
{EventName: }
 */

