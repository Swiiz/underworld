use ctx::{Frame, GraphicsCtx};
use renderer::Renderer;
use wgpu::SurfaceTarget;

pub mod color;
pub mod ctx;
pub mod renderer;
pub mod sprite;

pub use cgmath as maths;

pub struct Graphics<'w> {
    pub ctx: GraphicsCtx<'w>,
    pub renderer: Renderer,
}

impl<'w> Graphics<'w> {
    pub fn new(window_size: (u32, u32), target: impl Into<SurfaceTarget<'w>>) -> Self {
        Self {
            ctx: GraphicsCtx::new(window_size, target),
            renderer: Renderer::new(),
        }
    }

    pub fn resize(&mut self, window_size: (u32, u32)) {
        self.ctx.resize(window_size);
        self.renderer.resize(&self.ctx, window_size)
    }

    pub fn render(&mut self, renderfunc: impl Fn(&mut Frame)) {
        if let Some(mut frame) = self.ctx.next_frame(&mut self.renderer) {
            renderfunc(&mut frame);
            frame.present();
        }
    }
}
