use crate::datatype::{Rotation, Dimension, CamDirection, Position, BlockUnit};
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::player::EDIT_RADIUS;
use crate::world::block::Block;
use crate::world::block::state::Matter;

use na::{
    Point3,
    Vector3,
    Transform3,
    Perspective3,
    Isometry3,
    Translation3,
};


type Matrix3D = [[f32; 4]; 4];

#[derive(Clone, PartialEq)]
pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: Rotation<f32>,  // manual pure euler angle rotation
    pub fovy: f32,
    pub zfar: f32,
    pub znear: f32,
    pub rot_speed: f32,
    pub trans_speed: f32,
}

impl Camera {
    pub fn new(rot_speed: f32, trans_speed: f32, position: Point3<f32>, rotation: Rotation<f32>) -> Self {
        Self {
            position: position,
            rotation: rotation,
            fovy: 1.3,
            rot_speed: rot_speed,
            trans_speed: trans_speed,
            zfar: 1000.0,
            znear: 0.1,
        }
    }

    // translating the camera
    pub fn translate(&mut self, vec: Transform3<f32>) {
        self.position = vec.transform_point(&self.position);
    }

    // rotating the camera
    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotation.x += x * self.rot_speed;
        self.rotation.y += y * self.rot_speed;
        self.rotation.z += z * self.rot_speed;
    }

    // translating the camera with rotation on y-axis embedded
    pub fn travel(&mut self, dir: Vec<CamDirection>) {
        let roty = Rotation::new(0.0, self.rotation.y, 0.0).matrix();

        let rot_pos = |dx, dy, dz| roty.transform_vector(&Vector3::new(dx, dy, dz));

        for d in dir {
            match d {
                CamDirection::Forward =>   {self.position = self.position + rot_pos( 0.0, 0.0, self.trans_speed)},
                CamDirection::Backward =>  {self.position = self.position + rot_pos( 0.0, 0.0,-self.trans_speed)},
                CamDirection::Leftward =>  {self.position = self.position + rot_pos(-self.trans_speed, 0.0, 0.0)},
                CamDirection::Rightward => {self.position = self.position + rot_pos( self.trans_speed, 0.0, 0.0)},
                CamDirection::Upward =>    {self.position = Translation3::new( 0.0,  self.trans_speed, 0.0).transform_point(&self.position)},
                CamDirection::Downward =>  {self.position = Translation3::new( 0.0, -self.trans_speed, 0.0).transform_point(&self.position)},
            }
        }
    }

    // generates the mvp matrix for meshes and other pipelines
    pub fn gen_mvp(&self, dimensions: Dimension<u32>) -> (Matrix3D, Matrix3D, Matrix3D) {
        let proj = Perspective3::new(dimensions.aspect() as f32, self.fovy, self.znear, self.zfar);

        let view = Isometry3::look_at_lh(&Point3::new(0.0, 0.0, 0.0), &Point3::new(0.0, 0.0, -1.0), &Vector3::new(0.0, -1.0, 0.0));

        let crd = &self.position.coords.data;
        let model = Translation3::new(crd[0], crd[1], crd[2]).to_homogeneous() * self.rotation.matrix();

        let proj_matrix = proj.as_matrix();
        let view_matrix = view.to_homogeneous();
        let model_matrix = model.try_inverse().unwrap();

        let proj_cooked: &[f32] = proj_matrix.as_slice();
        let view_cooked: &[f32] = view_matrix.as_slice();
        let model_cooked: &[f32] = model_matrix.as_slice();

        let proj_dt;
        let view_dt;
        let model_dt;

        unsafe {
            assert_eq!(proj_cooked.len(), 16);
            assert_eq!(view_cooked.len(), 16);
            assert_eq!(model_cooked.len(), 16);

            proj_dt = *(proj_cooked.as_ptr() as *const Matrix3D);
            view_dt = *(view_cooked.as_ptr() as *const Matrix3D);
            model_dt = *(model_cooked.as_ptr() as *const Matrix3D);
        }

        (proj_dt, view_dt, model_dt)
    }

    // returns a Chunk Position for breaking blocks
    pub fn raycast_break(&self, chunks: &Vec<Chunk>) -> Option<(Position<BlockUnit>, Block)> {
        let org_pos: Position<f32> = self.position.into();  // The original position
        let mut cur_pos: Position<f32> = self.position.into();  // Current position of the raycast
        let mut block_pos = None;  // <Block> position which the position of that world.block that was hit

        'ray: while block_pos == None {
            println!("R: {:?}", ((org_pos.x - cur_pos.x).powi(2) +
                (org_pos.y - cur_pos.y).powi(2) +
                (org_pos.z - cur_pos.z).powi(2)
            ).sqrt());

            // checks if the cur_pos is out of the range of the max radius
            // note: uses pythagoras theorem
            if ((org_pos.x - cur_pos.x).powi(2) +
                (org_pos.y - cur_pos.y).powi(2) +
                (org_pos.z - cur_pos.z).powi(2)
            ).sqrt() > EDIT_RADIUS as f32 {
                break 'ray;
            }

            // TODO: we can speed up the process by stripping chunks that'll never hit
            for chunk in chunks.iter() {
                let bx: f32 = chunk.position.x.into_block().into();
                let bX: f32 = (chunk.position.x.incr()).into_block().into();
                let by: f32 = chunk.position.y.into_block().into();
                let bY: f32 = (chunk.position.y.incr()).into_block().into();
                let bz: f32 = chunk.position.z.into_block().into();
                let bZ: f32 = (chunk.position.z.incr()).into_block().into();

                // check if the chunk contains the current ray position
                if  (bx <= cur_pos.x && cur_pos.x < bX) &&
                    (by <= cur_pos.y && cur_pos.y < bY) &&
                    (bz <= cur_pos.z && cur_pos.z < bZ)  {
                    // world.block raw position
                    let brx = if (cur_pos.x % CHUNK_SIZE as f32) < 0f32 {CHUNK_SIZE as f32 + (cur_pos.x % CHUNK_SIZE as f32)} else {cur_pos.x % CHUNK_SIZE as f32};
                    let bry = if (cur_pos.y % CHUNK_SIZE as f32) < 0f32 {CHUNK_SIZE as f32 + (cur_pos.y % CHUNK_SIZE as f32)} else {cur_pos.y % CHUNK_SIZE as f32};
                    let brz = if (cur_pos.z % CHUNK_SIZE as f32) < 0f32 {CHUNK_SIZE as f32 + (cur_pos.z % CHUNK_SIZE as f32)} else {cur_pos.z % CHUNK_SIZE as f32};

                    // world.block (cooked) position
                    let bx = brx.floor() as usize;
                    let by = bry.floor() as usize;
                    let bz = brz.floor() as usize;

                    // now grabbing individual blocks
                    let block = chunk.block_data[bx*CHUNK_SIZE*CHUNK_SIZE+by*CHUNK_SIZE+bz];
                    if block.state.breakable && block.state.matter == Matter::Solid {
                        block_pos = Some((
                            Position::new(
                                BlockUnit(bx as f32) + chunk.position.x.into_block(),
                                BlockUnit(by as f32) + chunk.position.x.into_block(),
                                BlockUnit(bz as f32) + chunk.position.x.into_block(),
                            ),
                            block,
                        ));
                        break 'ray;
                    }
                }
            }

            println!("Player Rotation: {:?}", self.rotation);

            // TODO: casts the ray in the direction
            cur_pos = Position::new(cur_pos.x, cur_pos.y, cur_pos.z+0.5);  // TODO: temp
        }

        block_pos
    }

    // pub fn frustum(&self, dimensions: Dimension<u32>) {
    //     let proj = Perspective3::new(dimensions.aspect() as f32, self.fovy, self.znear, self.zfar);
    //
    //     let view = Isometry3::look_at_lh(&Point3::new(0.0, 0.0, 0.0), &Point3::new(0.0, 0.0, -1.0), &Vector3::new(0.0, -1.0, 0.0));
    //
    //     let crd = &self.position.coords.data;
    //     let model = Translation3::new(crd[0], crd[1], crd[2]).to_homogeneous() * self.rotation.matrix();
    //
    //     let proj_matrix = proj.as_matrix();
    //     let view_matrix = view.to_homogeneous();
    //     let model_matrix = model.try_inverse().unwrap();
    //
    //     let mvp = proj_matrix * view_matrix * model_matrix;
    //
    //     let near_xy = mvp.transform_point(&Point3::new(-self.znear, -self.znear, self.znear)).coords.data;
    //     let near_Xy = mvp.transform_point(&Point3::new( self.znear, -self.znear, self.znear)).coords.data;
    //     let near_xY = mvp.transform_point(&Point3::new(-self.znear,  self.znear, self.znear)).coords.data;
    //     let near_XY = mvp.transform_point(&Point3::new( self.znear,  self.znear, self.znear)).coords.data;
    //
    //     let far_xy = mvp.transform_point(&Point3::new(-self.zfar, -self.zfar,  self.zfar)).coords.data;
    //     let far_Xy = mvp.transform_point(&Point3::new( self.zfar, -self.zfar,  self.zfar)).coords.data;
    //     let far_xY = mvp.transform_point(&Point3::new(-self.zfar,  self.zfar,  self.zfar)).coords.data;
    //     let far_XY = mvp.transform_point(&Point3::new( self.zfar,  self.zfar,  self.zfar)).coords.data;
    //
    //     // println!("NEAR-xy: {:?}", near_xy);
    //     // println!("NEAR-Xy: {:?}", near_Xy);
    //     // println!("NEAR-xY: {:?}", near_xY);
    //     // println!("NEAR-XY: {:?}", near_XY);
    //     //
    //     // println!("FAR-xy: {:?}", far_xy);
    //     // println!("FAR-Xy: {:?}", far_Xy);
    //     // println!("FAR-xY: {:?}", far_xY);
    //     // println!("FAR-XY: {:?}", far_XY);
    // }

    // fn line_intersection() {
    //
    // }
}
