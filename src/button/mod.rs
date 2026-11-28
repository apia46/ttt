use vello::{kurbo::Rect, peniko::Color};
use winit::event::{ElementState, MouseButton};

use crate::{input::{InputState, MouseInput}, render::{Renderer, draw::{self, TextLayout}}, space::{Context, Contexted}};

pub mod simple;

pub struct Button<T:Context> {
    pub rect: Contexted<Rect, T>,
    pub fill_color: Color,
    pub border_color: Color,
    pub text: Option<TextLayout>,
    pub disabled: bool,
    hovered: bool,
    clicked: bool,
}

pub struct NewButtonArgs<T:Context> {
    pub rect: Contexted<Rect, T>,
    pub fill_color: Color,
    pub border_color: Color,
    pub text: Option<TextLayout>,
    pub disabled: bool,
}

impl<T:Context> Button<T> {
    pub fn new(args:NewButtonArgs<T>) -> Self {
        let NewButtonArgs { rect, fill_color, border_color, text, disabled } = args;
        Button { rect, fill_color, border_color, text, disabled, hovered: false, clicked: false }
    }

    pub fn draw(&mut self, renderer:&mut Renderer, state:T::State<'_>) {
        renderer.draw_rect(draw::DrawRectArgs {
            rect: self.rect,
            style: draw::Style { stroke: Some((4., self.border_color)), fill: Some(self.fill_color) }
        }, state);
        if let Some(layout) = &self.text {
            renderer.draw_text(self.rect.center(), layout, state);
        }
    }

    /// returns whether or not this is hovered
    pub fn mouse_moved(&mut self, input_state:&InputState, state:T::State<'_>) -> bool {
        self.hovered = self.rect.contains(input_state.mouse_position.convert(state));
        if !self.hovered { self.clicked = false; }
        self.hovered
    }

    /// returns whether or not the button was triggered
    pub fn mouse_input(&mut self, input:&mut MouseInput) -> bool {
        if self.hovered && !self.disabled {
            match input {
                MouseInput { state, button: MouseButton::Left, consumed: false } => {
                    input.consumed = true;
                    match state {
                        ElementState::Pressed => { self.clicked = true; },
                        ElementState::Released => { self.clicked = false; return true; },
                    }
                },
                _ => {},
            }
        }
        false
    }
}

