use educe::Educe;
use vello::kurbo::{Affine, Point, Vec2};

use crate::{input::InputState, space::{Context, Contexted, Screen, Space}, util::Lerp, window::Window};

const CAMERA_SPEED:f64 = 30000.;
const VELOCITY_DECAY:f64 = 1.5e-6;

#[derive(Educe)]
#[educe(Default)]
pub struct Camera {
    pub pos: World<Point>,
    state: CameraState,
    #[educe(Default=1)]
    scale: f64,
    pub camera_moved: bool,
}

#[derive(Educe)]
#[educe(Default)]
enum CameraState {
    #[educe(Default)]
    Normal(NormalCameraState),
    Homing(HomingCameraState),
}

#[derive(Educe)]
#[educe(Default)]
struct NormalCameraState {
    velocity: World<Vec2>,
    zoom_point: Screen<Point>,
    #[educe(Default=1)]
    target_scale: f64,
}

#[derive(Default)]
struct HomingCameraState {
    target_pos: World<Point>,
    target_scale: f64,
}

#[derive(Clone, Copy, PartialEq)]
pub struct WorldContext;

pub type World<T> = Contexted<T, WorldContext>;

impl Context for WorldContext {
    type State<'a> = &'a Camera;

    fn transform_to<'a>(state:Self::State<'a>) -> Affine {
        state.transform_to_world()
    }

    fn transform_from<'a>(state:Self::State<'a>) -> Affine {
        Self::transform_to(state).inverse()
    }
}

impl Camera {
    #[must_use]
    fn transform_to_world(&self) -> Affine {
        Affine::scale(self.scale).then_translate(self.pos.to_vec2().inner)
    }

    #[must_use]
    fn default_pos(&self, window:&Window) -> World<Point> {
        // (at the default scale)
        (window.size * -0.5).to_point().cast()
    }

    pub fn go_to_default_pos(&mut self, window:&Window) {
        self.cancel_homing();
        self.pos = self.default_pos(window);
    }

    pub fn process(&mut self, input_state:&InputState, delta_time:f64) {
        let adjusted_decay = VELOCITY_DECAY.powf(delta_time);

        if input_state.keyboard_input_direction() != Vec2::ZERO { self.cancel_homing(); };

        let old_pos = self.pos;

        match &mut self.state {
            CameraState::Normal(state) => {
                state.velocity += (input_state.keyboard_input_direction() * CAMERA_SPEED * delta_time).wrapped();
                let new_scale = (1.-adjusted_decay).lerp(self.scale, state.target_scale);
                // zoom correction: the mouse should be at the same world position before and after the zoom
                // mouse * scale_i + pos_i = mouse * scale_f + pos_f
                // mouse * (scale_i-scale_f) + pos_i = pos_f
                // zoom_correction = mouse * (scale_i-scale_f)
                let zoom_correction = (state.zoom_point.to_vec2() * (self.scale - new_scale)).cast();
                state.velocity *= adjusted_decay;
                self.pos += state.velocity * delta_time + zoom_correction;
                self.scale = new_scale;
            },
            CameraState::Homing(state) => {
                self.pos = (1.-adjusted_decay).lerp(self.pos, state.target_pos);
                self.scale = (1.-adjusted_decay).lerp(self.scale, state.target_scale);
                if self.pos.distance_squared(state.target_pos) < 1.
                    && (self.scale - state.target_scale).abs() < 0.01 {
                    self.pos = state.target_pos;
                    self.state = CameraState::Normal(NormalCameraState { target_scale: state.target_scale, ..Default::default() });
                }
            },
        }

        self.camera_moved = old_pos != self.pos;
    }

    fn cancel_homing(&mut self) -> Option<&mut NormalCameraState> {
        match &mut self.state {
            CameraState::Normal(_) => {},
            CameraState::Homing(_) => { self.state = CameraState::Normal(NormalCameraState { target_scale: self.scale, ..Default::default() }); },
        }
        if let CameraState::Normal(state) = &mut self.state { Some(state) } else { None }
    }

    pub fn scroll_input(&mut self, amount:f64, mouse_position:Screen<Point>) {
        let Some(state) = self.cancel_homing() else { return; };
        state.zoom_point = mouse_position;
        state.target_scale *= 2_f64.powf(amount*-0.02);
        state.target_scale = state.target_scale.clamp(0., 4.);
    }

    pub fn drag_input(&mut self, distance:Screen<Vec2>) {
        if let Some(_) = self.cancel_homing() {
            self.pos -= distance.convert(&*self);
        }
    }

    pub fn home(&mut self, window:&Window) {
        self.state = CameraState::Homing(HomingCameraState { target_pos: self.default_pos(window), target_scale: 1. });
    }

    pub fn triple(&mut self, window:&Window) {
        self.pos = (self.pos.to_vec2() / 3.).to_point();
        self.scale /= 3.;
        self.home(window);
    }
}

