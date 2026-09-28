use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// A 2D vector in world units. World y grows downward, like screen space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub const ZERO: V2 = V2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        V2 { x, y }
    }

    pub fn from_angle(angle: f32) -> Self {
        V2::new(angle.cos(), angle.sin())
    }

    pub fn len_sq(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn len(self) -> f32 {
        self.len_sq().sqrt()
    }

    /// Unit vector, or zero for a (near) zero vector.
    pub fn norm(self) -> Self {
        let len = self.len();
        if len > 1e-6 {
            self * (1.0 / len)
        } else {
            V2::ZERO
        }
    }

    pub fn dot(self, other: V2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn dist_sq(self, other: V2) -> f32 {
        (self - other).len_sq()
    }

    pub fn dist(self, other: V2) -> f32 {
        (self - other).len()
    }

    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn rotate(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        V2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        V2::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f32> for V2 {
    type Output = V2;
    fn mul(self, s: f32) -> V2 {
        V2::new(self.x * s, self.y * s)
    }
}

impl Neg for V2 {
    type Output = V2;
    fn neg(self) -> V2 {
        V2::new(-self.x, -self.y)
    }
}

impl AddAssign for V2 {
    fn add_assign(&mut self, o: V2) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl SubAssign for V2 {
    fn sub_assign(&mut self, o: V2) {
        self.x -= o.x;
        self.y -= o.y;
    }
}
