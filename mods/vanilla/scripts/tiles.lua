

local tiles = {
  {
    -- Debug
    id = "vanilla:debug",
    client_sprite = { sheet = "System", position = Vec2(0, 0), size = Unit.Vec2}
  },
  {
    -- Dirt
    id = "vanilla:dirt",
    client_sprite = { sheet = "BasicTiles", position = Vec2(7, 1), size = Unit.Vec2}
  },
  {
    -- Diamond
    id = "vanilla:diamond",
    client_sprite = { sheet = "BasicTiles", position = Vec2(6, 1), size = Unit.Vec2}
  },
  {
    -- Gold
    id = "vanilla:gold",
    client_sprite = { sheet = "BasicTiles", position = Vec2(6, 2), size = Unit.Vec2}
  }
}

for _, t in pairs(tiles) do
  table.insert(Game.tile_registry, t)
end