use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }

    pub fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y)
    }

    pub fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s)
    }

    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn dist(self, o: Self) -> f32 {
        self.sub(o).length()
    }

    pub fn norm(self) -> Self {
        let l = self.length();
        if l < 1e-6 {
            Self::ZERO
        } else {
            self.mul(1.0 / l)
        }
    }

    pub fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }

    pub fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y
    }
}
