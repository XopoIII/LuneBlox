--!strict
local roblox = require("@lune/roblox") :: any
local BrickColor = roblox.BrickColor
local Color3 = roblox.Color3
local Content = roblox.Content
local Font = roblox.Font
local UDim = roblox.UDim
local UDim2 = roblox.UDim2
local Vector2int16 = roblox.Vector2int16
local Vector3int16 = roblox.Vector3int16

-- A number with a fractional part, given where an integer
-- is expected, is truncated toward zero and not rejected

assert(Color3.fromRGB(1.5, 2.9, 255.9) == Color3.fromRGB(1, 2, 255))

assert(UDim.new(0, 1.5).Offset == 1)
assert(UDim.new(0, -1.5).Offset == -1)

assert(UDim2.fromOffset(1.5, -2.5) == UDim2.fromOffset(1, -2))
assert(UDim2.new(0, 1.5, 0, -2.5) == UDim2.new(0, 1, 0, -2))

assert(Vector2int16.new(1.9, -1.9) == Vector2int16.new(1, -1))
assert(Vector3int16.new(1.9, -1.9, 0.5) == Vector3int16.new(1, -1, 0))

assert(BrickColor.new(1.5) == BrickColor.new(1))
assert(BrickColor.palette(1.5) == BrickColor.palette(1))

assert(Font.fromId(12.5).Family == "rbxassetid://12")

assert(Content.fromAssetId(12.5).Uri == "rbxassetid://12")
assert(Content.fromAssetId(0.5) == Content.none)

-- A number that has no integer to truncate to is still rejected

assert(not pcall(function()
	return UDim.new(0, 0 / 0)
end))
assert(not pcall(function()
	return UDim.new(0, math.huge)
end))
assert(not pcall(function()
	return Vector2int16.new(0 / 0, 0)
end))
assert(not pcall(function()
	return Vector3int16.new(999_999.5, 0, 0)
end))
assert(not pcall(function()
	return Color3.fromRGB(256.5, 0, 0)
end))
assert(not pcall(function()
	return Font.fromId(math.huge)
end))
