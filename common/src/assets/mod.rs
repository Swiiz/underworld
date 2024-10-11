use crate::{tilemap::tile::Tile, utils::registry::Registry};

pub struct CommonAssets {
    pub tiles: Registry<Tile>,
}

impl CommonAssets {
    pub fn load() -> Self {
        let tiles = Registry::load_json_part_from_disk("assets/terrain/tiles.json", "@common");

        Self { tiles }
    }
}
