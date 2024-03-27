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

use super::api::ModsApi;

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
    fn init<S: NetworkSide>(&self, api: &ModsApi<S>) {
        let source = fs::read_to_string(self.root.join(&self.manifest.entrypoint))
            .unwrap_or_else(|e| panic!("Could not read mod entryoint! {e}"));
        api.exec_lua(source)
            .unwrap_or_else(|e| warn!("Could not compile mod entrypoint, lua errored! {e}"));
    }
}

pub struct ModLoader<S: NetworkSide> {
    pub api: ModsApi<S>,
    mods: HashMap<String, LoadedMod>, // Hashed by manifest id
}

impl<S: NetworkSide> ModLoader<S> {
    pub fn new() -> Self {
        let api = ModsApi::new();

        let mods_dir = fs::read_dir("mods").expect("Could not read mods dir!");
        let mut mods = HashMap::new();
        for entry in mods_dir {
            if let Ok((Ok(md), root)) = entry.map(|e| (e.metadata(), e.path())) {
                if md.is_dir() {
                    if let Ok(m) = fs::read_to_string(root.join("manifest.json")) {
                        let manifest = serde_json::from_str::<ModManifest>(&m)
                            .unwrap_or_else(|e| panic!("Invalid manifest at {root:?}! {e}"));
                        let id = manifest.id.clone();
                        info!("Loaded mod: {id}");
                        let loaded_mod = LoadedMod { manifest, root };
                        loaded_mod.init(&api);
                        mods.insert(id, loaded_mod);
                    } else {
                        warn!("Could not read mod manifest at: {root:?}")
                    }
                }
            }
        }

        Self { api, mods }
    }
}
