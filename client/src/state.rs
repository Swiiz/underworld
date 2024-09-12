use std::cell::OnceCell;

use common::core::spatial::Position;
use ecs::{Entities, Entity, EntityId};
use graphics::ctx::Frame;

use crate::{
    core::assets::ClientAssets, core::camera::Camera, core::network::NetworkClient,
    core::platform::PlatformInput, core::rendering::draw_entities, core::tilemap::ClientTileMap,
    player::PlayerController,
};

pub enum ClientState {
    Connecting,
    Connected {
        player_entity: OnceCell<EntityId>,
        camera: Camera,
        controller: PlayerController,

        terrain: ClientTileMap,
        entities: Entities,
    },
}

impl ClientState {
    pub fn update(&mut self, dt: f32, network: &mut NetworkClient) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                controller,
                player_entity,
                entities,
                ..
            } => {
                if let Some(player) = entities.edit(*player_entity.get().unwrap()) {
                    controller.move_player(&player, dt, network);
                }
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, assets: &ClientAssets) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                camera,
                entities,
                terrain,
                ..
            } => {
                terrain.render(frame, assets, camera);
                draw_entities(entities, frame, camera);
            }
        }
    }

    pub fn update_camera_pos(&mut self) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                camera,
                player_entity,
                entities,
                ..
            } => {
                if let Some(player) = entities.edit(*player_entity.get().unwrap()) {
                    camera.pos = player.get::<Position>().unwrap().0;
                }
            }
        }
    }

    pub fn input(&mut self, event: &PlatformInput, window_size: impl Into<(u32, u32)>) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                controller,
                camera,
                terrain,
                ..
            } => {
                controller.handle_input(event);
                camera.handle_input(event);
                terrain.input(&event, window_size);
            }
        }
    }

    pub fn set_entity_position(&mut self, entity: EntityId, pos: Position) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected { entities, .. } => {
                if let Some(mut e) = entities.edit(entity) {
                    e.set(pos);
                }
            }
        }
    }
}
