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
        let textures_base_path = PathBuf::from("assets/textures/");
        let textures = Registry::load_whole_json_from_disk_mapped(
            textures_base_path.join("spritesheets.json"),
            |mut v: SpriteSheetSource| {
                v.path = textures_base_path
                    .join(v.path)
                    .to_string_lossy()
                    .to_string();
                v
            },
        );
        let tiles = Registry::load_json_part_from_disk_mapped(
            "assets/terrain/tiles.json",
            "@client",
            |v| load_handles(v, &textures),
        );

        Self {
            common,
            textures,
            tiles,
        }
    }
}
