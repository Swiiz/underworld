use std::{collections::HashMap, path::PathBuf};

use log::debug;

use crate::{tilemap::tile::Tile, utils::registry::Registry};

pub struct CommonAssets {
    pub tiles: Registry<Tile>,
}

impl CommonAssets {
    pub fn load() -> Self {
        let tiles = load_tiles();

        Self { tiles }
    }
}

fn load_tiles() -> Registry<Tile> {
    let mut tiles = Registry::new();

    let base_path = PathBuf::from("assets/terrain/");

    serde_json::from_str::<HashMap<String, HashMap<String, serde_json::Value>>>(
        std::fs::read_to_string(base_path.join("tiles.json"))
            .unwrap()
            .as_str(),
    )
    .expect("Failed to load tiles manifest")
    .into_iter()
    .map(|(k, mut v)| {
        let v = v
            .remove("@common")
            .expect("Failed to load common tile manifest");
        (
            k,
            serde_json::from_value::<Tile>(v).expect("Failed to parse common tile"),
        )
    })
    .for_each(|(k, v)| {
        debug!("Registering tile: {}", k);
        tiles.register(k, v);
    });

    tiles
}
// ClientTileData<StaticHandle>>
