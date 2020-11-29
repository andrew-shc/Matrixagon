use crate::event::EventName;
use crate::world::ChunkID;
use crate::datatype::{Position, ChunkUnit, Dimension};
use crate::world::player::camera::Camera;

use std::any::TypeId;
use std::collections::HashMap;


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
