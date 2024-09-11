pub mod assets;
pub mod network;
pub mod state;

use std::{net::SocketAddr, time::Instant};

use assets::ServerAssets;
use cgmath::{Vector2, Zero};
use common::{
    core::{spatial::Position, EntityKind},
    logger::info,
    network::proto::{
        extra::{CommonPing, ServerboundDisconnect},
        login::{ClientboundLoginSuccess, ServerboundLoginStart},
        play::{ClientboundRemoveEntity, ClientboundSpawnEntity, ServerboundSetPlayerPos},
        SyncComponentSelection,
    },
    utils::timer::Timer,
};
use ecs::Entity;
use network::{remote::NetRemoteClient, NetworkServer};
use state::ServerState;

pub const SERVER_UPS_CAP: f32 = 60.;

pub fn run_server() {
    let mut server = GameServer::new();

    loop {
        server.update();

        std::time::Duration::from_secs_f32(1. / SERVER_UPS_CAP)
            .checked_sub(server.timer.last_update.elapsed())
            .map(|d| std::thread::sleep(d));
    }
}

pub struct GameServer {
    assets: ServerAssets,
    timer: Timer,
    network: NetworkServer,

    state: ServerState,
}

impl GameServer {
    pub fn new() -> Self {
        let assets = ServerAssets::load();
        let timer = Timer::new();
        let network = NetworkServer::new();
        let state = ServerState::new(&assets);

        Self {
            assets,
            timer,
            network,
            state,
        }
    }

    pub fn update(&mut self) {
        let _dt = self.timer.update_dt();

        self.network.listen_for_connections();

        self.network.handle_packets(|network, addr, packet| {
            if let Some(CommonPing { time }) = packet.try_decode::<CommonPing>() {
                if let Some(NetRemoteClient { last_packet, .. }) = network.get_remote_mut(&addr) {
                    *last_packet = Instant::now();
                    network.send_to([addr], &CommonPing { time });
                }
            }

            if let Some(packet) = packet.try_decode::<ServerboundDisconnect>() {
                info!("Client disconnected: {:?} for reason: {:?}", addr, packet);
                network.disconnect(&addr);
                return;
            }

            if network.is_connecting(&addr) {
                if let Some(ServerboundLoginStart { username }) = packet.try_decode() {
                    connect_player(network, addr, username, &mut self.state);
                }
            } else {
                if let Some(ServerboundSetPlayerPos { pos }) = packet.try_decode() {
                    self.state.set_player_position(&addr, pos, network);
                }
            }
        });

        self.network.handle_disconnections(|network, addr, client| {
            info!("Client disconnected: {:?}", addr);

            self.state.entities.edit(client.entity).unwrap().despawn();
            network.broadcast(&ClientboundRemoveEntity {
                entity: client.entity.into(),
            });
        });

        // Send data, update server state

        self.network.flush();
    }
}

fn connect_player(
    network: &mut NetworkServer,
    addr: SocketAddr,
    username: String,
    state: &mut ServerState,
) {
    info!("Client connected: {:?}", username);

    // Save the ecs state before spawning the player
    let ecs_state = state.entities.save();
    let terrain = state.terrain.clone();

    let client_entity = state
        .entities
        .spawn()
        .set(EntityKind::Player)
        .set(Position(Vector2::zero()))
        .id();

    network.accept_connection(addr, NetRemoteClient::new(username, client_entity));

    network.send_to([addr], &ClientboundLoginSuccess { terrain, ecs_state });

    network.broadcast(&ClientboundSpawnEntity {
        entity: client_entity.into(),
        state: state
            .entities
            .save_entity::<SyncComponentSelection>(client_entity),
    });
}
