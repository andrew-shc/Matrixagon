pub trait VertexType {}
pub trait IndexType {}

impl IndexType for u32 {}  // DEPRECATED: will be removing the index type in favor of vulkano's index


#[derive(Default, Copy, Clone, Debug)]
pub struct UIVert {
    pub pos: [f32; 2],  // 2D position
    pub col: [f32; 4],  // RGBA colors
}

#[derive(Default, Copy, Clone, Debug)]
pub struct CubeVert {
    pub pos: [f32; 3],  // 3D position
    pub ind: u32,  // Texture array index
    pub txtr: [u32; 2],  // TODO: try boolean, because there are only 1.0 and 0.0
}

vulkano::impl_vertex!(UIVert, pos, col);
vulkano::impl_vertex!(CubeVert, pos, ind, txtr);

impl VertexType for UIVert {}
impl VertexType for CubeVert {}


pub mod ui_simpl_vs { vulkano_shaders::shader!{ty: "vertex", path: "resource/shaders/ui.vert",} }
pub mod ui_simpl_fs { vulkano_shaders::shader!{ty: "fragment", path: "resource/shaders/ui.frag",} }

// texture array should be static relative to program
pub mod cube_vs { vulkano_shaders::shader!{ty: "vertex", path: "resource/shaders/cube.vert",} }
pub mod cube_fs { vulkano_shaders::shader!{ty: "fragment", path: "resource/shaders/cube.frag",} }
