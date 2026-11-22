use educe::Educe;
use macros::parse_color;
use parley::{FontContext, LayoutContext};
use vello::{Scene, kurbo::{Point, Rect, Vec2}, peniko::{Color, Gradient, ImageBrush, ImageData}};

pub const BG_COLOR:Color = parse_color!("#161e4e");

use crate::{board::{BoardState, Value}, game::GameState, window::Window};
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
    pub fn draw_scene(&mut self, state:&RenderState, input_state:&InputState, game_state:&GameState, window:&Window) -> &Scene {
        self.scene.reset();
        for (value, s) in game_state.board.iter_terminal_visible(
            || DrawSceneState {
                    matching_mouse_position: input_state.mouse_board_position.is_some(),
                    ..Default::default()
                },
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
            let rect = Rect::from_points(s.pos - half_size, s.pos + half_size);
            self.draw_rect(draw::DrawRectArgs {
                rect, style: draw::Style { stroke: Some((s.scale * 0.05, Color::WHITE)), fill: s.matching_mouse_position.then_some(parse_color!("#bbbbbb")) },
            }, &state.camera);
            if let Some(image) = value.image() {
                self.draw_image(draw::DrawImageArgs { rect, image, alpha: 1.0 }, Some(&state.camera));
            }
        }
        self.draw_text_immediate(Vec2::new(20., 20.), draw::BuildTextArgs {
            text: &format!("turn: {:?}", game_state.other.turn), font_size: 100., color: Color::BLACK, center: false
        }, None);
        &self.scene
    }
}

