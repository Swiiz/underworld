use wgpu::{CommandEncoder, RenderPass};

use crate::{
    ctx::GraphicsCtx,
    sprite::renderer::SpriteRendererPart,
    text::renderer::TextRendererPart,
};

pub struct Renderer {
    pub sprites: SpriteRendererPart,
    pub text: TextRendererPart,
}

impl Renderer {
    pub(crate) fn parts(&mut self) -> Vec<&mut dyn RendererPart> {
        vec![&mut self.sprites, &mut self.text]
    }
}

pub(crate) trait RendererPart {
    fn prepare(&mut self, ctx: &GraphicsCtx, encoder: &mut CommandEncoder);
    fn render<'a>(&'a mut self, render_pass: &mut RenderPass<'a>, ctx: &GraphicsCtx);
    fn finish(&mut self) {}
    fn resize(&mut self, ctx: &GraphicsCtx, window_size: (u32, u32));
}
