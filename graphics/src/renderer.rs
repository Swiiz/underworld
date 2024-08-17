use crate::{
    ctx::{GraphicsCtx, RenderCtx},
    sprite::renderer::SpriteRenderer,
};

pub struct Renderer {
    pub sprites: SpriteRenderer,
}

impl Renderer {
    pub fn submit(&mut self, render: &mut RenderCtx, ctx: &GraphicsCtx) {
        self.sprites.submit(render, ctx);
    }

    pub fn parts(&mut self) -> Vec<&mut dyn RendererPart> {
        vec![&mut self.sprites]
    }
}

pub trait RendererPart {
    fn resize(&mut self, ctx: &GraphicsCtx, window_size: (u32, u32));
    fn submit(&mut self, render: &mut RenderCtx, ctx: &GraphicsCtx);
}
