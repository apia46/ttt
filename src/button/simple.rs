use vello::{kurbo::Rect, peniko::Color};

use crate::{button::{Button, NewButtonArgs}, input::{InputState, MouseInput}, render::{Renderer, draw}, space::{Context, Contexted}};

/// a simple button that makes its text the first time its rendered
pub struct SimpleButton<T:Context> {
    inner: Button<T>,
    text: String,
    text_color: Color,
    font_size: f32,
}

pub struct NewSimpleButtonArgs<T:Context> {
    pub rect: Contexted<Rect, T>,
    pub fill_color: Color,
    pub border_color: Color,
    pub text: String,
    pub text_color: Color,
    pub font_size: f32,
    pub disabled: bool,
}

impl<T:Context> SimpleButton<T> {
    pub fn new(args:NewSimpleButtonArgs<T>) -> Self {
        let NewSimpleButtonArgs { rect, fill_color, border_color, text, text_color, font_size,  disabled } = args;
        SimpleButton {
            inner: Button::new(NewButtonArgs { rect, fill_color, border_color, text: None, disabled}), text, text_color, font_size
        }
    }

    pub fn draw(&mut self, renderer:&mut Renderer, state:T::State<'_>) {
        if self.inner.text.is_none() {
            self.inner.text = Some(renderer.build_text(draw::BuildTextArgs { text: &self.text, font_size: self.font_size, color: self.text_color, center: true }))
        }
        self.inner.draw(renderer, state);
    }

    pub fn mouse_moved(&mut self, input_state:&InputState, state:T::State<'_>) -> bool {
        self.inner.mouse_moved(input_state, state)
    }

    pub fn mouse_input(&mut self, input:&mut MouseInput) -> bool {
        self.inner.mouse_input(input)
    }
}

