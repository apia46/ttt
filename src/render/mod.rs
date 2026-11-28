use educe::Educe;
use macros::parse_color;
use parley::{FontContext, LayoutContext};
use vello::{Scene, kurbo::{Point, Rect, Size, Vec2}, peniko::{Color, Gradient, ImageBrush, ImageData}};
use vello_common::color::HueDirection;

pub const BG_COLOR:Color = parse_color!("#161e4e");

use crate::{game::{GameState, GameStateState, Player}, space::{Camera, Screen, Space, World, anchor::{Anchor, BottomMiddle}}, window::Window};
use crate::input::InputState;

mod state;
pub mod draw;

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
    pos: World<Point>,
    layer: usize,
    #[educe(Default=true)]
    matching_mouse_position: bool,
    #[educe(Default=true)]
    in_domain: bool,
    filled: Option<Option<Player>>,
}

impl Renderer {
    #[must_use]
    pub fn draw_scene(&mut self, _state:&RenderState, input_state:&InputState, game_state:&mut GameState, camera:&Camera, window:&Window) -> &Scene {
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
            let half_size = Vec2::new(0.5, 0.5).wrapped() * s.scale;
            let rect = World::<Rect>::from_points(s.pos - half_size, s.pos + half_size);
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
            }, &camera);
            if let Some(image) = value.image() {
                self.draw_image(draw::DrawImageArgs { rect, image, alpha: 1.0 }, &camera);
            }
        }
        match game_state.other.state {
            GameStateState::Play => {
                self.draw_text_immediate(Screen::<Point>::new(20., 20.), draw::BuildTextArgs {
                    text: &format!("turn: {:?}", game_state.other.turn), font_size: 100., color: Player::color(Some(game_state.other.turn)), center: false
                }, ());
            },
            GameStateState::ExpandPrompt(ref mut prompt) => {
                self.draw_rect(draw::DrawRectArgs {
                    rect: Anchor::<Rect, BottomMiddle>::from_center_size(Point::new(0., -50.).wrapped(), Size::new(300., 100.).wrapped()),
                    style: draw::Style { stroke: Some((8., parse_color!("#668899"))), fill: Some(parse_color!("#223344")) }
                }, window);
                self.draw_text_immediate(Anchor::<Point, BottomMiddle>::new(0., -70.), draw::BuildTextArgs {
                    text: "Expand?", font_size: 20., color: Color::WHITE, center: true
                }, window);
                prompt.yes_button.draw(self, window);
                prompt.no_button.draw(self, window);
            },
            GameStateState::End(result) => {
                let text = match result {
                    None => "Tie!",
                    Some(Player::X) => "Player X win!",
                    Some(Player::O) => "Player O win!",
                };
                self.draw_text_immediate(World::<Point>::new(0., -500.), draw::BuildTextArgs {
                    text, font_size: 100., color: Player::color(result), center: true
                }, &camera);
            }
        }
        game_state.other.home_button.draw(self, window);
        &self.scene
    }
}

