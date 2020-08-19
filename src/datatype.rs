use crate::world::chunk::CHUNK_SIZE;

use winit::dpi::PhysicalSize;

use num_traits::float::Float;

use na::{
    Matrix4,
    Point3
};

use std::ops::{Div, Mul, Add, Sub, Neg};
use std::fmt::Debug;


// TODO: in future, there will be a custom float and integer type or uses another library's numeric types
// an empty trait for all standard float
trait StdFloat {}

impl StdFloat for f32 {}
impl StdFloat for f64 {}

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

// pointing to which direction from the center
#[derive(Debug)]
pub enum Direction {
    Up, Down,
    Left, Right,
    Front, Back,
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Dimension<T: Copy + Div + Mul + PartialEq> {
    pub height: T,
    pub width: T,
}

impl<T: Copy + Div + Mul + PartialEq + Into<f64>> Dimension<T> {
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

impl<T: Copy + Div + Mul + PartialEq> From<PhysicalSize<T>> for Dimension<T> {
    fn from(item: PhysicalSize<T>) -> Self {
        Self {
            height: item.height,
            width: item.width,
        }
    }
}

impl<T: Copy + Div + Mul + PartialEq> From<Dimension<T>> for [T; 2] {
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

impl Position<LocalBU> {

    // converts the local world.block position into a flatten vector world.block position
    pub fn into_vec_pos(self) -> usize {
        let x = if self.x.0 >= 0.0 {self.x.0.floor() as usize} else {(self.x.0+CHUNK_SIZE as f32).floor() as usize};
        let y = if self.x.0 >= 0.0 {self.y.0.floor() as usize} else {(self.y.0+CHUNK_SIZE as f32).floor() as usize};
        let z = if self.x.0 >= 0.0 {self.z.0.floor() as usize} else {(self.z.0+CHUNK_SIZE as f32).floor() as usize};

        x*CHUNK_SIZE*CHUNK_SIZE+y*CHUNK_SIZE+z
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug)]
pub enum LineIntersect<T: Copy + Clone + Debug> {
    None,
    // intersected at the point (x, y, z)
    Intersected(T, T, T),
    // if both lines are of same slope
    Parallel,
    // if both lines are of same slope and same y-intercept
    Coincident,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Line<T: Copy + PartialEq + Debug> {
    pub a: Position<T>,
    pub b: Position<T>,
}

impl<T: Copy + PartialEq + Debug> Line<T> {
    pub fn new(a: Position<T>, b: Position<T>) -> Self {
        Self { a, b }
    }
}

impl Line<f32> {
    // NOTE: this intersection does not care about the third position z
    pub fn intersect2d(&self, b: Line<f32>) -> LineIntersect<f32> {
        Self::inner_intersect2d(*self, b)
    }

    // this is an associate function so the intersect3d can swizzle the components of line a (self)
    fn inner_intersect2d(a: Line<f32>, b: Line<f32>) -> LineIntersect<f32> {
        let ua_n = (b.b.x-b.a.x)*(a.a.y-b.a.y) - (b.b.y-b.a.y)*(a.a.x-b.a.x);
        let ua_d = (b.b.y-b.a.y)*(a.b.x-a.a.x) - (b.b.x-b.a.x)*(a.b.y-a.a.y);
        let ub_n = (a.b.x-a.a.x)*(a.a.y-b.a.y) - (a.b.y-a.a.y)*(a.a.x-b.a.x);
        let ub_d = (b.b.y-b.a.y)*(a.b.x-a.a.x) - (b.b.x-b.a.x)*(a.b.y-a.a.y);

        if (0.0 < ua_n/ua_d && ua_n/ua_d < 1.0) && (0.0 < ub_n/ub_d && ub_n/ub_d < 1.0) {
            let x = a.a.x + (ua_n/ua_d)*(a.b.x-a.a.x);
            let y = a.a.y + (ua_n/ua_d)*(a.b.y-a.a.y);

            LineIntersect::Intersected(x, y, 0.0)
        } else if ua_n == 0.0 && ua_d == 0.0 && ub_n == 0.0 && ub_d == 0.0 {
            LineIntersect::Coincident
        } else if ua_d == 0.0 && ub_d == 0.0 {
            LineIntersect::Parallel
        } else {
            LineIntersect::None
        }
    }

    // NOTE: this intersection uses all three xyz components composed of multiple intersect2d()
    pub fn intersect3d(&self, b: Line<f32>) -> LineIntersect<f32> {

        // most likely we only need 2 2D intersections; placed extra just in case
        let x_plane = Self::inner_intersect2d(
            Line::new(Position::new(self.a.z, self.a.y, 0.0), Position::new(self.b.z, self.b.y, 0.0)),
            Line::new(Position::new(   b.a.z,    b.a.y, 0.0), Position::new(   b.b.z,    b.b.y, 0.0))
        );
        let y_plane = Self::inner_intersect2d(
            Line::new(Position::new(self.a.x, self.a.z, 0.0), Position::new(self.b.x, self.b.z, 0.0)),
            Line::new(Position::new(   b.a.x,    b.a.z, 0.0), Position::new(   b.b.x,    b.b.z, 0.0))
        );
        let z_plane = Self::inner_intersect2d(*self, b);

        println!("Planes (xyz): {:?}", [x_plane, y_plane, z_plane]);

        LineIntersect::None
    }
}

// **************************************************************************
// Following datatypes are the basic in-game units used for many calculations
// **************************************************************************

// The intermediate results when converting an unbounded numerical unit to a bounded LocalBU
pub enum LocalBUIntermediate {
    Ok(LocalBU),  // Successfully converted into a LocalBU unit
    UBound(LocalBU),  // The value has to be modulated by chunk size
}

// TODO: using it later when domain restriction is added
pub enum DomainRestriction {
    Natural,  // 1,2,3,4,...
    Whole,  // 0,1,2,3,4,...
    Integer,  // ...,-2,-1,0,1,2,...
    Rational,  // decimals that can be represented with fractions

