use std::{collections::HashSet};

use educe::Educe;
use vello::kurbo::{Rect, Vec2};
use winit::{event::{ElementState, KeyEvent, MouseButton}, keyboard::{KeyCode, PhysicalKey}};

use crate::{board::{Position, PositionError, Value}, game::GameStateState, window::Window};
use crate::render::camera::ScreenPoint;
use crate::state::State;

#[derive(Educe)]
#[educe(Default)]
pub struct InputState {
    pub mouse_position: ScreenPoint,
    pub mouse_board_position: Option<Position>,
    pub camera_drag: bool,
    pub held_keys: HashSet<KeyCode>,
}

pub fn mouse_moved(to:ScreenPoint, state:&mut State) {
    let from = state.input_state.mouse_position;
    let distance = to.to_vec2() - from.to_vec2();
    state.input_state.mouse_position = to;
    let screen_position = state.render_state.camera.point_to_world(state.input_state.mouse_position);
    state.input_state.mouse_board_position = Position::from_vec2_visible(screen_position.to_vec2() / crate::board::BOARD_SIZE + Vec2::new(0.5, 0.5), &state.game_state.board.board);
    if state.input_state.camera_drag { state.render_state.camera.drag_input(distance); }
}

pub fn mouse_input(button_state:ElementState, mouse_button:MouseButton, state:&mut State, window:&Window) {
    match mouse_button {
        MouseButton::Left => {
            if button_state == ElementState::Pressed { return }
            match state.game_state.other.state {
                GameStateState::ExpandPrompt(ref mut position) => {
                    let center = crate::ui::expand_prompt_center(window);
                    let button_rect = Rect::from_center_size(center + crate::ui::EXPAND_PROMPT_BUTTON_POS_OFFSETS.0, crate::ui::EXPAND_PROMPT_BUTTON_HALF_SIZE*2.);
                    if button_rect.contains(state.input_state.mouse_position.0) {
                        let position = std::mem::take(position);
                        state.game_state.expand_prompt_resolved(position, true);
                        state.render_state.camera.triple(window);
                    } else {
                        let button_rect = Rect::from_center_size(center + crate::ui::EXPAND_PROMPT_BUTTON_POS_OFFSETS.1, crate::ui::EXPAND_PROMPT_BUTTON_HALF_SIZE*2.);
                        if button_rect.contains(state.input_state.mouse_position.0) {
                            let position = std::mem::take(position);
                            state.game_state.expand_prompt_resolved(position, false);
                        }
                    }
                },
                GameStateState::Play => if let Some(position) = &state.input_state.mouse_board_position {
                    let (layers, None | Some(PositionError::PositionTooShallow)) = position.get_stack(&state.game_state.board.board) else { panic!() };
                    let filled = layers.iter().any(|(_, l)| l.value.is_full());

                    let (layer, None) = position.get_layer_mut(&mut state.game_state.board.board, true) else {
                        println!("ahh!!!");
                        return
                    };
                    if position.depth() == state.game_state.board.layers {
                        if layer.value == Value::Empty && position.within(&state.game_state.other.domain) && !filled {
                            state.game_state.other.place_at(layer);
                            state.game_state.placed_at(position);
                        }
                    } else { layer.expand_or_create(); }
                },
                GameStateState::End(_) => if let Some(position) = &state.input_state.mouse_board_position {
                    let (layer, None) = position.get_layer_mut(&mut state.game_state.board.board, true) else {
                        println!("ahh!!!");
                        return
                    };
                    if position.depth() < state.game_state.board.layers { layer.expand_or_create(); }
                }
            }
        },
        MouseButton::Right | MouseButton::Middle => state.input_state.camera_drag = button_state == ElementState::Pressed,
        _ => {}
    }
}

pub fn mouse_scroll(amount:Vec2, state:&mut State) {
    state.render_state.camera.scroll_input(amount.y, state.input_state.mouse_position);
}

pub fn keyboard_input(event:KeyEvent, state:&mut State, window:&Window) {
    let KeyEvent { physical_key:PhysicalKey::Code(key), state:key_state, .. } = event else { return; };
    let held_keys = &mut state.input_state.held_keys;
    match key_state {
        ElementState::Pressed => {
            held_keys.insert(key);
            match key {
                KeyCode::KeyH => {
                    state.render_state.camera.home(&window);
                },
                _ => {}
            }
        },
        ElementState::Released => {
            held_keys.remove(&key);
        }
    }
}

impl InputState {
    #[must_use]
    pub fn keyboard_input_direction(&self) -> Vec2 {
        let mut result = Vec2::ZERO;
        for (key, direction) in [
            (KeyCode::KeyW, Vec2::new(0., -1.)), (KeyCode::KeyA, Vec2::new(-1., 0.)),
            (KeyCode::KeyS, Vec2::new(0., 1.)), (KeyCode::KeyD, Vec2::new(1., 0.)),
        ] {
            result += self.held_keys.get(&key).and(Some(direction)).unwrap_or_default();
        }
        result
    }
}

