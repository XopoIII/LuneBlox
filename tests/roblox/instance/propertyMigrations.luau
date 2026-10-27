--!strict
local roblox = require("@lune/roblox") :: any
local BrickColor = roblox.BrickColor
local Content = roblox.Content
local Instance = roblox.Instance
local UDim = roblox.UDim

-- A legacy BrickColor reads through Color, including when Color came from a file.
local part = Instance.new("Part")
assert(part.BrickColor == BrickColor.new(part.Color))
part.Color = BrickColor.Red().Color
assert(part.BrickColor == BrickColor.Red())

-- Writing the legacy property migrates it instead of leaving competing values behind.
part.BrickColor = BrickColor.Blue()
assert(part.Color == BrickColor.Blue().Color)
assert(part.BrickColor == BrickColor.Blue())
local partRoundTrip = roblox.deserializeModel(roblox.serializeModel({ part }))[1]
assert(partRoundTrip.Color == BrickColor.Blue().Color)
assert(partRoundTrip.BrickColor == BrickColor.Blue())

-- ContentId and Content convert in both directions.
local decal = Instance.new("Decal")
assert(decal.Texture == "")
decal.Texture = "rbxassetid://123"
assert(decal.TextureContent == Content.fromUri("rbxassetid://123"))
local decalRoundTrip = roblox.deserializeModel(roblox.serializeModel({ decal }))[1]
assert(tostring(decalRoundTrip.Texture) == "rbxassetid://123")
assert(decalRoundTrip.TextureContent == Content.fromUri("rbxassetid://123"))
decal.TextureContent = Content.fromUri("rbxassetid://456")
assert(tostring(decal.Texture) == "rbxassetid://456")
decal.TextureContent = Content.none
assert(decal.Texture == "")

-- Migrations with several destinations update every replacement property.
local corner = Instance.new("UICorner")
corner.CornerRadius = UDim.new(0, 8)
assert(corner.BottomLeftRadius == UDim.new(0, 8))
assert(corner.BottomRightRadius == UDim.new(0, 8))
assert(corner.TopLeftRadius == UDim.new(0, 8))
assert(corner.TopRightRadius == UDim.new(0, 8))
corner.BottomLeftRadius = UDim.new(0, 12)
assert(corner.CornerRadius == UDim.new(0, 12))
