use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use mlua::{Error, Lua, LuaSerdeExt, Table, Value};
use network::NetworkSide;
use platform::{
    colored::{ColoredString, Colorize},
    info, warn,
};
use serde::{de::DeserializeOwned, Deserialize};

use crate::world::terrain::Tile;

use super::loader::MOD_SCRIPTS_FOLDER;

pub struct ModsApi<S: NetworkSide> {
    _marker: PhantomData<S>,
    lua: Lua,
}

fn lua_print_prefix() -> String {
    format!("[{}]", "LUA".magenta())
}

impl<S: NetworkSide> ModsApi<S> {
    pub fn new(mods_path: PathBuf) -> Self {
        let lua = Lua::new();

        lua.globals()
            .set(
                "print",
                lua.create_function(|_, string: String| {
                    info!("{} {}", lua_print_prefix(), string);
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        lua.globals()
            .set(
                "require",
                lua.create_function(move |lua, mut path: String| {
                    if !path.ends_with(".lua") {
                        path += ".lua";
                    }
                    path = path.replace(
                        ":",
                        &(std::path::MAIN_SEPARATOR_STR.to_string() + MOD_SCRIPTS_FOLDER),
                    );
                    let path = mods_path.join(path);
                    if let Ok(source) = fs::read_to_string(path.clone()) {
                        lua.load(source).exec().unwrap_or_else(|e| {
                            warn!(
                                "{} Tried loading invalid lua module! {path:?}: {e}",
                                lua_print_prefix(),
                            )
                        })
                    } else {
                        warn!(
                            "{} Tried loading non existing lua module! {path:?}",
                            lua_print_prefix()
                        );
                    }
                    Ok(())
                })
                .unwrap(),
            )
            .unwrap();

        lua.globals()
            .set("Game", {
                let t = lua.create_table().unwrap();
                t.set("tile_registry", lua.create_table().unwrap()).unwrap();
                t.set("networkside", S::ID).unwrap();
                t
            })
            .unwrap();

        Self {
            _marker: PhantomData,
            lua,
        }
    }

    /// Game.tile_registry is an array of Tile
    pub fn tile_registry(&self) -> impl Iterator<Item = Tile<S>> + '_ {
        self.lua
            .globals()
            .get::<_, Table>("Game")
            .unwrap()
            .get::<_, Table>("tile_registry")
            .unwrap()
            .pairs()
            .map(|r: Result<(usize, _), _>| self.lua.from_value(r.unwrap().1))
            .filter_map(|r| {
                if let Err(e) = r {
                    warn!(
                        "{} Invalid entry in Game.tile_registry! {e}",
                        lua_print_prefix()
                    );
                    None
                } else {
                    r.ok()
                }
            })
    }

    pub fn exec_lua(&self, source: String) -> Result<(), Error> {
        self.lua.load(source).exec()
    }
}
