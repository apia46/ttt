use std::marker::PhantomData;

use vello::kurbo::{Affine, Vec2};

use crate::{space::{Context, Contexted, Screen}, util::ComponentWise, window::Window};

pub trait AnchorPoint: Copy + PartialEq {
    const POSITION:Screen<Vec2>;
}

#[derive(Clone, Copy, PartialEq)]
pub struct AnchorContext<T:AnchorPoint>(PhantomData<T>);

pub type Anchor<T, U> = Contexted<T, AnchorContext<U>>;

macro_rules! anchor_point {
    ($name:ident, $position:expr) => {
        #[derive(Clone, Copy, PartialEq)]
        pub struct $name;
        impl AnchorPoint for $name { const POSITION:Screen<Vec2> = Screen::wrap($position); }
    };
}

anchor_point!(BottomMiddle, Vec2::new(0.5, 1.0));
anchor_point!(BottomRight, Vec2::new(1.0, 1.0));

impl<T:AnchorPoint> Context for AnchorContext<T> {
    type State<'a> = &'a Window;

    fn transform_from<'a>(state:Self::State<'a>) -> Affine {
        Affine::translate(state.size.mul(T::POSITION).to_inner())
    }
}
