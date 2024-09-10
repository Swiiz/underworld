use std::time::Instant;

pub struct Timer {
    pub start: Instant,
    pub last_update: Instant,
    pub last_render: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            last_update: Instant::now(),
            last_render: Instant::now(),
        }
    }

    pub fn update_dt(&mut self) -> f32 {
        let e = self.last_update.elapsed().as_secs_f32();
        self.last_update = Instant::now();
        e
    }

    // Move into client?
    pub fn render_dt(&mut self) -> f32 {
        let e = self.last_render.elapsed().as_secs_f32();
        self.last_render = Instant::now();
        e
    }
}
