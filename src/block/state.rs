#[derive(Copy, Clone)]
pub struct BlockState {
    pub transparent: bool,
}

impl Default for BlockState {
    fn default() -> Self {
        Self {
            transparent: false,
        }
    }
}
