use vello::kurbo::Vec2;
use vello::peniko::Color;
use winit::event::{ElementState, MouseButton};

use crate::render::draw::BuildTextArgs;
use crate::render::Renderer;

use crate::state::State;
#[derive(Default)]
pub struct HoverState {
    // sticky; doesnt clear if you stop hovering
}

pub fn update(state: &mut State) {
    update_hover(state);
}

pub fn mouse_moved(state:&mut State) {
    update_hover(state);
}

pub fn mouse_input(button_state:ElementState, mouse_button:MouseButton, state:&mut State) {
    if let (ElementState::Pressed, MouseButton::Left) = (button_state, mouse_button) {
    }
}

#[derive(Debug)]
pub enum HoverInfo {
    Heading(String),
    Body(String),
}

pub fn update_hover(state:&mut State) {

}

fn text_line(string:&str, font_size:f32, color:Color, renderer:&mut Renderer, pos:&mut Vec2) {
    let text = renderer.build_text(BuildTextArgs { text: &string, font_size, color, center: false });
    renderer.draw_text(*pos, &text, None);
    pos.y += text.height() as f64;
}

impl HoverState {
    pub fn draw(&self, renderer:&mut Renderer) {
        let mut pos:Vec2 = Vec2::new(20., 20.,);
    }
}

