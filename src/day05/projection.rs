use std::ops::RangeInclusive;

pub type Coord = i16;
pub type X = Coord;
pub type Y = Coord;

/// Horizontal x and y are normal x and y.
#[derive(Default)]
pub struct Horizontal;

/// Vertical x and y are normal y and x (swapped).
#[derive(Default)]
pub struct Vertical;

/// Diag-pos x is normal x, but y is projected along y=x onto x=0.
#[derive(Default)]
pub struct DiagPos;

/// Diag-neg x is normal x, but y is projected along y=-x onto x=0.
#[derive(Default)]
pub struct DiagNeg;

pub trait ProjectOnto<T> {
    /// Given local (x, y) coordinates in `Self`, return y-coordinate in `T`.
    fn project_onto(x: X, y: Y) -> Y;
}

macro_rules! impl_proj_onto {
    ($from:ty, $to:ty, $func:expr) => {
        impl ProjectFrom<$from> for $to {
            fn project_from(x: X, y: Y) -> Y {
                ($func)(x, y)
            }
        }
    };
}

impl_proj_onto!(Horizontal, DiagPos, |x, y| y - x);
impl_proj_onto!(Horizontal, DiagNeg, |x, y| y + x);
impl_proj_onto!(Horizontal, Vertical, |x, _y| x);

impl_proj_onto!(Vertical, DiagPos, |x, y| x - y);
impl_proj_onto!(Vertical, DiagNeg, |x, y| x + y);
impl_proj_onto!(Vertical, Horizontal, |x, _y| x);

impl_proj_onto!(DiagPos, Horizontal, |x, y| y + x);
impl_proj_onto!(DiagPos, Vertical, |x, _y| x);
impl_proj_onto!(DiagPos, DiagNeg, |x, y| y + 2 * x);

impl_proj_onto!(DiagNeg, Horizontal, |x, y| y - x);
impl_proj_onto!(DiagNeg, Vertical, |x, _y| x);
impl_proj_onto!(DiagNeg, DiagPos, |x, y| y - 2 * x);

pub trait ProjectFrom<T> {
    /// The reverse of project_onto().
    fn project_from(x: X, y: Y) -> Y;
}

impl<F, T: ProjectFrom<F>> ProjectOnto<T> for F {
    fn project_onto(x: X, y: Y) -> Y {
        T::project_from(x, y)
    }
}

impl<T> ProjectFrom<T> for T {
    fn project_from(_x: X, y: Y) -> Y {
        y
    }
}

pub trait ProjectableOnto {
    fn project_onto<F: ProjectOnto<T>, T>(self, y: Y) -> Self;
}

impl ProjectableOnto for X {
    fn project_onto<F: ProjectOnto<T>, T>(self, y: Y) -> Self {
        F::project_onto(self, y)
    }
}

impl ProjectableOnto for (X, X) {
    fn project_onto<F: ProjectOnto<T>, T>(self, y: Y) -> Self {
        (F::project_onto(self.0, y), F::project_onto(self.1, y))
    }
}

impl ProjectableOnto for RangeInclusive<X> {
    fn project_onto<F: ProjectOnto<T>, T>(self, y: Y) -> Self {
        let (start, end) = (F::project_onto(*self.start(), y), F::project_onto(*self.end(), y));
        if start <= end {
            start..=end
        } else {
            end..=start
        }
    }
}

pub trait IntersectWith<T> {
    /// Given two local y-coordinates in `Self` and `T`, find the resulting
    /// x-coordinate of the intersection point (note that x-axis is shared for
    /// all non-vertical line directions). This intersection almost always exists
    /// except for diag-pos vs diag-neg case where the difference between the two
    /// y-coordinates is not divisible by 2.
    fn intersect_with(y1: Y, y2: Y) -> Option<X>;
}

macro_rules! impl_intersect_with {
    ($ty1:ty, $ty2:ty, $func:expr) => {
        impl IntersectWith<$ty2> for $ty1 {
            fn intersect_with(y1: Y, y2: Y) -> Option<X> {
                ($func)(y1, y2)
            }
        }
        impl IntersectWith<$ty1> for $ty2 {
            fn intersect_with(y1: Y, y2: Y) -> Option<X> {
                ($func)(y2, y1)
            }
        }
    };
    ($ty:ty) => {
        impl IntersectWith<$ty> for $ty {
            fn intersect_with(_y1: Y, _y2: Y) -> Option<X> {
                None
            }
        }
    };
}

impl_intersect_with!(Horizontal, DiagPos, |y1, y2| Some(y1 - y2));
impl_intersect_with!(Horizontal, DiagNeg, |y1, y2| Some(y2 - y1));
impl_intersect_with!(DiagPos, DiagNeg, |y1, y2| ((y2 - y1) % 2 == 0).then(|| (y2 - y1) / 2));
impl_intersect_with!(Horizontal);
impl_intersect_with!(DiagPos);
impl_intersect_with!(DiagNeg);

pub trait IntersectableWith {
    fn intersect_with<F: IntersectWith<T>, T>(self, y: Y) -> Option<X>;
}

impl IntersectableWith for Y {
    fn intersect_with<F: IntersectWith<T>, T>(self, y: Y) -> Option<X> {
        F::intersect_with(self, y)
    }
}

pub trait LineDirection:
    Default
    + ProjectFrom<Horizontal>
    + ProjectFrom<Vertical>
    + ProjectOnto<Horizontal>
    + ProjectOnto<Vertical>
    + ProjectOnto<Self::A>
    + ProjectOnto<Self::B>
    + IntersectWith<Self::A>
    + IntersectWith<Self::B>
{
    type A: LineDirection + ProjectFrom<Self> + IntersectWith<Self>;
    type B: LineDirection + ProjectFrom<Self> + IntersectWith<Self>;
}

impl LineDirection for Horizontal {
    type A = DiagPos;
    type B = DiagNeg;
}

impl LineDirection for DiagPos {
    type A = Horizontal;
    type B = DiagNeg;
}

impl LineDirection for DiagNeg {
    type A = Horizontal;
    type B = DiagPos;
}
