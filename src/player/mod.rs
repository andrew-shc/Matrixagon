use crate::player::camera::Camera;
use std::fmt::{Debug, Formatter};
use std::fmt;

pub mod camera;


pub struct Player {
    pub camera: Camera,
}

impl Player {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(1.0,1.0)
        }
    }
}

impl Debug for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Player")
            .field("position", &self.camera.position)
            .field("rotation", &self.camera.rotation)
            .finish()
    }
}
