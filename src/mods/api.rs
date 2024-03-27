use std::marker::PhantomData;

use mlua::{Error, Lua, LuaSerdeExt, Table, Value};
use network::NetworkSide;
use platform::{
    colored::{ColoredString, Colorize},
    info, warn,
};
use serde::{de::DeserializeOwned, Deserialize};

use crate::world::terrain::Tile;

pub struct ModsApi<S: NetworkSide> {
    _marker: PhantomData<S>,
    lua: Lua,
}

fn lua_print_prefix() -> String {
    format!("[{}]", "LUA".magenta())
}

impl<S: NetworkSide> ModsApi<S> {
    pub fn new() -> Self {
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

        lua.globals().set("networkside", S::ID).unwrap();

        lua.globals()
            .set("Game", {
                let t = lua.create_table().unwrap();
                t.set("tile_registry", lua.create_table().unwrap()).unwrap();
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
                if r.is_err() {
                    warn!(
                        "{} Invalid entry in Game.tile_registry!",
                        lua_print_prefix()
                    );
                }
                r.ok()
            })
    }

    pub fn exec_lua(&self, source: String) -> Result<(), Error> {
        self.lua.load(source).exec()
    }
}
