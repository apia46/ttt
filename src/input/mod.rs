use std::{collections::HashSet};

use educe::Educe;
use vello::kurbo::{Point, Vec2};
use winit::{event::{ElementState, KeyEvent, MouseButton}, keyboard::{KeyCode, PhysicalKey}};

use crate::{board::{Position, PositionError, Value}, game::GameStateState, space::{Screen, Space}, window::Window};
use crate::state::State;

pub struct MouseInput {
    pub state: ElementState,
    pub button: MouseButton,
    pub consumed: bool,
}

impl MouseInput {
    pub fn new(state:ElementState, button:MouseButton) -> Self {
        MouseInput { state, button, consumed: false }
    }
}


#[derive(Educe)]
#[educe(Default)]
pub struct InputState {
    pub mouse_position: Screen<Point>,
    pub mouse_board_position: Option<Position>,
    pub camera_drag: bool,
    pub held_keys: HashSet<KeyCode>,
}

pub fn mouse_moved(to:Screen<Point>, state:&mut State, window:&Window) {
    let from = state.input_state.mouse_position;
    let distance = to.to_vec2() - from.to_vec2();
    state.input_state.mouse_position = to;
    let world_position = state.input_state.mouse_position.convert(&state.camera);
    state.input_state.mouse_board_position = Position::from_vec2_visible(world_position.to_vec2() / crate::board::BOARD_SIZE + Vec2::new(0.5, 0.5).wrapped(), &state.game_state.board.board);
    if state.input_state.camera_drag { state.camera.drag_input(distance); }
    state.game_state.other.home_button.mouse_moved(&state.input_state, window);
    match state.game_state.other.state {
        GameStateState::Play => {},
        GameStateState::ExpandPrompt(ref mut prompt) => {
            prompt.yes_button.mouse_moved(&state.input_state, window);
            prompt.no_button.mouse_moved(&state.input_state, window);
        },
        GameStateState::End(_) => {},
    }
}

pub fn mouse_input(mut input:MouseInput, state:&mut State, window:&Window) {
    if state.game_state.other.home_button.mouse_input(&mut input) {
        state.camera.home(&window);
    }
    match input.button {
        MouseButton::Left => {
            match state.game_state.other.state {
                GameStateState::ExpandPrompt(ref mut prompt) => {
                    let yes_pressed = prompt.yes_button.mouse_input(&mut input);
                    let no_pressed = prompt.no_button.mouse_input(&mut input);
                    if yes_pressed {
                        let position = std::mem::take(&mut prompt.expand_position);
                        state.game_state.expand_prompt_resolved(position, true);
                        state.camera.triple(window);
                    } else if no_pressed {
                        let position = std::mem::take(&mut prompt.expand_position);
                        state.game_state.expand_prompt_resolved(position, false);
                    }
                },
                GameStateState::Play => if let Some(position) = &state.input_state.mouse_board_position {
                    if input.state == ElementState::Pressed { return }
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
                    if input.state == ElementState::Pressed { return }
                    let (layer, None) = position.get_layer_mut(&mut state.game_state.board.board, true) else {
                        println!("ahh!!!");
                        return
                    };
                    if position.depth() < state.game_state.board.layers { layer.expand_or_create(); }
                }
            }
            mouse_moved(state.input_state.mouse_position, state, window);
        },
        MouseButton::Right | MouseButton::Middle => state.input_state.camera_drag = input.state == ElementState::Pressed,
        _ => {}
    }
}

pub fn mouse_scroll(amount:Vec2, state:&mut State) {
    state.camera.scroll_input(amount.y, state.input_state.mouse_position);
}

pub fn keyboard_input(event:KeyEvent, state:&mut State, window:&Window) {
    let KeyEvent { physical_key:PhysicalKey::Code(key), state:key_state, .. } = event else { return; };
    let held_keys = &mut state.input_state.held_keys;
    match key_state {
        ElementState::Pressed => {
            held_keys.insert(key);
            match key {
                KeyCode::KeyH => {
                    state.camera.home(&window);
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

