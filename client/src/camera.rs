use cgmath::{Matrix3, Vector2};

use crate::platform::PlatformInput;

pub struct Camera {
    pub pos: Vector2<f32>,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos: Vector2::new(0., 0.),
            zoom: 0.2,
        }
    }

    pub fn handle_scroll(&mut self, event: &PlatformInput) {
        let PlatformInput::MouseScrolled { y, .. } = event else {
            return;
        };

        self.zoom += y * self.zoom * 0.05;
        self.zoom = self.zoom.max(0.02).min(1.0);
    }

    pub fn view_transform(&self) -> Matrix3<f32> {
        Matrix3::from_scale(self.zoom) * Matrix3::from_translation(-self.pos)
    }
}
