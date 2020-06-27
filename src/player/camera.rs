use crate::datatype::{Rotation, Dimension, CamDirection};

use na::{
    Point3,
    Vector3,
    Transform3,
    Perspective3,
    Isometry3,
    Translation3,
};


type Matrix3D = [[f32; 4]; 4];

pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: Rotation<f32>,  // manual pure euler angle rotation
    pub fovy: f32,
    rot_speed: f32,
    trans_speed: f32,
}

impl Camera {
    pub fn new(rot_speed: f32, trans_speed: f32) -> Self {
        Self {
            position: Point3::origin(),
            rotation: Rotation::new(0.0, 0.0, 0.0),
            fovy: 3.14 / 2.0,
            rot_speed: rot_speed,
            trans_speed: trans_speed,
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
        let proj = Perspective3::new(dimensions.aspect() as f32, self.fovy, 0.1, 1000.0);

        let view = Isometry3::look_at_lh(&Point3::new(0.0, 0.0, 0.0), &Point3::new(0.0, 0.0, -1.0), &Vector3::new(0.0, -1.0, 0.0));

        let crd = &self.position.coords.data;
        let model = Translation3::new(crd[0], crd[1], crd[2]);
        let model = model.to_homogeneous() * self.rotation.matrix();

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
}
