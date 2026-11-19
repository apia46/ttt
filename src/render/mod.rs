use std::time::Duration;

use educe::Educe;
use macros::parse_color;
use parley::{FontContext, LayoutContext};
use vello::{Scene, peniko::{Color, Gradient, ImageBrush, ImageData}};

pub const BG_COLOR:Color = parse_color!("#161e4e");

use crate::{hover::HoverState, window::Window};
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

#[derive(Educe)]
#[educe(Default)]
pub struct RendererState {
    pub previous_frame: Duration,
    pub delta_time: f64,
    #[educe(Default=Duration::from_secs_f64(1./60.))]
    pub refresh_rate: Duration,
}

impl Renderer {
    pub fn update(&mut self, render_state: &mut RenderState, input_state:&InputState) {
    }

    pub fn process(&mut self, render_state: &mut RenderState, input_state:&InputState, delta_time:f64) {
        render_state.camera.process(&input_state.user_state, delta_time);
    }

    #[must_use]
    pub fn draw_scene(&mut self, state:&RenderState, window:&Window, hover_state:&HoverState) -> &Scene {
        self.scene.reset();
        hover_state.draw(self);
        &self.scene
    }
}

