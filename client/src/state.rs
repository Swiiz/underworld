use std::cell::OnceCell;

use common::{core::spatial::Position, logger::debug, state::CommonState};
use ecs::{Entity, EntityId};
use graphics::ctx::Frame;

use crate::{
    assets::ClientAssets, camera::Camera, network::NetworkClient, platform::PlatformInput,
    player::PlayerController, rendering::draw_entities,
};

pub enum ClientState {
    Connecting,
    Connected {
        player_entity: OnceCell<EntityId>,
        camera: Camera,
        controller: PlayerController,

        common: CommonState,
    },
}

impl ClientState {
    pub fn update(&mut self, dt: f32, network: &mut NetworkClient) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                common,
                controller,
                player_entity,
                ..
            } => {
                if let Some(player) = common.entities.edit(*player_entity.get().unwrap()) {
                    controller.move_player(&player, dt, network);
                }
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, assets: &ClientAssets) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected { camera, common, .. } => {
                //self.terrain.render(frame, assets, &self.camera);

                draw_entities(&common.entities, frame, camera);
            }
        }
    }

    pub fn update_camera_pos(&mut self) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                camera,
                common,
                player_entity,
                ..
            } => {
                if let Some(player) = common.entities.edit(*player_entity.get().unwrap()) {
                    camera.pos = player.get::<Position>().unwrap().0;
                }
            }
        }
    }

    pub fn input(&mut self, event: &PlatformInput, window_size: impl Into<(u32, u32)>) {
        //self.terrain.input(&event, window_size);
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                controller, camera, ..
            } => {
                controller.handle_input(event);
                camera.handle_input(event);
            }
        }
    }

    pub fn set_entity_position(&mut self, entity: EntityId, pos: Position) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected { common, .. } => {
                if let Some(mut e) = common.entities.edit(entity) {
                    e.set(pos);
                }
            }
        }
    }
}
