--!strict
local roblox = require("@lune/roblox") :: any
local Enum = roblox.Enum

-- Constructors & properties

assert(tostring(Enum) == "Enum")
assert(tostring(Enum.KeyCode) == "Enum.KeyCode")
assert(tostring(Enum.KeyCode.X) == "Enum.KeyCode.X")

-- NOTE: We use the axis enum here since it is unlikely
-- any more will be added to it and change the value
assert(Enum.Axis.X.Name == "X")
assert(Enum.Axis.X.Value == 0)

-- An item answers to the name it had before Roblox renamed it,
-- and is the same item, reporting the name it has now

assert(Enum.KeyCode.Unknown == Enum.KeyCode.None)
assert(Enum.KeyCode.Unknown.Name == "None")
assert(Enum.KeyCode.Unknown.Value == 0)
assert(not pcall(function()
	return Enum.KeyCode.NotAKeyCode
end))

-- Methods

local foundKeyCode = false
for _, enum in Enum:GetEnums() do
	if enum == Enum.KeyCode then
		foundKeyCode = true
		break
	end
end
assert(foundKeyCode, "GetEnums did not contain Enum.KeyCode")

local foundKeyCodeX = false
for _, keyCode in Enum.KeyCode:GetEnumItems() do
	if keyCode == Enum.KeyCode.X then
		foundKeyCodeX = true
		break
	end
end
assert(foundKeyCodeX, "GetEnumItems did not contain X for Enum.KeyCode")

for _, keyCode in Enum.KeyCode:GetEnumItems() do
	assert(
		Enum.KeyCode:FromValue(keyCode.Value) == keyCode,
		"FromValue did not return the KeyCode from its value"
	)
	assert(
		Enum.KeyCode:FromName(keyCode.Name) == keyCode,
		"FromName did not return the KeyCode from its name"
	)
end

assert(
	Enum.KeyCode:FromName("NotAKeyCode") == nil,
	"FromName should return nil for an unknown name"
)
assert(Enum.KeyCode:FromValue(-1) == nil, "FromValue should return nil for a negative value")
assert(Enum.KeyCode:FromValue(1e9) == nil, "FromValue should return nil for an unknown value")
assert(
	Enum.KeyCode:FromValue(Enum.KeyCode.X.Value + 0.5) == Enum.KeyCode.X,
	"FromValue should truncate"
)
assert(
	Enum.KeyCode:FromName("Unknown") == Enum.KeyCode.None,
	"FromName should accept a legacy name"
)

-- Enum items should keep a stable identity, so that they work as table keys and with rawequal

assert(
	rawequal(Enum.KeyCode.X, Enum.KeyCode.X),
	"Reading an enum item twice should give the same userdata"
)
assert(
	rawequal(Enum.KeyCode.X, Enum.KeyCode:FromName("X")),
	"FromName should give the same userdata"
)
assert(
	rawequal(Enum.KeyCode.X, Enum.KeyCode:FromValue(Enum.KeyCode.X.Value)),
	"FromValue should give the same userdata"
)

for _, keyCode in Enum.KeyCode:GetEnumItems() do
	assert(
		rawequal(keyCode, Enum.KeyCode[keyCode.Name]),
		"GetEnumItems should give the same userdata"
	)
end

local keyBinds = { [Enum.KeyCode.X] = "jump" }
assert(keyBinds[Enum.KeyCode.X] == "jump", "Enum items should be usable as table keys")

local part = roblox.Instance.new("Part")
part.Material = Enum.Material.Wood
assert(
	rawequal(part.Material, Enum.Material.Wood),
	"Enum items read from properties should give the same userdata"
)
part:SetAttribute("Material", Enum.Material.Wood)
assert(
	rawequal(part:GetAttribute("Material"), Enum.Material.Wood),
	"Enum items read from attributes should give the same userdata"
)

-- Enums should keep a stable identity too, just like enum items

assert(rawequal(Enum.KeyCode, Enum.KeyCode), "Reading an enum twice should give the same userdata")
assert(rawequal(Enum.KeyCode, Enum.KeyCode.X.EnumType), "EnumType should give the same userdata")

for _, enum in Enum:GetEnums() do
	if enum == Enum.KeyCode then
		assert(rawequal(enum, Enum.KeyCode), "GetEnums should give the same userdata")
	end
end

-- A name an item had before Roblox renamed it gives the same userdata as its current name

assert(
	rawequal(Enum.KeyCode.Unknown, Enum.KeyCode.None),
	"A legacy name should give the same userdata"
)
assert(Enum.KeyCode.Unknown.Name == "None", "A legacy name should report the current name")
