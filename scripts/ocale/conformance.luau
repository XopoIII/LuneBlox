--!nocheck
-- Runs unchanged in LuneBlox and in Roblox Open Cloud Luau Execution (OCALE).
-- Selene skips this file because the Roblox branch uses engine globals.

local environment = getfenv()
local runningInRoblox = environment.game ~= nil
local api = {}

if runningInRoblox then
	api.BrickColor = environment.BrickColor
	api.Color3 = environment.Color3
	api.Content = environment.Content
	api.Enum = environment.Enum
	api.Instance = environment.Instance
	api.Random = environment.Random
	api.UDim = environment.UDim
	api.Vector3 = environment.Vector3
	api.Vector3int16 = environment.Vector3int16
	api.jsonEncode = function(value)
		return environment.game:GetService("HttpService"):JSONEncode(value)
	end
else
	local roblox = require("@lune/roblox")
	local serde = require("@lune/serde")
	for _, name in
		{
			"BrickColor",
			"Color3",
			"Content",
			"Enum",
			"Instance",
			"Random",
			"UDim",
			"Vector3",
			"Vector3int16",
		}
	do
		api[name] = roblox[name]
	end
	api.jsonEncode = function(value)
		return serde.encode("json", value)
	end
end

local results = {}
local function record(name, value)
	table.insert(results, { name = name, value = value })
end

local vector = api.Vector3.new(1, 2, 3)
record("vector_dot", vector:Dot(api.Vector3.new(4, 5, 6)))
record("vector_cross", tostring(vector:Cross(api.Vector3.new(4, 5, 6))))
record("integer_truncation", tostring(api.Vector3int16.new(1.9, -2.9, 3.1)))

local part = api.Instance.new("Part")
part.Color = api.BrickColor.Red().Color
record("brick_color_read", part.BrickColor.Name)
part.BrickColor = api.BrickColor.Blue()
record("brick_color_write", part.Color:ToHex())

local decal = api.Instance.new("Decal")
decal.Texture = "rbxassetid://123"
record("content_id_write", decal.TextureContent.Uri)
decal.TextureContent = api.Content.fromUri("rbxassetid://456")
record("content_id_read", decal.Texture)

local corner = api.Instance.new("UICorner")
corner.CornerRadius = api.UDim.new(0, 8)
record("multi_property_migration", corner.TopRightRadius.Offset)

record("enum_legacy_alias", api.Enum.KeyCode.Unknown == api.Enum.KeyCode.None)
record("enum_from_name", api.Enum.KeyCode:FromName("A").Value)
record("enum_from_value", api.Enum.KeyCode:FromValue(97).Name)

local random = api.Random.new(12345)
record("random_sequence", {
	random:NextInteger(1, 1000000),
	random:NextInteger(1, 1000000),
	random:NextInteger(1, 1000000),
})

local graphemes = {}
for first, last in utf8.graphemes("a" .. utf8.char(0x301) .. "b") do
	table.insert(graphemes, { first, last })
end
record("utf8_graphemes", graphemes)

record("buffer_json", api.jsonEncode(buffer.fromstring("LuneBlox")))

print("LUNEBLOX_CONFORMANCE:" .. api.jsonEncode(results))
