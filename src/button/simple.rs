use vello::{kurbo::Rect, peniko::Color};

use crate::{button::{Button, NewButtonArgs}, input::{InputState, MouseInput}, render::{Renderer, draw}, space::{Context, Contexted}};

pub struct Style {
    pub fill_color: Color,
    pub hover_fill_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub font_size: f32,
}

/// a simple button that makes its text the first time its rendered
pub struct SimpleButton<T:Context> {
    inner: Button<T>,
    text: String,
    style: &'static Style,
}

pub struct NewSimpleButtonArgs<T:Context> {
    pub rect: Contexted<Rect, T>,
    pub text: String,
    pub style: &'static Style,
    pub disabled: bool,
}

impl<T:Context> SimpleButton<T> {
    pub fn new(args:NewSimpleButtonArgs<T>) -> Self {
        let NewSimpleButtonArgs { rect, text, style,  disabled } = args;
        SimpleButton {
            inner: Button::new(NewButtonArgs { rect, fill_color: style.fill_color, border_color: style.border_color, text: None, disabled}), text, style, 
        }
    }

    pub fn draw(&mut self, renderer:&mut Renderer, state:T::State<'_>) {
        if self.inner.text.is_none() {
            self.inner.text = Some(renderer.build_text(draw::BuildTextArgs { text: &self.text, font_size: self.style.font_size, color: self.style.text_color, center: true }))
        }
        self.inner.draw(renderer, state);
    }

    pub fn mouse_moved(&mut self, input_state:&InputState, state:T::State<'_>) -> bool {
        let hovered = self.inner.mouse_moved(input_state, state);
        self.inner.fill_color = if hovered { self.style.hover_fill_color } else { self.style.fill_color };
        hovered
    }

    pub fn mouse_input(&mut self, input:&mut MouseInput) -> bool {
        self.inner.mouse_input(input)
    }
}

