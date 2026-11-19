use std::time::{Duration, Instant};

use educe::Educe;

use crate::{hover::{self, HoverState}, input::{self, InputState}, render::{RenderState, Renderer}};

#[derive(Educe)]
#[educe(Default)]
pub struct State {
    #[educe(Default=Instant::now())]
    pub start_time: Instant,
    pub previous_tick: Duration,
    pub input_state: InputState,
    pub render_state: RenderState,
    pub hover_state: HoverState,
}

impl State {
    pub fn update(&mut self, renderer:&mut Renderer, delta_time:f64) {
        self.input_state.update();
        hover::update(self);
        renderer.update(&mut self.render_state, &self.input_state);
    }

    pub fn process(&mut self, renderer:&mut Renderer, delta_time:f64) {
        renderer.process(&mut self.render_state, &self.input_state, delta_time);
        if self.render_state.camera.camera_moved {
            input::mouse_moved(self.input_state.user_state.mouse_position, self);
        }
    }
}

