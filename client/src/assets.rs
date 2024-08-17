use graphics::{
    maths::Vector2,
    sprite::{SpriteSheetData, SpriteSheetHandle, SpriteSheetsRegistry},
};

pub struct Assets {
    pub sprite_sheets: SpriteSheets,
}

pub struct SpriteSheets {
    pub debug: SpriteSheetHandle,
    pub characters: SpriteSheetHandle,
}

pub fn load_sprite_sheets() -> (SpriteSheets, SpriteSheetsRegistry) {
    let mut reg = SpriteSheetsRegistry::default();

    let debug = reg.push(SpriteSheetData {
        path: "assets/debug.png".to_string(),
        sprite_px_size: Vector2::new(16, 16),
    });

    let characters = reg.push(SpriteSheetData {
        path: "assets/characters.png".to_string(),
        sprite_px_size: Vector2::new(16, 16),
    });

    let lookup = SpriteSheets { characters, debug };

    (lookup, reg)
}
