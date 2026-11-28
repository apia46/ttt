use std::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign};

use vello::kurbo::{Affine, Point, Rect, Size, Vec2};

use crate::space::{Context, Contexted, Space};

impl Space for Point { fn apply(self, transform:Affine) -> Self { transform * self } }

impl Space for Vec2 {
    fn apply(self, transform:Affine) -> Self {
        (transform.with_translation(Vec2::ZERO) * self.to_point()).to_vec2()
    }
}

impl Space for vello::kurbo::Size {
    fn apply(self, transform:Affine) -> Self {
        self.to_vec2().apply(transform).to_size()
    }
}

// ah... imperfect
impl Space for Rect {
    fn apply(self, transform:Affine) -> Self {
        Rect::from_center_size(self.center().apply(transform), self.size().apply(transform))
    }
}

impl<T:Space+Default, U:Context> Default for Contexted<T, U> {
    fn default() -> Self {
        T::default().wrapped()
    }
}

macro_rules! operation {
    ($name:ident, $name_assign:ident, $fn:ident, $fn_assign:ident) => {
        impl<T:Space+$name<V>, U:Context, V:Space> $name<Contexted<V, U>> for Contexted<T, U> where <T as $name<V>>::Output : Space {
            type Output = Contexted<T::Output, U>;
            fn $fn(self, rhs: Contexted<V, U>) -> Self::Output {
                self.inner.$fn(rhs.inner).wrapped()
            }
        }

        impl<T:Space+$name_assign<V>, U:Context, V:Space> $name_assign<Contexted<V, U>> for Contexted<T, U> {
            fn $fn_assign(&mut self, rhs: Contexted<V, U>) {
                self.inner.$fn_assign(rhs.inner);
            }
        }
    };
}
operation!(Add, AddAssign, add, add_assign);
operation!(Sub, SubAssign, sub, sub_assign);
operation!(Mul, MulAssign, mul, mul_assign);

trait Scalar {}
impl Scalar for f64 {}

impl<T:Space+Mul<V>, U:Context, V:Scalar> Mul<V> for Contexted<T, U> where <T as Mul<V>>::Output : Space {
    type Output = Contexted<T::Output, U>;
    fn mul(self, rhs:V) -> Self::Output {
        self.inner.mul(rhs).wrapped()
    }
}

impl<T:Space+MulAssign<V>, U:Context, V:Scalar> MulAssign<V> for Contexted<T, U> {
    fn mul_assign(&mut self, rhs:V) {
        self.inner.mul_assign(rhs);
    }
}

impl<T:Space+Div<V>, U:Context, V:Scalar> Div<V> for Contexted<T, U> where <T as Div<V>>::Output : Space {
    type Output = Contexted<T::Output, U>;
    fn div(self, rhs:V) -> Self::Output {
        self.inner.div(rhs).wrapped()
    }
}

impl<T:Space+DivAssign<V>, U:Context, V:Scalar> DivAssign<V> for Contexted<T, U> {
    fn div_assign(&mut self, rhs:V) {
        self.inner.div_assign(rhs);
    }
}

impl<T:Context> Contexted<Vec2, T> {
    pub fn new(x:f64, y:f64) -> Self {
        Vec2::new(x, y).wrapped()
    }

    pub fn to_point(self) -> Contexted<Point, T> {
        self.inner.to_point().wrapped()
    }
}

impl<T:Context> Contexted<Point, T> {
    pub fn new(x:f64, y:f64) -> Self {
        Point::new(x, y).wrapped()
    }

    pub fn to_vec2(self) -> Contexted<Vec2, T> {
        self.inner.to_vec2().wrapped()
    }

    pub fn distance_squared(self, other:Self) -> f64 {
        self.inner.distance_squared(other.inner)
    }
}

impl<T:Context> Contexted<Rect, T> {
    pub fn from_points(p0:Contexted<impl Space+Into<Point>, T>, p1:Contexted<impl Space+Into<Point>, T>) -> Self {
        Rect::from_points(p0.inner, p1.inner).wrapped()
    }

    pub fn from_center_size(center:Contexted<impl Space+Into<Point>, T>, size: Contexted<impl Space+Into<Size>, T>) -> Self {
        Rect::from_center_size(center.inner, size.inner).wrapped()
    }

    pub fn center(&self) -> Contexted<Point, T> {
        self.inner.center().wrapped()
    }

    pub fn contains(&self, point:Contexted<impl Space+Into<Point>, T>) -> bool {
        self.inner.contains(point.inner)
    }
}

