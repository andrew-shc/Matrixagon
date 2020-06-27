use winit::dpi::PhysicalSize;

use std::ops::{Div, Mul, Add, Sub};
use std::fmt::Debug;

use num_traits::float::Float;

use na::{
    Matrix4,
    Point3
};


pub enum BlockFace {
    Top, Bottom,
    Left, Right,
    Front, Back,
}

#[derive(Debug)]
pub enum CamDirection {
    Forward, Backward,
    Leftward, Rightward,
    Upward, Downward,
}

#[derive(Debug, Copy, Clone)]
pub struct Dimension<T: Copy + Div + Mul> {
    pub height: T,
    pub width: T,
}

impl<T: Copy + Div + Mul + Into<f64>> Dimension<T> {
    pub fn new(height: T, width: T) -> Self {
        Self {
            height,
            width
        }
    }

    pub fn resize(&mut self, height: T, width: T) {
        self.height = height;
        self.width = width;
    }

    // returns the aspect ratio
    pub fn aspect(&self) -> f64 {
        self.width.into() / self.height.into()
    }
}

impl<T: Copy + Div + Mul> From<PhysicalSize<T>> for Dimension<T> {
    fn from(item: PhysicalSize<T>) -> Self {
        Self {
            height: item.height,
            width: item.width,
        }
    }
}

impl<T: Copy + Div + Mul> From<Dimension<T>> for [T; 2] {
    fn from(item: Dimension<T>) -> Self {
        [item.width, item.height]
    }
}

impl From<Dimension<u32>> for [f32; 2] {
    fn from(item: Dimension<u32>) -> Self {
        [item.width as f32, item.height as f32]
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Position<T: Copy + PartialEq + Debug> {
    pub x: T,
    pub y: T,
    pub z: T,
}

// TODO: impl Add and Subtract traits to the position
impl<T> Position<T>
    where T: Copy + PartialEq + Debug + Mul + Add + Sub
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z,
        }
    }
}

impl Default for Position<f32> {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Default for Position<u32> {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl<T: Copy + PartialEq + Debug + 'static> From<Point3<T>> for Position<T> {
    fn from(item: Point3<T>) -> Self {
        Self {
            x: item.coords.data[0],
            y: item.coords.data[1],
            z: item.coords.data[2]
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rotation<T: Copy + Debug + PartialEq + Float> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl Rotation<f32> {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        let sx = self.x.sin();
        let cx = self.x.cos();
        let sy = self.y.sin();
        let cy = self.y.cos();
        let sz = self.z.sin();
        let cz = self.z.cos();

        Matrix4::new( // z
             cz, -sz, 0.0, 0.0,
             sz,  cz, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ) * Matrix4::new( // y
             cy, 0.0,  sy, 0.0,
            0.0, 1.0, 0.0, 0.0,
            -sy, 0.0,  cy, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ) * Matrix4::new( // x
            1.0, 0.0, 0.0, 0.0,
            0.0,  cx, -sx, 0.0,
            0.0,  sx,  cx, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }
}

impl Default for Rotation<f32> {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}
