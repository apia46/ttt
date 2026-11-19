use crate::render::{RendererState, camera::Camera};

#[derive(Default)]
pub struct RenderState {
    pub renderer_state: RendererState,
    pub camera: Camera,
}

