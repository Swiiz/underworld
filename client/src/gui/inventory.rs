use cgmath::{Array, Matrix, Matrix3, SquareMatrix, Vector2};
use graphics::{
    ctx::Frame,
    sprite::{Sprite, SpriteDrawParams},
};

use crate::core::{assets::ClientAssets, platform::PlatformInput};

use super::Gui;

pub struct PlayerInventory {}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Gui for PlayerInventory {
    fn input(&mut self, input: &PlatformInput) {}

    fn render(&self, frame: &mut Frame, assets: &ClientAssets) {
        frame.renderer.sprites.draw(
            Sprite {
                sheet: assets.textures.get_id("inventory"),
                pos: Vector2::new(0, 0),
                size: Vector2::from_value(1),
            },
            SpriteDrawParams {
                transform: Matrix3::from_scale(2.)
                    * Matrix3::from_translation(Vector2::new(-0.5, -0.5)),
                ..Default::default()
            },
        )
    }
}
