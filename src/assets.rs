use cgmath::Vector2;
use graphics::sprite::{SpriteRegistry, SpriteSheetData, SpriteSheetKey};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SpriteSheets {
    Characters,
}

impl SpriteSheetKey for SpriteSheets {
    fn register_spritesheets(registry: &mut SpriteRegistry<Self>)
    where
        Self: Sized,
    {
        registry.register(
            Self::Characters,
            SpriteSheetData {
                path: "assets/characters.png".into(),
                sprite_px_size: Vector2 { x: 16, y: 16 },
            },
        );
    }
}
