mod camera;
mod impls;
pub mod anchor;

use std::marker::PhantomData;

pub use camera::{Camera, World};
use vello::kurbo::Affine;

#[derive(Clone, Copy, PartialEq)]
pub struct Contexted<T:Space, U:Context> {
    inner: T,
    _context: PhantomData<U>,
}

/// something within 2d space
pub trait Space : Copy + PartialEq {
    fn apply(self, transform:Affine) -> Self;

    fn to_screen(self) -> Screen<Self> {
        Screen::wrap(self)
    }

    fn to_context<'a, T:Context>(self, state:T::State<'a>) -> Contexted<Self, T> {
        Contexted::from_screen(self.to_screen(), state)
    }

    /// escape hatch; be careful
    fn wrapped<T:Context>(self) -> Contexted<Self, T> {
        Contexted::wrap(self)
    }
}

pub trait Context : Copy + PartialEq {
    type State<'a> : Copy;
    fn transform_from<'a>(state:Self::State<'a>) -> Affine;
    fn transform_to<'a>(state:Self::State<'a>) -> Affine {
        Self::transform_from(state).inverse()
    }
}

impl<T:Space, U:Context> Contexted<T, U> {
    /// escape hatch; be careful
    pub fn cast<V:Context>(self) -> Contexted<T, V> {
        Contexted::wrap(self.inner)
    }

    /// escape hatch; be careful
    pub fn to_inner(self) -> T {
        self.inner
    }

    /// escape hatch; be careful
    pub fn as_inner(&mut self) -> &mut T {
        &mut self.inner
    }

    pub const fn wrap(inner:T) -> Self {
        Self { inner, _context: PhantomData }
    }

    pub fn to_screen<'a>(self, state:U::State<'a>) -> Screen<T> {
        self.inner.apply(U::transform_to(state).inverse()).wrapped()
    }

    pub fn from_screen<'a>(value:Screen<T>, state:U::State<'a>) -> Self {
        value.inner.apply(U::transform_to(state)).wrapped()
    }
}

impl<T:Space> Contexted<T, ScreenContext> {
    pub fn convert<'a, U:Context>(self, state:U::State<'a>) -> Contexted<T, U> {
        Contexted::from_screen(self, state)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ScreenContext;

pub type Screen<T> = Contexted<T, ScreenContext>;

impl Context for ScreenContext {
    type State<'a> = ();
    fn transform_from<'a>(_state:Self::State<'a>) -> Affine { Affine::IDENTITY }
}

