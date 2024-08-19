use cgmath::{Matrix3, Vector2};
use ecs::{Entities, Entity, Query};
use graphics::{
    ctx::Frame,
    sprite::{Sprite, SpriteDrawParams},
};

use crate::camera::Camera;

use common::core::spatial::Position;

pub struct RenderData {
    pub sprites: Vec<(Sprite, SpriteDrawParams)>,
}

impl RenderData {
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
        }
    }

    pub fn add_sprite(&mut self, sprite: Sprite, draw_params: SpriteDrawParams) {
        self.sprites.push((sprite, draw_params));
    }

    pub fn with(mut self, sprite: Sprite, draw_params: SpriteDrawParams) -> Self {
        self.add_sprite(sprite, draw_params);
        self
    }

    pub fn draw(&self, transform: Matrix3<f32>, frame: &mut Frame) {
        for (sprite, draw_params) in self.sprites.iter() {
            frame.renderer.sprites.draw(
                sprite.clone(),
                SpriteDrawParams {
                    transform: transform
                        * draw_params.transform
                        * Matrix3::from_translation(-Vector2::new(0.5, 0.5)),
                    ..draw_params.clone()
                },
            );
        }
    }
}

pub fn draw_entities(entities: &Entities, frame: &mut Frame, camera: &Camera) {
    for entity in entities.with::<RenderData>().iter() {
        let pos = entity
            .get::<Position>()
            .map(|x| x.0)
            .unwrap_or(Vector2::new(0.0, 0.0));

        let transform = camera.view_transform() * Matrix3::from_translation(pos);

        entity.get::<RenderData>().unwrap().draw(transform, frame);
    }
}
