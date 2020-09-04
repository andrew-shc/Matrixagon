use super::super::{Widget, Layout, ObjType, Context};
use crate::datatype as dt;

use winit::event::Event;

/*
Stack Layout
------------
A stack machine based layout to push or pop screens.
 */

pub struct StackLayout {
    position: dt::Position<f32>,
    objects: Vec<&'static ObjType<dyn Widget, dyn Layout>>,
}

impl StackLayout {
    pub fn new() -> Self {
        Self {
            position: dt::Position::new(0.0, 0.0, 0.0),
            objects: Vec::new(),
        }
    }
}

impl<'a> Widget for StackLayout {
    fn update(&mut self, e: &Event<()>) {
        unimplemented!()
    }

    fn render(&self, ctx: &mut Context) {
        ctx.add_square();
        unimplemented!()
    }
}

impl Layout for StackLayout {
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
