use macros::parse_color;
use parley::{FontContext, LayoutContext};
use vello::{Scene, peniko::{Color, Gradient, ImageBrush, ImageData}};

pub const BG_COLOR:Color = parse_color!("#161e4e");

use crate::{board::BoardState, window::Window};
use crate::input::InputState;

mod state;
pub mod draw;
pub mod camera;

pub use state::RenderState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorBrush(Color);

impl Default for ColorBrush {
    fn default() -> Self {
        ColorBrush(Color::BLACK)
    }
}

impl From<ColorBrush> for vello::peniko::Brush<ImageBrush<&ImageData>, &Gradient> {
    fn from(ColorBrush(value): ColorBrush) -> Self {
        value.into()
    }
}

#[derive(Default)]
pub struct Renderer {
    scene: Scene,
    font_cx: FontContext,
    layout_cx: LayoutContext<ColorBrush>,
}

impl Renderer {
    pub fn process(&mut self, render_state: &mut RenderState, input_state:&InputState, delta_time:f64) {
        render_state.camera.process(&input_state.user_state, delta_time);
    }

    #[must_use]
    pub fn draw_scene(&mut self, state:&RenderState, board_state:&BoardState, window:&Window) -> &Scene {
        self.scene.reset();
        &self.scene
    }
}

