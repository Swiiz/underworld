use ecs::Entity;
use winit::{
    event::{DeviceEvent, RawKeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::core::spatial::Position;

/// Player component
pub struct PlayerTag;

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
    pub fn handle_platform_input(&mut self, input: &DeviceEvent) {
        match self {
            PlayerController::Moving {
                forward,
                backward,
                left,
                right,
            } => match input {
                DeviceEvent::Key(RawKeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                }) => {
                    *match keycode {
                        KeyCode::KeyW => forward,
                        KeyCode::KeyS => backward,
                        KeyCode::KeyA => left,
                        KeyCode::KeyD => right,
                        _ => return,
                    } = state == &winit::event::ElementState::Pressed
                }
                _ => {}
            },
        }
    }

    pub fn update_player_entity(&self, entity: impl Entity, dt: f32) {
        match self {
            PlayerController::Moving {
                forward,
                backward,
                left,
                right,
            } => {
                let mut pos = entity.get_mut::<Position>().unwrap();

                if *forward {
                    pos.y += dt;
                }
                if *backward {
                    pos.y -= dt;
                }
                if *left {
                    pos.x -= dt;
                }
                if *right {
                    pos.x += dt;
                }
            }
        }
    }
}
