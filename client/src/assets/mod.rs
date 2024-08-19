use std::{collections::HashMap, path::PathBuf};

use graphics::sprite::{SpriteSheetData, SpriteSheetHandle, SpriteSheetsRegistry};

pub struct GraphicAssets {
    textures: HashMap<String, SpriteSheetHandle>,
}

impl GraphicAssets {
    pub fn load() -> (Self, SpriteSheetsRegistry) {
        let mut textures = HashMap::new();
        let mut reg = SpriteSheetsRegistry::default();

        let base_path = PathBuf::from("assets/textures/");

        serde_json::from_str::<HashMap<String, SpriteSheetData>>(
            std::fs::read_to_string(base_path.join("entries.json"))
                .unwrap()
                .as_str(),
        )
        .expect("Failed to load spritesheets manifest")
        .into_iter()
        .for_each(|(k, mut v)| {
            v.path = base_path.join(v.path).to_string_lossy().to_string();
            textures.insert(k, reg.push(v));
        });

        (Self { textures }, reg)
    }

    pub fn get_texture(&self, name: &str) -> SpriteSheetHandle {
        self.textures.get(name).unwrap().clone()
    }
}
