use crate::ctx::{GraphicsCtx, RenderCtx};
use downcast_rs::{impl_downcast, Downcast};
use std::{any::TypeId, collections::HashMap};

pub trait Renderable {
    type Renderer: RendererPlugin + Draw<Self>
    where
        Self: Sized;
}

pub trait RendererPlugin: Downcast {
    fn resize(&mut self, graphics_ctx: &GraphicsCtx, window_size: (u32, u32));
    fn submit(&mut self, render_ctx: &mut RenderCtx, graphics_ctx: &GraphicsCtx);
}
impl_downcast!(RendererPlugin);

pub trait Draw<T: Renderable> {
    type Params;
    fn draw(&mut self, value: T, params: Self::Params);
}

pub struct Renderer {
    plugins: HashMap<TypeId, Box<dyn RendererPlugin>>,
}

impl Renderer {
    pub(crate) fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub(crate) fn resize(&mut self, graphics: &GraphicsCtx, window_size: (u32, u32)) {
        for r in self.plugins.values_mut() {
            r.resize(graphics, window_size);
        }
    }

    pub fn add_plugin<T: RendererPlugin>(&mut self, plugin: T) {
        self.plugins.insert(TypeId::of::<T>(), Box::new(plugin));
    }

    pub fn get_plugin<T: RendererPlugin>(&mut self) -> &mut T {
        self.plugins
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut::<T>()
            .unwrap()
    }

    pub(crate) fn submit(&mut self, rctx: &mut RenderCtx, gctx: &GraphicsCtx) {
        for r in self.plugins.values_mut() {
            r.submit(rctx, gctx);
        }
    }
}
