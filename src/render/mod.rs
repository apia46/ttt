use educe::Educe;
use macros::parse_color;
use parley::{FontContext, LayoutContext};
use vello::{Scene, kurbo::{Point, Rect, Vec2}, peniko::{Color, Gradient, ImageBrush, ImageData}};

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

#[derive(Clone, Educe)]
#[educe(Default)]
struct DrawSceneState {
    #[educe(Default=crate::board::BOARD_SIZE)]
    scale: f64,
    pos: Point,
    layer: usize,
    #[educe(Default=true)]
    matching_mouse_position: bool,
}

impl Renderer {
    pub fn process(&mut self, render_state: &mut RenderState, input_state:&InputState, delta_time:f64) {
        render_state.camera.process(&input_state, delta_time);
    }

    #[must_use]
    pub fn draw_scene(&mut self, state:&RenderState, input_state:&InputState, board_state:&BoardState, window:&Window) -> &Scene {
        self.scene.reset();
        for (value, s) in board_state.iter_terminal_visible(
                DrawSceneState::default,
            |prev, direction| {
                let scale = prev.scale / 3.;
                let matching_mouse_position = prev.matching_mouse_position && input_state.mouse_board_position.as_ref().is_some_and(|p| p.matches_at(direction, prev.layer));
                DrawSceneState {
                    scale, pos: prev.pos + direction * scale, layer: prev.layer + 1,
                    matching_mouse_position,
                }
            }
        ) {
            let half_size = Vec2::new(0.5, 0.5) * s.scale;
            self.draw_rect(draw::DrawRectArgs {
                rect: Rect::from_points(s.pos - half_size, s.pos + half_size),
                style: draw::Style { stroke: Some((s.scale * 0.05, Color::WHITE)), fill: s.matching_mouse_position.then_some(parse_color!("#bbbbbb")) },
            }, &state.camera);
        }
        &self.scene
    }
}

