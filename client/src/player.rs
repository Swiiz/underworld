use crate::core::{network::NetworkClient, platform::PlatformInput, rendering::RenderData};
use cgmath::{InnerSpace, Vector2, Zero};
use common::{
    core::spatial::Position, network::proto::play::ServerboundSetPlayerPos, utils::maths::MaybeNan,
};
use ecs::Entity;
use winit::{event::ElementState, keyboard::KeyCode};

/// Player controller
pub enum PlayerEntityController {
    Moving {
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
    },
}

impl Default for PlayerEntityController {
    fn default() -> Self {
        Self::Moving {
            forward: false,
            backward: false,
            left: false,
            right: false,
        }
    }
}

impl PlayerEntityController {
    pub fn handle_input(&mut self, input: &PlatformInput) {
        match self {
            PlayerEntityController::Moving {
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

    pub fn move_player(&self, entity: &impl Entity, dt: f32, network: &mut NetworkClient) {
        match self {
            PlayerEntityController::Moving {
                forward,
                backward,
                left,
                right,
            } => {
                let mut render_data = entity.get_mut::<RenderData>().unwrap();
                let sheet_pos = &mut render_data.sprites[0].0.pos;

                let mut dir = Vector2::<f32>::zero();

                if *forward {
                    *sheet_pos = Vector2::new(0, 3);
                    dir.y += 1.;
                }
                if *backward {
                    *sheet_pos = Vector2::new(0, 0);
                    dir.y -= 1.;
                }
                if *left {
                    *sheet_pos = Vector2::new(0, 2);
                    dir.x -= 1.;
                }
                if *right {
                    *sheet_pos = Vector2::new(0, 1);
                    dir.x += 1.;
                }

                let mut pos = entity.get_mut::<Position>().unwrap();
                let speed = 1.5;

                pos.0 += (dir.normalize() * dt * speed).no_nan();

                if *forward || *backward || *left || *right {
                    network.send(&ServerboundSetPlayerPos { pos: *pos });
                }
            }
        }
    }
}

pub struct PlayerInventoryController {
    pub actionbar_slot: u8,
}

impl Default for PlayerInventoryController {
    fn default() -> Self {
        Self { actionbar_slot: 0 }
    }
}

impl PlayerInventoryController {
    pub fn handle_input(&mut self, input: &PlatformInput) {
        match input {
            &PlatformInput::Keyboard { key, state } => {
                if state.is_pressed() {
                    if let Some(slot) = [
                        KeyCode::Digit1,
                        KeyCode::Digit2,
                        KeyCode::Digit3,
                        KeyCode::Digit4,
                        KeyCode::Digit5,
                        KeyCode::Digit6,
                        KeyCode::Digit7,
                        KeyCode::Digit8,
                        KeyCode::Digit9,
                        KeyCode::Digit0,
                    ]
                    .iter()
                    .position(|k| k == &key)
                    {
                        self.actionbar_slot = slot as u8;
                    }
                }
            }
            _ => {}
        }
    }
}
