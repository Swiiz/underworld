use crate::{platform::PlatformInput, rendering::RenderData};
use cgmath::{InnerSpace, Vector2, Zero};
use common::{core::spatial::Position, utils::maths::MaybeNan};
use ecs::Entity;
use winit::{event::ElementState, keyboard::KeyCode};

/// Player controller
pub enum PlayerController {
    Moving {
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
    },
}

impl Default for PlayerController {
    fn default() -> Self {
        Self::Moving {
            forward: false,
            backward: false,
            left: false,
            right: false,
        }
    }
}

impl PlayerController {
    pub fn handle_platform_input(&mut self, input: &PlatformInput) {
        match self {
            PlayerController::Moving {
                forward,
                backward,
                left,
                right,
            } => match input {
                &PlatformInput::Keyboard { key, state } => {
                    *match key {
                        KeyCode::KeyW => forward,
                        KeyCode::KeyS => backward,
                        KeyCode::KeyA => left,
                        KeyCode::KeyD => right,
                        _ => return,
                    } = state == ElementState::Pressed
                }
                _ => {}
            },
        }
    }

    pub fn update_entity(&self, entity: &impl Entity, dt: f32) {
        match self {
            PlayerController::Moving {
                forward,
                backward,
                left,
                right,
            } => {
                let mut render_data = entity.get_mut::<RenderData>().unwrap();
                let sheet_pos = &mut render_data.sprites[0].0.pos;

                let mut dir = Vector2::<f32>::zero();

                if *forward {
                    *sheet_pos = Vector2::new(1, 3);
                    dir.y += 1.;
                }
                if *backward {
                    *sheet_pos = Vector2::new(1, 0);
                    dir.y -= 1.;
                }
                if *left {
                    *sheet_pos = Vector2::new(1, 1);
                    dir.x -= 1.;
                }
                if *right {
                    *sheet_pos = Vector2::new(1, 2);
                    dir.x += 1.;
                }

                let mut pos = entity.get_mut::<Position>().unwrap();
                let speed = 5.0;

                pos.0 += (dir.normalize() * dt * speed).no_nan();
            }
        }
    }
}
