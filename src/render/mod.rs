use educe::Educe;
use macros::parse_color;
use parley::{FontContext, LayoutContext};
use vello::{Scene, kurbo::{Point, Rect, Vec2}, peniko::{Color, Gradient, ImageBrush, ImageData}};
use vello_common::color::HueDirection;

pub const BG_COLOR:Color = parse_color!("#161e4e");

use crate::{board::{BoardState, Value}, game::{GameState, GameStateState, Player}, window::Window};
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
    #[educe(Default=true)]
    in_domain: bool,
    filled: Option<Option<Player>>,
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
                    in_domain: !matches!(game_state.other.state, GameStateState::End(_)),
                    ..Default::default()
                },
            |prev, direction, layer| {
                let scale = prev.scale / 3.;
                let matching_mouse_position = prev.matching_mouse_position && input_state.mouse_board_position.as_ref().is_some_and(|p| p.matches_at(direction, prev.layer));
                let in_domain = prev.in_domain && (game_state.other.domain.depth() <= prev.layer || game_state.other.domain.matches_at(direction, prev.layer));
                let filled = prev.filled.or_else(|| layer.value.into());
                DrawSceneState {
                    scale, pos: prev.pos + direction * scale, layer: prev.layer + 1,
                    matching_mouse_position, in_domain, filled
                }
            }
        ) {
            let half_size = Vec2::new(0.5, 0.5) * s.scale;
            let rect = Rect::from_points(s.pos - half_size, s.pos + half_size);
            let mut fill = match (s.in_domain, s.filled) {
                (true, None) => parse_color!("#ffffff44"),
                (_, Some(player)) => Player::color(player).with_alpha(0.4),
                (false, None) => parse_color!("#0000"),
            };
            if s.matching_mouse_position {
                fill = fill.lerp(Color::WHITE, 0.2, HueDirection::Shorter);
            }
            self.draw_rect(draw::DrawRectArgs {
                rect, style: draw::Style { stroke: Some((s.scale * 0.05, Color::WHITE)), fill: Some(fill) },
            }, Some(&state.camera));
            if let Some(image) = value.image() {
                self.draw_image(draw::DrawImageArgs { rect, image, alpha: 1.0 }, Some(&state.camera));
            }
        }
        match game_state.other.state {
            GameStateState::Play => {
                self.draw_text_immediate(Vec2::new(20., 20.), draw::BuildTextArgs {
                    text: &format!("turn: {:?}", game_state.other.turn), font_size: 100., color: Player::color(Some(game_state.other.turn)), center: false
                }, None);
            },
            GameStateState::ExpandPrompt(_) => {
                let center = crate::ui::expand_prompt_center(window);
                self.draw_rect(draw::DrawRectArgs {
                    rect: Rect::from_center_size(center, crate::ui::EXPAND_PROMPT_HALF_SIZE*2.),
                    style: draw::Style { stroke: Some((8., parse_color!("#668899"))), fill: Some(parse_color!("#223344")) }
                }, None);
                self.draw_text_immediate(center.to_vec2() + Vec2::new(0., -20.), draw::BuildTextArgs {
                    text: "Expand?", font_size: 20., color: Color::WHITE, center: true
                }, None);

                let button_pos = center + crate::ui::EXPAND_PROMPT_BUTTON_POS_OFFSETS.0;
                self.draw_rect(draw::DrawRectArgs {
                    rect: Rect::from_center_size(button_pos, crate::ui::EXPAND_PROMPT_BUTTON_HALF_SIZE*2.),
                    style: draw::Style { stroke: Some((4., parse_color!("#66997a"))), fill: Some(parse_color!("#223344")) }
                }, None);
                self.draw_text_immediate(button_pos.to_vec2(), draw::BuildTextArgs {
                    text: "Yes", font_size: 20., color: Color::WHITE, center: true
                }, None);

                let button_pos = center + crate::ui::EXPAND_PROMPT_BUTTON_POS_OFFSETS.1;
                self.draw_rect(draw::DrawRectArgs {
                    rect: Rect::from_center_size(button_pos, crate::ui::EXPAND_PROMPT_BUTTON_HALF_SIZE*2.),
                    style: draw::Style { stroke: Some((4., parse_color!("#854747"))), fill: Some(parse_color!("#223344")) }
                }, None);
                self.draw_text_immediate(button_pos.to_vec2(), draw::BuildTextArgs {
                    text: "No", font_size: 20., color: Color::WHITE, center: true
                }, None);
            },
            GameStateState::End(result) => {
                let text = match result {
                    None => "Tie!",
                    Some(Player::X) => "Player X win!",
                    Some(Player::O) => "Player O win!",
                };
                self.draw_text_immediate(Vec2::new(0., -500.), draw::BuildTextArgs {
                    text, font_size: 100., color: Player::color(result), center: true
                }, Some(&state.camera));
            }
        }
        &self.scene
    }
}

