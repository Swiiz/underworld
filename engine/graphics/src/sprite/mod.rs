use std::{collections::HashMap, hash::Hash, path::PathBuf};

use cgmath::{Matrix3, Vector2};
use serde::{Deserialize, Serialize};
use texture_packer::{
    exporter::ImageExporter, importer::ImageImporter, texture::Texture as _, TexturePacker,
    TexturePackerConfig,
};
use wgpu::{BindGroup, BindGroupLayout, Texture};

pub mod renderer;

use crate::{color::Color3, ctx::GraphicsCtx};

pub struct Atlas<K> {
    pub(super) sheets: HashMap<K, SpriteSheet>,
    bind_group: BindGroup,
}

pub trait SpriteSheetKey: Clone + Eq + Hash + 'static {
    fn register_spritesheets(registry: &mut SpriteRegistry<Self>)
    where
        Self: Sized;
}

pub struct SpriteSheetData {
    pub path: PathBuf,
    pub sprite_px_size: Vector2<u32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SpriteSheet {
    pub size_px: Vector2<u32>,
    pub sprite_size_px: Vector2<u32>,
    tex_coords: Vector2<f32>,
    tex_dims: Vector2<f32>,
}

impl SpriteSheet {
    fn normalize_tex_vec(&self, vec: Vector2<u32>) -> Vector2<f32> {
        let sprite_size_px = self.sprite_size_px.map(|x| x as f32);
        let sprite_pos = vec.map(|x| x as f32);
        let size_px = self.size_px.map(|x| x as f32);

        let pos_px = sprite_size_px.zip(sprite_pos, |a, b| a * b);
        pos_px.zip(size_px, |a, b| a / b)
    }

    fn tex_coords(&self, sprite_coords: Vector2<u32>) -> Vector2<f32> {
        self.tex_coords
            + self
                .normalize_tex_vec(sprite_coords)
                .zip(self.tex_dims, |a, b| a * b)
    }

    fn tex_dims(&self, sprite_dims: Vector2<u32>) -> Vector2<f32> {
        self.normalize_tex_vec(sprite_dims)
            .zip(self.tex_dims, |a, b| a * b)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Sprite<SsK> {
    pub sheet: SsK,
    pub position: Vector2<u32>,
    pub size: Vector2<u32>,
}

#[derive(Clone)]
pub struct SpriteParams {
    pub transform: Matrix3<f32>,
    pub tint: Color3,
    pub depth: f32,
}

pub struct SpriteRegistry<K> {
    to_load: HashMap<K, SpriteSheetData>,
}

impl<K: SpriteSheetKey> SpriteRegistry<K> {
    pub fn new() -> Self {
        Self {
            to_load: HashMap::new(),
        }
    }

    pub fn register(&mut self, key: K, spritesheet_data: SpriteSheetData) {
        self.to_load.insert(key, spritesheet_data);
    }

    pub(super) fn build_atlas(
        self,
        ctx: &GraphicsCtx,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Atlas<K> {
        let mut packer = TexturePacker::new_skyline(TexturePackerConfig {
            max_width: 4096,
            max_height: 4096,
            texture_padding: 0,
            trim: false,
            allow_rotation: false,
            ..Default::default()
        });

        let images = self.to_load.iter().map(|(k, ssd)| {
            (
                k,
                ImageImporter::import_from_file(&ssd.path).expect("Unable to load sprite(sheet)!"),
            )
        });

        images.for_each(|(k, img)| {
            packer
                .pack_own(k, img)
                .expect("Failed to pack sprite(sheet) into global atlas!")
        });

        let sheets = packer
            .get_frames()
            .into_iter()
            .map(|(k, sheet)| {
                (
                    (*k).clone(),
                    SpriteSheet {
                        size_px: Vector2 {
                            x: sheet.frame.w,
                            y: sheet.frame.h,
                        },
                        sprite_size_px: self.to_load.get(k).unwrap().sprite_px_size,
                        tex_coords: Vector2 {
                            x: sheet.frame.x as f32 / packer.width() as f32,
                            y: sheet.frame.y as f32 / packer.height() as f32,
                        },
                        tex_dims: Vector2 {
                            x: sheet.frame.w as f32 / packer.width() as f32,
                            y: sheet.frame.h as f32 / packer.height() as f32,
                        },
                    },
                )
            })
            .collect();

        let image = ImageExporter::export(&packer)
            .expect("An error occured while exporting global  atlas!")
            .to_rgba8();
        let size: Vector2<u32> = image.dimensions().into();

        let (_texture, bind_group) =
            create_texture(ctx, size, image.into_vec(), texture_bind_group_layout);

        Atlas { sheets, bind_group }
    }
}

fn create_texture(
    ctx: &GraphicsCtx,
    size: Vector2<u32>,
    image: Vec<u8>,
    texture_bind_group_layout: &BindGroupLayout,
) -> (Texture, BindGroup) {
    let texture_size = wgpu::Extent3d {
        width: size.x,
        height: size.y,
        depth_or_array_layers: 1,
    };

    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("2d_texture"),
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("2d_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 1.0,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("2d_texture_bind_group"),
    });

    ctx.queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        image.as_ref(),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.x),
            rows_per_image: Some(size.y),
        },
        texture_size,
    );

    (texture, bind_group)
}