    // Restriction only stops add Rational since Irrational number don't truly exist in computers
    // and imaginary numbers are adding additional complexity without much of a use in game
}


// TODO: add domain restriction for units (e.g. natural numbers, whole numbers, integers, etc.)
// NOTE: DO NOT access the inner data of each unit, they are publicize for concise initialization
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct BlockUnit(pub f32);  // 1 In-Game Block Sized == 1 Meter; This is by default, should be global world.block unit
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct LocalBU(pub f32);  // Local Block Unit bounded by CHUNK_SIZE; Information will be lost through bounding
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct ChunkUnit(pub f32);  // 1 ChunkUnit = 32 BlockUnit
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct SectorUnit(pub f32);  // 1 SectorUnit = 16 ChunkUnit

// BlockUnit, a.k.a wB (world Block unit)
impl BlockUnit {
    pub fn into_chunk(self) -> ChunkUnit {
        ChunkUnit(self.0 / 32f32)
    }

    pub fn into_chunk_int(self) -> ChunkUnit {
        ChunkUnit((self.0 / 32f32).floor())
    }

    pub fn into_sector(self) -> SectorUnit {
        SectorUnit(self.0 / 32f32 / 16f32)
    }

    pub fn into_inner(self) -> f32 {
        self.0
    }

    pub fn inner(&self) -> f32 {
        self.0
    }

    pub fn into_local_bu(self) -> LocalBUIntermediate {
        if self.0 == self.0 % CHUNK_SIZE as f32 {
            LocalBUIntermediate::Ok(LocalBU(self.0))
        } else {
            LocalBUIntermediate::UBound(LocalBU(self.0))
        }
    }

    // increment
    #[inline(always)]
    pub fn incr(self) -> Self {
        Self(self.0+1.0)
    }

    // decrement
    #[inline(always)]
    pub fn decr(self) -> Self {
        Self(self.0-1.0)
    }
}

// LocalBU, a.k.a wLB (world Local Block unit)
impl LocalBU {
}

// ChunkUnit, a.k.a wC (world Chunk unit)
impl ChunkUnit {
    pub fn into_block(self) -> BlockUnit {
        BlockUnit(self.0 * CHUNK_SIZE as f32)
    }

    pub fn into_sector(self) -> SectorUnit {
        SectorUnit(self.0 / 16f32)
    }

    pub fn into_inner(self) -> f32 {
        self.0
    }

    pub fn inner(&self) -> f32 {
        self.0
    }

    // increment
    #[inline(always)]
    pub fn incr(self) -> Self {
        Self(self.0+1.0)
    }

    // decrement
    #[inline(always)]
    pub fn decr(self) -> Self {
        Self(self.0-1.0)
    }
}

// SectorUnit, a.k.a wS (world Sector unit)
impl SectorUnit {
    pub fn into_chunk(self) -> ChunkUnit {
        ChunkUnit(self.0 * 16f32)
    }

    pub fn into_block(self) -> BlockUnit {
        BlockUnit(self.0 * 16f32 * 32f32)
    }

    pub fn into_inner(self) -> f32 {
        self.0
    }

    pub fn inner(&self) -> f32 {
        self.0
    }

    // increment
    #[inline(always)]
    pub fn incr(self) -> Self {
        Self(self.0+1.0)
    }

    // decrement
    #[inline(always)]
    pub fn decr(self) -> Self {
        Self(self.0-1.0)
    }
}

impl Add for BlockUnit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for BlockUnit {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for BlockUnit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for BlockUnit {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Neg for BlockUnit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for LocalBU {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for LocalBU {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for LocalBU {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for LocalBU {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Add for ChunkUnit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for ChunkUnit {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for ChunkUnit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for ChunkUnit {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Neg for ChunkUnit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for SectorUnit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for SectorUnit {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for SectorUnit {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for SectorUnit {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Neg for SectorUnit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl From<BlockUnit> for f32 {
    fn from(itm: BlockUnit) -> Self {
        itm.0
    }
}

impl From<BlockUnit> for i32 {
    fn from(itm: BlockUnit) -> Self {
        itm.0.round() as i32
    }
}

impl From<BlockUnit> for usize {
    fn from(itm: BlockUnit) -> Self {
        itm.0.round() as usize
    }
}


impl From<LocalBU> for f32 {
    fn from(itm: LocalBU) -> Self {
        itm.0
    }
}

impl From<ChunkUnit> for f32 {
    fn from(itm: ChunkUnit) -> Self {
        itm.0
    }
}

impl From<ChunkUnit> for i64 {
    fn from(itm: ChunkUnit) -> Self {
        itm.0 as i64
    }
}

impl From<SectorUnit> for f32 {
    fn from(itm: SectorUnit) -> Self {
        itm.0
    }
}
