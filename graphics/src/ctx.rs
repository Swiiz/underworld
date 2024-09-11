use wgpu::*;

use crate::{color::Color3, renderer::Renderer};

pub struct GraphicsCtx<'w> {
    pub(crate) device: Device,
    pub(super) queue: Queue,
    pub(super) surface: Surface<'w>,
    pub(super) surface_texture_format: TextureFormat,
    pub(super) surface_capabilities: SurfaceCapabilities,

    depth_texture: Texture,
    depth_texture_view: TextureView,
    depth_texture_sampler: Sampler,
}

pub struct RenderCtx {
    pub(crate) view: TextureView,
    pub(crate) encoder: CommandEncoder,

    surface_texture: SurfaceTexture,
}

impl<'w> GraphicsCtx<'w> {
    pub fn new(window_size: impl Into<(u32, u32)>, target: impl Into<SurfaceTarget<'w>>) -> Self {
        let window_size = window_size.into();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: util::backend_bits_from_env().unwrap_or(Backends::all()),
            ..Default::default()
        });
        let surface = instance
            .create_surface(target)
            .unwrap_or_else(|e| panic!("Could not create graphics surface: {e}"));
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .unwrap_or_else(|e| panic!("Could not acquire graphics device: {e}"));

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_texture_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let (depth_texture, depth_texture_view, depth_texture_sampler) =
            create_depth_texture(&device, window_size);

        let mut _self = Self {
            device,
            queue,
            surface,
            surface_capabilities,
            surface_texture_format,
            depth_texture,
            depth_texture_sampler,
            depth_texture_view,
        };

        _self.resize(window_size);

        _self
    }

    pub(crate) fn resize(&mut self, window_size: (u32, u32)) {
        if window_size.0 > 0 && window_size.1 > 0 {
            self.surface.configure(
                &self.device,
                &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: self.surface_texture_format,
                    width: window_size.0,
                    height: window_size.1,
                    present_mode: self.surface_capabilities.present_modes[0],
                    alpha_mode: self.surface_capabilities.alpha_modes[0],
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                },
            );
            let (depth_texture, depth_texture_view, depth_texture_sampler) =
                create_depth_texture(&self.device, window_size);
            self.depth_texture = depth_texture;
            self.depth_texture_view = depth_texture_view;
            self.depth_texture_sampler = depth_texture_sampler;
        }
    }

    pub(crate) fn next_frame<'a>(&'a mut self, renderer: &'a mut Renderer) -> Option<Frame<'a>> {
        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(|e| match e {
                wgpu::SurfaceError::OutOfMemory => {
                    panic!("The system is out of memory for rendering!")
                }
                _ => format!("An error occured during surface texture acquisition: {e}"),
            })
            .ok()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        for part in renderer.parts() {
            part.prepare(&self, &mut encoder);
        }

        Some(Frame {
            render: RenderCtx {
                surface_texture,
                encoder,
                view,
            },
            ctx: self,
            renderer,
        })
    }
}

pub fn create_depth_texture(
    device: &wgpu::Device,
    (width, height): (u32, u32),
) -> (Texture, TextureView, Sampler) {
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some("Depth texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let texture = device.create_texture(&desc);

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::LessEqual),
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        ..Default::default()
    });

    (texture, view, sampler)
}

pub struct Frame<'a> {
    pub ctx: &'a GraphicsCtx<'a>,
    pub render: RenderCtx,
    pub renderer: &'a mut Renderer,
}

impl<'a> Frame<'a> {
    pub(crate) fn present(mut self) {
        self.render();

        self.ctx
            .queue
            .submit(std::iter::once(self.render.encoder.finish()));
        for part in self.renderer.parts() {
            part.finish();
        }
        self.render.surface_texture.present();
    }

    fn render(&mut self) {
        let mut pass = self
            .render
            .encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("Sprite Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color3::gray(0.01).into()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.ctx.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

        for part in self.renderer.parts() {
            part.render(&mut pass, &self.ctx);
        }
    }
}
