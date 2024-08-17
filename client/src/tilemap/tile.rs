use graphics::sprite::Sprite;

pub struct Tile {
    pub sprite: Sprite,
}

pub struct TileRegistry {
    pub tiles: Vec<Tile>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TileId(pub usize);

impl TileRegistry {
    pub fn new() -> Self {
        Self { tiles: vec![] }
    }

    pub fn add_tile(&mut self, tile: Tile) -> TileId {
        self.tiles.push(tile);
        TileId(self.tiles.len() - 1)
    }

    pub fn get(&self, id: TileId) -> &Tile {
        self.tiles
            .get(id.0)
            .unwrap_or_else(|| panic!("Invalid tile id: {}", id.0))
    }
}
