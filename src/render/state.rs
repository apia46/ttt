use std::time::Duration;

use educe::Educe;

#[derive(Educe)]
#[educe(Default)]
pub struct RenderState {
    pub previous_frame: Duration,
    pub delta_time: f64,
    #[educe(Default=Duration::from_secs_f64(1./60.))]
    pub refresh_rate: Duration,
}

