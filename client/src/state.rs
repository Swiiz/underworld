use std::cell::OnceCell;

use common::core::spatial::Position;
use ecs::{Entities, Entity, EntityHandle, EntityId};
use graphics::ctx::Frame;

use crate::{
    core::{
        assets::ClientAssets, camera::Camera, network::NetworkClient, platform::PlatformInput,
        rendering::draw_entities, tilemap::ClientTileMap,
    },
    overlays,
    player::{PlayerEntityController, PlayerInventoryController},
};

pub enum ClientState {
    Connecting,
    Connected {
        player_entity: OnceCell<EntityId>,
        camera: Camera,
        pe_controller: PlayerEntityController,
        pi_controller: PlayerInventoryController,

        remote: Remote,
    },
}

pub struct Remote {
    pub terrain: ClientTileMap,
    pub entities: Entities,
}

impl ClientState {
    pub fn update(&mut self, dt: f32, network: &mut NetworkClient) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                pe_controller: controller,
                player_entity,
                remote: Remote { entities, .. },
                ..
            } => {
                if let Some(player) = entities.edit(*player_entity.get().unwrap()) {
                    controller.move_player(&player, dt, network);
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, assets: &ClientAssets, draw_overlay: bool) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                camera,
                player_entity,
                remote: Remote { entities, terrain },
                ..
            } => {
                if let Some(player) = entities.edit(*player_entity.get().unwrap()) {
                    camera.pos = player.get::<Position>().unwrap().0;
                }

                terrain.render(frame, assets, camera, draw_overlay);
                draw_entities(entities, frame, camera);

                if draw_overlay {
                    overlays::play_overlay(frame, &self, assets);
                }
            }
        }
    }

    pub fn input(&mut self, event: &PlatformInput, window_size: impl Into<(u32, u32)>) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                pe_controller,
                pi_controller,
                camera,
                remote: Remote { terrain, .. },
                ..
            } => {
                pe_controller.handle_input(event);
                pi_controller.handle_input(event);
                camera.handle_input(event);
                terrain.input(&event, window_size);
            }
        }
    }
}

impl Remote {
    pub fn sync_entity_position(&mut self, entity: EntityId, pos: Position) {
        if let Some(mut e) = self.entities.edit(entity) {
            e.set(pos);
        }
    }
}
