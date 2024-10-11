use cgmath::{Array, Matrix3, Vector2};
use graphics::{
    color::Color3,
    ctx::Frame,
    sprite::{Sprite, SpriteDrawParams},
    text::{HorizontalAlign, Layout, Section, Text, VerticalAlign},
};

use crate::{core::assets::ClientAssets, state::ClientState};

pub fn debug_overlay(frame: &mut Frame, dt: f32) {
    frame.renderer.text.draw_section(
        Section::default()
            .add_text(
                Text::new(&format!("FPS: {}", 1. / dt))
                    .with_color(Color3::WHITE)
                    .with_scale(24.),
            )
            .with_layout(
                Layout::default()
                    .h_align(HorizontalAlign::Left)
                    .v_align(VerticalAlign::Top),
            )
            .with_screen_position(Vector2::new(10.0, 10.0))
            .to_owned(),
    )
}

pub fn play_overlay(frame: &mut Frame, state: &ClientState, assets: &ClientAssets) {
    let ClientState::Connected { .. } = state else {
        return;
    };

    match state {
        ClientState::Connected { pi_controller, .. } => {
            for i in 0..10 {
                frame.renderer.sprites.draw(
                    Sprite {
                        sheet: assets.textures.get_id("actionbar"),
                        pos: Vector2::new((i == pi_controller.actionbar_slot).into(), 0),
                        size: Vector2::from_value(1),
                    },
                    SpriteDrawParams {
                        transform: Matrix3::from_translation(Vector2::new(
                            -0.81 + i as f32 * 0.18,
                            -1.,
                        )) * Matrix3::from_scale(0.15)
                            * Matrix3::from_translation(Vector2::new(-0.5, 0.)),
                        ..Default::default()
                    },
                )
            }
        }
        _ => {}
    }
}
