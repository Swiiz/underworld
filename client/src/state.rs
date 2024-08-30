use common::state::CommonState;
use graphics::ctx::Frame;

use crate::{
    assets::ClientAssets,
    camera::Camera,
    platform::PlatformInput,
    player::PlayerController,
    rendering::draw_entities,
};

pub enum ClientState {
    Connecting,
    Connected {
        camera: Camera,
        //player_entity: EntityId,
        controller: PlayerController,

        common: CommonState,
    },
}

impl ClientState {
    pub fn new_connected(common: CommonState) -> Self {
        Self::Connected {
            camera: Camera::new(),
            controller: PlayerController::default(),
            common,
        }
    }

    pub fn update_player(&mut self, dt: f32) {
        /*
        if let Some(player) = self.entities.get(&self.player_entity) {
            self.controller.update_entity(&player, dt);
        } */
    }

    pub fn update_world(&mut self, dt: f32) {
        //update
    }

    pub fn render(&self, frame: &mut Frame, assets: &ClientAssets) {
        match self {
            ClientState::Connecting => {}
            ClientState::Connected {
                camera,
                controller,
                common,
            } => {
                //self.terrain.render(frame, assets, &self.camera);

                draw_entities(&common.entities, frame, camera);
            }
        }
    }

    pub fn update_camera_pos(&mut self) {
        /*
        if let Some(player) = self.entities.get(&self.player_entity) {
            self.camera.pos = player.get::<Position>().unwrap().0;
        } */
    }

    //TODO: Move into Game Client
    pub fn input(&mut self, event: &PlatformInput, window_size: impl Into<(u32, u32)>) {
        /*
        self.terrain.input(&event, window_size);
        self.controller.handle_platform_input(&event);
        self.camera.handle_scroll(&event); */
    }
}
