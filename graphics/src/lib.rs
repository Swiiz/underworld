use ctx::{Frame, GraphicsCtx};
use renderer::Renderer;

pub mod color;
pub mod ctx;
pub mod renderer;
pub mod sprite;
pub mod text;

pub use cgmath as maths;
use sprite::{renderer::SpriteRendererPart, SpriteSheetSource};
use text::renderer::TextRendererPart;
use wgpu::SurfaceTarget;

pub struct Graphics {
    pub ctx: GraphicsCtx<'static>,
    pub renderer: Renderer,
}

impl Graphics {
    pub fn new<'a>(
        window_size: impl Into<(u32, u32)>,
        target: impl Into<SurfaceTarget<'static>>,
        textures: impl Iterator<Item = &'a SpriteSheetSource>,
    ) -> Self {
        let window_size = window_size.into();
        let ctx = GraphicsCtx::new(window_size, target);
        Self {
            renderer: Renderer {
                sprites: SpriteRendererPart::new(&ctx, window_size, textures),
                text: TextRendererPart::new(&ctx, window_size),
            },
            ctx,
        }
    }

    pub fn resize(&mut self, window_size: impl Into<(u32, u32)>) {
        let window_size = window_size.into();
        self.ctx.resize(window_size);
        for part in self.renderer.parts() {
            part.resize(&self.ctx, window_size)
        }
    }

    pub fn render(&mut self, mut renderfunc: impl FnMut(&mut Frame)) {
        if let Some(mut frame) = self.ctx.next_frame(&mut self.renderer) {
            renderfunc(&mut frame);
            frame.present();
        }
    }
}
