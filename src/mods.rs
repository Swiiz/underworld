use std::{collections::HashMap, fs, marker::PhantomData, path::PathBuf};

use mlua::{Lua, LuaSerdeExt, Table, Value};
use network::NetworkSide;
use platform::{colored::Colorize, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    world::{
        self,
        terrain::{self, Tile},
    },
    App,
};

#[derive(Serialize, Deserialize)]
pub struct ModManifest {
    id: String,
    entrypoint: PathBuf,
}

pub struct LoadedMod {
    root: PathBuf,
    manifest: ModManifest,
}

impl LoadedMod {
    fn init(&self, lua: &Lua) {
        let source = fs::read_to_string(self.root.join(&self.manifest.entrypoint))
            .unwrap_or_else(|e| panic!("Could not read mod entryoint! {e}"));
        lua.load(source)
            .exec()
            .unwrap_or_else(|e| warn!("Could not compile mod entrypoint, lua errored! {e}"));
    }
}

pub struct ModLoader<S: NetworkSide> {
    _marker: PhantomData<S>,
    lua: Lua,
    mods: HashMap<String, LoadedMod>, // Hashed by manifest id
}

impl<S: NetworkSide + for<'a> Deserialize<'a>> ModLoader<S> {
    pub fn new() -> Self {
        let lua = Lua::new();

        Self {
            lua,
            mods: HashMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn init(app: &mut App<S>) {
        {
            // Init phase

            let lua = &app.mods.lua;

            lua.globals()
                .set(
                    "print",
                    lua.create_function(|_, string: String| {
                        info!("[{}] {}", "LUA".magenta(), string);
                        Ok(())
                    })
                    .unwrap(),
                )
                .unwrap();

            lua.globals().set("networkside", S::ID).unwrap();

            lua.globals()
                .set("Game", {
                    let t = lua.create_table().unwrap();
                    t.set("tile_registry", lua.create_table().unwrap()).unwrap();
                    t
                })
                .unwrap();
        }

        app.mods.load();

        {
            // Post Init-Phase

            let lua = &app.mods.lua;

            lua.globals()
                .get::<_, Table>("Game")
                .unwrap()
                .get::<_, Table>("tile_registry")
                .unwrap()
                .for_each(|_: usize, t: Value| {
                    let tile = lua
                        .from_value::<Tile<S>>(t)
                        .expect("Invalid tile in Game.tile_registry !");
                    app.world.terrain.tile_registry.add(tile);
                    Ok(())
                })
                .unwrap();
        }
    }

    fn load(&mut self) {
        let mods_dir = fs::read_dir("mods").expect("Could not read mods dir!");
        for entry in mods_dir {
            if let Ok((Ok(md), root)) = entry.map(|e| (e.metadata(), e.path())) {
                if md.is_dir() {
                    if let Ok(m) = fs::read_to_string(root.join("manifest.json")) {
                        let manifest = serde_json::from_str::<ModManifest>(&m)
                            .unwrap_or_else(|e| panic!("Invalid manifest at {root:?}! {e}"));
                        let id = manifest.id.clone();
                        info!("Loaded mod: {id}");
                        let loaded_mod = LoadedMod { manifest, root };
                        loaded_mod.init(&self.lua);
                        self.mods.insert(id, loaded_mod);
                    } else {
                        warn!("Could not read mod manifest at: {root:?}")
                    }
                }
            }
        }
    }
}
