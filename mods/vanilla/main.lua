print("Vanilla mod loaded from " .. networkside)

local tiles = {
  {
    -- Debug
    id = "vanilla:debug",
    client_sprite = { sheet = "System", position = { x = 0, y = 0 }, size =  {x = 1, y = 1 }}
  },
  {
    -- Dirt
    id = "vanilla:dirt",
    client_sprite = { sheet = "BasicTiles", position = { x = 7, y = 1 }, size = { x = 1, y = 1 }}
  },
  {
    -- Diamond
    id = "vanilla:diamond",
    client_sprite = { sheet = "BasicTiles", position = { x = 6, y = 1 }, size = { x = 1, y = 1 }}
  },
  {
    -- Gold
    id = "vanilla:gold",
    client_sprite = { sheet = "BasicTiles", position = { x = 6, y = 2 }, size = { x = 1, y = 1 }}
  }
}

for _, t in pairs(tiles) do
  table.insert(Game.tile_registry, t)
end