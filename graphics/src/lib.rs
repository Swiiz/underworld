use ctx::{Frame, GraphicsCtx};
use renderer::Renderer;

pub mod color;
pub mod ctx;
pub mod renderer;
pub mod sprite;

pub use cgmath as maths;

pub struct Graphics {
    pub ctx: GraphicsCtx<'static>,
    pub renderer: Renderer,
}

impl Graphics {
    pub fn resize(&mut self, window_size: impl Into<(u32, u32)>) {
        let window_size = window_size.into();
        self.ctx.resize(window_size);
        for part in self.renderer.parts() {
            part.resize(&self.ctx, window_size)
        }
    }

    pub fn render(&mut self, renderfunc: impl Fn(&mut Frame)) {
        if let Some(mut frame) = self.ctx.next_frame(&mut self.renderer) {
            renderfunc(&mut frame);
            frame.present();
        }
    }
}
