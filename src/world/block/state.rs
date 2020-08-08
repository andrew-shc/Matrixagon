#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Matter {
    Liquid,  // passes the raycast-break/place test
    Solid,  // specifies if its breakable
    Gas,  // usually not interactable
}

// TODO: add seriliaziation in near future
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlockState {
    pub matter: Matter,
    pub transparent: bool,
    pub placeable: bool,
    pub breakable: bool,
}

impl Default for BlockState {
    fn default() -> Self {
        Self {
            matter: Matter::Solid,
            transparent: false,
            placeable: true,
            breakable: true,
        }
    }
}
