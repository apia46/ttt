
use winit::event::{ElementState, MouseButton};

use crate::input::{MouseInputResponse, UserState};

#[derive(Default)]
pub struct InputState {
    pub user_state: UserState,
}

impl InputState {
    pub fn update(&mut self) {
    }

    #[must_use]
    pub fn mouse_input(&mut self, button_state:ElementState, mouse_button:MouseButton) -> MouseInputResponse {
        let mut response = MouseInputResponse::default();
        response
    }
}

