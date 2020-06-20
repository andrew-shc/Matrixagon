use crate::datatype::{Position, Rotation, Dimension, CamDirection};

use cgmath::{perspective, Matrix4, Vector3, Point3, SquareMatrix, Deg, Rad};
use cgmath::num_traits::real::Real;


pub struct Camera {
    pub position: Position<f32>,
    pub rotation: Rotation<Deg<f32>>,
    rot_speed: f32,
    trans_speed: f32,
    fov: Deg<f32>,
}

impl Camera {
    pub fn new(rot_speed: f32, trans_speed: f32) -> Self {
        Self {
            position: Position::default(),
            rotation: Rotation::default(),
            rot_speed: rot_speed,
            trans_speed: trans_speed,
            fov: Deg(60.0),
        }
    }

    // translating the camera
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.position.x += x * self.trans_speed;
        self.position.y += y * self.trans_speed;
        self.position.z += z * self.trans_speed;
    }

    // rotating the camera
    pub fn rotate(&mut self, x: Deg<f32>, y: Deg<f32>, z: Deg<f32>) {
        self.rotation.x += x * self.rot_speed;
        self.rotation.y += y * self.rot_speed;
        self.rotation.z += z * self.rot_speed;
    }

    // translating the camera with rotation on y-axis embedded
    pub fn travel(&mut self, dir: Vec<CamDirection>) {
        for d in dir {
            match d {
                CamDirection::Forward =>   {self.translate(-Rad::from(self.rotation.y).0.cos(), 0.0, -Rad::from(self.rotation.y).0.sin())},
                CamDirection::Backward =>  {self.translate( Rad::from(self.rotation.y).0.cos(), 0.0,  Rad::from(self.rotation.y).0.sin())},
                CamDirection::Leftward =>  {self.translate(-Rad::from(self.rotation.y).0.sin(), 0.0,  Rad::from(self.rotation.y).0.cos())},
                CamDirection::Rightward => {self.translate( Rad::from(self.rotation.y).0.sin(), 0.0, -Rad::from(self.rotation.y).0.cos())},
                CamDirection::Upward =>    {self.translate(0.0, 1.0, 0.0)},
                CamDirection::Downward =>  {self.translate(0.0, -1.0, 0.0)},
                CamDirection::None => {},
            }
        }
    }

    // generates the mvp matrix for meshes and other pipelines
    pub fn gen_mvp(&self, dimensions: Dimension<u32>) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
        let proj = perspective (self.fov, dimensions.aspect() as f32, 0.1 , 1000.0);
        let view = Matrix4::from_angle_x(self.rotation.x) * Matrix4::from_angle_y(self.rotation.y) *
            Matrix4::look_at(Point3::new(self.position.x, self.position.y, 1.0+self.position.z), self.position.into(), Vector3::new(0.0, -1.0, 0.0));
        let world = Matrix4::identity();

        (proj, view, world)
    }
}
