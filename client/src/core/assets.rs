use std::{collections::HashMap, path::PathBuf};

use common::{
    assets::CommonAssets,
    utils::{handle::StaticHandle, registry::Registry},
};
use graphics::sprite::{SpriteSheetHandle, SpriteSheetSource};

use crate::core::tilemap::{load_handles, ClientTileData};

pub type TexturesRegistry = Registry<SpriteSheetSource, SpriteSheetHandle>;
pub type ClientTileRegistry = Registry<ClientTileData>;

pub struct ClientAssets {
    pub common: CommonAssets,
    pub textures: TexturesRegistry,
    pub tiles: ClientTileRegistry,
}

impl ClientAssets {
    pub fn load() -> Self {
        let common = CommonAssets::load();
        let textures = load_textures();
        let tiles = load_tiles(&textures);

        Self {
            common,
            textures,
            tiles,
        }
    }
}

fn load_textures() -> TexturesRegistry {
    let mut textures = TexturesRegistry::new();

    let base_path = PathBuf::from("assets/textures/");

    serde_json::from_str::<HashMap<String, SpriteSheetSource>>(
        std::fs::read_to_string(base_path.join("spritesheets.json"))
            .unwrap()
            .as_str(),
    )
    .expect("Failed to load spritesheets manifest")
    .into_iter()
    .for_each(|(k, mut data)| {
        data.path = base_path.join(data.path).to_string_lossy().to_string();
        textures.register(k, data);
    });

    textures
}

pub struct SpriteAsset {}

fn load_tiles(textures: &TexturesRegistry) -> ClientTileRegistry {
    let mut tiles = Registry::new();

    let base_path = PathBuf::from("assets/terrain/");

    let mut entries = serde_json::from_str::<HashMap<String, HashMap<String, serde_json::Value>>>(
        std::fs::read_to_string(base_path.join("tiles.json"))
            .unwrap()
            .as_str(),
    )
    .expect("Failed to load tiles manifest")
    .into_iter()
    .map(|(k, mut v)| {
        let v = v
            .remove("@client")
            .expect("Failed to load common tile manifest");
        (
            k,
            serde_json::from_value::<ClientTileData<StaticHandle>>(v)
                .expect("Failed to parse common tile"),
        )
    })
    .collect::<Box<_>>();

    // REALY IMPORTANT TO ENSURE TILES ARE ORDERED THE SAME WAY CLIENTSIDE AND SERVERSIDE
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (k, v) in entries {
        tiles.register(k, load_handles(v, textures));
    }

    tiles
}
