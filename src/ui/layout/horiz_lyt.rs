use super::super::{Widget, Layout, ObjType, Context};
use crate::datatype as dt;
use winit::event::Event;

pub struct HorizontalLayout {
    position: dt::Position<f32>,
    objects: Vec<&'static ObjType<dyn Widget, dyn Layout>>,
}

impl HorizontalLayout {
    pub fn new() -> Self {
        Self {
            position: dt::Position::new(0.0, 0.0, 0.0),
            objects: Vec::new(),
        }
    }
}

impl<'a> Widget for HorizontalLayout {
    fn update(&mut self, e: &Event<()>) {
        unimplemented!()
    }

    fn render(&self, ctx: &mut Context) {
        ctx.add_square();
        unimplemented!()
    }
}

impl Layout for HorizontalLayout {
    fn add_widget(&mut self) {
        unimplemented!()
    }

    fn add_layout(&mut self) {
        unimplemented!()
    }

    fn remove_layout(&mut self) {
        unimplemented!()
    }

    fn remove_widget(&mut self) {
        unimplemented!()
    }
}
