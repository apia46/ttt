use vello::kurbo::{Point, Rect, Size, Vec2};

use crate::window::Window;

pub const EXPAND_PROMPT_HALF_SIZE:Size = Size::new(150., 50.);
pub const EXPAND_PROMPT_BUTTON_HALF_SIZE:Size = Size::new(25., 10.);
pub const EXPAND_PROMPT_BUTTON_POS_OFFSETS:(Vec2, Vec2) = (Vec2::new(-50., 20.), Vec2::new(50., 20.));

pub fn expand_prompt_center(window:&Window) -> Point {
    let w = window.size.0;
    Point::new(w.x*0.5, w.y - EXPAND_PROMPT_HALF_SIZE.height - 50.)
}

