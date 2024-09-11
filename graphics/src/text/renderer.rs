use wgpu::*;
use wgpu_text::{
    glyph_brush::{ab_glyph::FontRef, OwnedSection, Section},
    BrushBuilder, TextBrush,
};

use crate::{
    ctx::{GraphicsCtx, RenderCtx},
    renderer::{Renderer, RendererPart},
};

pub struct TextRendererPart {
    brush: TextBrush<FontRef<'static>>,
    queue: Vec<OwnedSection>,
}

impl TextRendererPart {
    pub fn new(ctx: &GraphicsCtx, (width, height): (u32, u32)) -> Self {
        Self {
            brush: BrushBuilder::using_font_bytes(include_bytes!("font.ttf"))
                .expect("Failed to init font brush")
                .with_depth_stencil(Some(DepthStencilState {
                    format: TextureFormat::Depth32Float, // Same as in sprite renderer part
                    bias: DepthBiasState::default(),
                    depth_compare: CompareFunction::Less,
                    depth_write_enabled: false,
                    stencil: StencilState::default(),
                }))
                .build(&ctx.device, width, height, ctx.surface_texture_format),
            queue: vec![],
        }
    }

    pub fn draw_section(&mut self, section: OwnedSection) {
        self.queue.push(section.into());
    }
}

impl RendererPart for TextRendererPart {
    fn prepare(&mut self, ctx: &GraphicsCtx, _: &mut CommandEncoder) {
        self.brush
            .queue(
                &ctx.device,
                &ctx.queue,
                std::mem::take(&mut self.queue).as_slice(),
            )
            .expect("Failed to queue text sections for rendering");
    }

    fn render<'a>(&'a mut self, rpass: &mut RenderPass<'a>, _: &GraphicsCtx) {
        self.brush.draw(rpass);
    }

    fn resize(&mut self, ctx: &GraphicsCtx, (width, height): (u32, u32)) {
        self.brush
            .resize_view(width as f32, height as f32, &ctx.queue);
    }
}
