use cgmath::{Vector2, Zero};

pub trait MaybeNan: Zero {
    fn is_nan(&self) -> bool;
    fn replace_nan(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self.is_nan() {
            other
        } else {
            self
        }
    }

    fn no_nan(self) -> Self
    where
        Self: Sized,
    {
        self.replace_nan(Zero::zero())
    }
}

impl MaybeNan for Vector2<f32> {
    fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }
}
