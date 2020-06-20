use winit::dpi::PhysicalSize;

use cgmath::{Point3, Deg};
use std::ops::{Div, Mul, Add, Sub};
use std::fmt::Debug;


pub enum BlockFace {
    Top, Bottom,
    Left, Right,
    Front, Back,
}

pub enum CamDirection {
    Forward, Backward,
    Leftward, Rightward,
    Upward, Downward,
    None,
}

#[derive(Debug, Copy, Clone)]
pub struct Dimension<T: Copy + Div + Mul> {
    pub height: T,
    pub width: T,
}

impl<T: Copy + Div + Mul> Dimension<T> {
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
    pub fn aspect(&self) -> <T as std::ops::Div>::Output {
        self.width / self.height
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

    // multiply the whole (x, y, z) by a value
    pub fn mlp(&self, val: T) -> Position<<T as Mul>::Output>
        where <T as Mul>::Output: PartialEq + Copy + Debug
    {
        Position {
            x: self.x * val,
            y: self.y * val,
            z: self.z * val,
        }
    }

    // addition the whole (x, y, z) by a value
    pub fn add(&self, val: T) -> Position<<T as Add>::Output>
        where <T as Add>::Output: PartialEq + Copy + Debug
    {
        Position {
            x: self.x + val,
            y: self.y + val,
            z: self.z + val,
        }
    }

    // subtraction the whole (x, y, z) by a value
    pub fn sub(&self, val: T) -> Position<<T as Sub>::Output>
        where <T as Sub>::Output: PartialEq + Copy + Debug
    {
        Position {
            x: self.x - val,
            y: self.y - val,
            z: self.z - val,
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

impl<T: Copy+PartialEq+Debug> From<Position<T>> for Point3<T> {
    fn from(item: Position<T>) -> Self {
        Self {
            x: item.x,
            y: item.y,
            z: item.z,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rotation<T: Copy + Debug> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Copy + Debug> Rotation<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self {
            x,
            y,
            z,
        }
    }
}

impl Default for Rotation<Deg<f32>> {
    fn default() -> Self {
        Self {
            x: Deg(0.0),
            y: Deg(0.0),
            z: Deg(0.0),
        }
    }
}

impl Default for Rotation<u32> {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
        }
    }
}
