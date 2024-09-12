use std::{mem::size_of, num::NonZeroU64};

use bytemuck::{cast_slice, Pod, Zeroable};
use cgmath::Matrix3;

use crate::{ctx::GraphicsCtx, renderer::RendererPart};
use wgpu::{util::StagingBelt, *};

use super::{build_atlas, Atlas, Sprite, SpriteDrawParams, SpriteSheetSource};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SpriteInstance {
    transform: [[f32; 3]; 3],
    tex_pos: [f32; 2],
    tex_dims: [f32; 2],
    tint: [f32; 3],
    z_index: f32,
}

pub struct SpriteRendererPart {
    pipeline: RenderPipeline,
    quad_vertex_buf: Buffer,
    quad_index_buf: Buffer,
    sprite_instance_buf: Buffer,
    sprite_staging_belt: StagingBelt,

    proj_matrix: Matrix3<f32>,
    atlas: Atlas,
    queue: Vec<SpriteInstance>,
}

const MAX_SPRITES: u64 = 10_000;

impl SpriteRendererPart {
    pub fn new<'a>(
        ctx: &GraphicsCtx,
        window_size: impl Into<(u32, u32)>,
        sprite_sheets: impl Iterator<Item = &'a SpriteSheetSource>,
    ) -> Self {
        let window_size = window_size.into();
        let (sprite_pipeline, texture_bind_group_layout) = create_sprite_pipeline(
            &ctx.device,
            &ctx.depth_stencil_state,
            ctx.surface_texture_format,
        );

        let (quad_vertex_buf, quad_index_buf) = create_quad_vertex_buf(&ctx.device);
        let sprite_instance_buf = create_sprite_instance_buf(&ctx.device);
        let sprite_staging_belt =
            StagingBelt::new(std::mem::size_of::<SpriteInstance>() as u64 * MAX_SPRITES);

        let queue = Vec::with_capacity(MAX_SPRITES as usize);

        let atlas = build_atlas(sprite_sheets, ctx, &texture_bind_group_layout);

        let proj_matrix = compute_proj_matrix(window_size);

        Self {
            pipeline: sprite_pipeline,
            quad_vertex_buf,
            quad_index_buf,
            sprite_staging_belt,
            sprite_instance_buf,
            proj_matrix,
            queue,
            atlas,
        }
    }

    pub fn draw(&mut self, sprite: Sprite, params: SpriteDrawParams) {
        let spritesheet = self.atlas.sheets[sprite.sheet.0];

        self.queue.push(SpriteInstance {
            transform: (self.proj_matrix * params.transform).into(),
            tex_pos: spritesheet.tex_coords(sprite.pos).into(),
            tex_dims: spritesheet.tex_dims(sprite.size).into(),
            tint: params.tint.into(),
            z_index: params.depth,
        })
    }
}

impl RendererPart for SpriteRendererPart {
    fn resize(&mut self, _: &GraphicsCtx, window_size: (u32, u32)) {
        self.proj_matrix = compute_proj_matrix(window_size);
    }

    fn prepare(&mut self, gctx: &GraphicsCtx, encoder: &mut CommandEncoder) {
        let len = self.queue.len();

        let queue = std::mem::replace(&mut self.queue, Vec::with_capacity(len));

        let rawqueue = cast_slice(&queue);

        if len != 0 {
            {
                let byte_size = (queue.len() * size_of::<SpriteInstance>()) as u64;
                let mut bufmut = self.sprite_staging_belt.write_buffer(
                    encoder,
                    &self.sprite_instance_buf,
                    0,
                    NonZeroU64::new(byte_size).unwrap(),
                    &gctx.device,
                );
                bufmut.clone_from_slice(rawqueue);
            }
        }
        self.sprite_staging_belt.finish();
    }

    fn render(&mut self, render_pass: &mut wgpu::RenderPass<'_>, _: &GraphicsCtx) {
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_vertex_buffer(0, self.quad_vertex_buf.slice(..));
        render_pass.set_vertex_buffer(1, self.sprite_instance_buf.slice(..));
        render_pass.set_bind_group(0, &self.atlas.bind_group, &[]);
        render_pass.set_index_buffer(self.quad_index_buf.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..self.queue.len() as u32);
    }

    fn finish(&mut self) {
        self.sprite_staging_belt.recall();
    }
}

fn create_sprite_pipeline(
    device: &Device,
    depth_stencil_state: &DepthStencilState,
    surface_texture_format: TextureFormat,
) -> (RenderPipeline, BindGroupLayout) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
            label: Some("bind_group_layout"),
        });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("2d_render_pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: 4 * std::mem::size_of::<f32>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                        },
                    ],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                            shader_location: 4,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                            shader_location: 5,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                            shader_location: 6,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 13]>() as wgpu::BufferAddress,
                            shader_location: 7,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                            shader_location: 8,
                            format: wgpu::VertexFormat::Float32,
                        },
                    ],
                },
            ],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(depth_stencil_state.clone()),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    (render_pipeline, texture_bind_group_layout)
}

fn create_quad_vertex_buf(device: &Device) -> (Buffer, Buffer) {
    let (o, u) = (0., 1.);
    #[rustfmt::skip]
    let vertex_data: [f32; 16] = [
//    [ x,  y,  u,  v ]
        o,  o,  o,  u, // bottom left
        u,  o,  u,  u, // bottom right
        u,  u,  u,  o, // top right
        o,  u,  o,  o, // top left
    ];

    let index_data = &[0u16, 1, 2, 0, 2, 3];

    let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("quad_vertex_buffer"),
            contents: unsafe {
                // SAFETY: Safe as long as vertex_data is [f32]
                std::slice::from_raw_parts(
                    vertex_data.as_ptr() as *const u8,
                    vertex_data.len() * std::mem::size_of::<f32>(),
                )
            },
            usage: wgpu::BufferUsages::VERTEX,
        },
    );

    let index_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("quad_index_buffer"),
            contents: unsafe {
                // SAFETY: Safe as long as index_data is [u16]
                std::slice::from_raw_parts(
                    index_data.as_ptr() as *const u8,
                    index_data.len() * std::mem::size_of::<u16>(),
                )
            },
            usage: wgpu::BufferUsages::INDEX,
        },
    );

    (vertex_buffer, index_buffer)
}

fn create_sprite_instance_buf(device: &Device) -> Buffer {
    let bufdesc = BufferDescriptor {
        label: Some("Sprite instance buffer"),
        size: MAX_SPRITES * std::mem::size_of::<SpriteInstance>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };

    device.create_buffer(&bufdesc)
}

fn compute_proj_matrix((w, h): (u32, u32)) -> Matrix3<f32> {
    let (w, h) = (w as f32, h as f32);
    let (x, y) = if w < h { (1.0, w / h) } else { (h / w, 1.0) };
    Matrix3::from_nonuniform_scale(x, y)
}
