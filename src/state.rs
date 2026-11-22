use std::time::{Duration, Instant};

use educe::Educe;

use crate::{board::BoardState, input::{self, InputState}, render::{RenderState, Renderer}, game::GameState};

#[derive(Educe)]
#[educe(Default)]
pub struct State {
    #[educe(Default=Instant::now())]
    pub start_time: Instant,
    pub input_state: InputState,
    pub render_state: RenderState,
    pub game_state: GameState,
}

impl State {
    pub fn process(&mut self, renderer:&mut Renderer, delta_time:f64) {
        renderer.process(&mut self.render_state, &self.input_state, delta_time);
        if self.render_state.camera.camera_moved {
            input::mouse_moved(self.input_state.mouse_position, self);
        }
    }
}

