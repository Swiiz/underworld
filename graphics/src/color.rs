use cgmath::Vector3;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color3 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<Vector3<f32>> for Color3 {
    fn from(v: Vector3<f32>) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
        }
    }
}

impl Into<wgpu::Color> for Color3 {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: 1.0,
        }
    }
}

impl Into<[f32; 3]> for Color3 {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl Color3 {
    pub const WHITE: Self = Self::gray(1.0);
    pub const BLACK: Self = Self::gray(0.0);

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub const fn gray(l: f32) -> Self {
        Self::new(l, l, l)
    }
}
