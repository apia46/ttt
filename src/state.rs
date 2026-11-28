use std::time::Instant;

use educe::Educe;

use crate::{game::GameState, input::{self, InputState}, render::{RenderState, Renderer}, space::Camera, window::Window};

#[derive(Educe)]
#[educe(Default)]
pub struct State {
    #[educe(Default=Instant::now())]
    pub start_time: Instant,
    pub input_state: InputState,
    pub render_state: RenderState,
    pub camera: Camera,
    pub game_state: GameState,
}

impl State {
    pub fn process(&mut self, _renderer:&mut Renderer, delta_time:f64, window:&Window) {
        self.camera.process(&self.input_state, delta_time);
        if self.camera.camera_moved {
            input::mouse_moved(self.input_state.mouse_position, self, window);
        }
    }
}

