use winit::{error::EventLoopError, event_loop::ActiveEventLoop};

use crate::app::App;
use crate::window::Window;
use crate::render::Renderer;
use crate::state::State;

mod util;
mod board;

mod app;
mod window;

mod render;
mod input;
mod state;
mod game;
mod ui;

struct Game {
    window: Window,
    renderer: Renderer,
    state: State,
}

impl Game {
    #[must_use]
    async fn new(event_loop:&ActiveEventLoop) -> Game {
        Game {
            window: Window::new(event_loop).await,
            renderer: Renderer::default(),
            state: State::default(),
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    env_logger::init();
    App::new().run()
}

