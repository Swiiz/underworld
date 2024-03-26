use cgmath::Vector2;
use graphics::sprite::{Sprite, SpriteRegistry, SpriteSheetData, SpriteSheetKey};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SpriteSheets {
    Characters,
    System,
    BasicTiles,
}

pub const DEBUG_SPRITE: Sprite<SpriteSheets> = Sprite {
    sheet: SpriteSheets::System,
    position: Vector2::new(0, 0),
    size: Vector2::new(1, 1),
};

pub const VOID_SPRITE: Sprite<SpriteSheets> = Sprite {
    sheet: SpriteSheets::System,
    position: Vector2::new(1, 0),
    size: Vector2::new(1, 1),
};

impl SpriteSheetKey for SpriteSheets {
    fn register_spritesheets(registry: &mut SpriteRegistry<Self>)
    where
        Self: Sized,
    {
        registry.register(
            Self::Characters,
            SpriteSheetData {
                path: "mods/vanilla/assets/characters.png".into(),
                sprite_px_size: Vector2 { x: 16, y: 16 },
            },
        );
        registry.register(
            Self::System,
            SpriteSheetData {
                path: "mods/vanilla/assets/system.png".into(),
                sprite_px_size: Vector2 { x: 16, y: 16 },
            },
        );
        registry.register(
            Self::BasicTiles,
            SpriteSheetData {
                path: "mods/vanilla/assets/basictiles.png".into(),
                sprite_px_size: Vector2 { x: 16, y: 16 },
            },
        );
    }
}
