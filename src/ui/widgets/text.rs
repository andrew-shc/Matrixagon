use crate::ui::{Widget, Context};
use winit::event::Event;

struct Text {
    text: String
}

impl Widget for Text {
    fn update(&mut self, e: &Event<()>) {
        unimplemented!()
    }

    fn render(&self, ctx: &mut Context) {
        unimplemented!()
    }
}
