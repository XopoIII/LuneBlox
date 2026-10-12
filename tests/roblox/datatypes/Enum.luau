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

assert(Enum.KeyCode:FromName("NotAKeyCode") == nil, "FromName should return nil for an unknown name")
assert(Enum.KeyCode:FromValue(-1) == nil, "FromValue should return nil for a negative value")
assert(Enum.KeyCode:FromValue(1e9) == nil, "FromValue should return nil for an unknown value")
assert(Enum.KeyCode:FromValue(Enum.KeyCode.X.Value + 0.5) == Enum.KeyCode.X, "FromValue should truncate")
assert(Enum.KeyCode:FromName("Unknown") == Enum.KeyCode.None, "FromName should accept a legacy name")
