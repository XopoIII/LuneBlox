--!strict
-- NOTE: luau-lsp warns without this for the roblox types below
--!nolint UnknownType
local roblox = require("@lune/roblox") :: any

-- The global is Roblox's Enum, with no require

assert(Enum ~= nil, "Missing 'Enum' global")
assert(typeof(Enum) == "Enums", `Global 'Enum' should be Enums, got '{typeof(Enum)}'`)
assert(tostring(Enum) == "Enum")
assert(typeof(Enum.KeyCode) == "Enum")
assert(typeof(Enum.KeyCode.ButtonA) == "EnumItem")

-- Items carry their name, value and enum

assert(Enum.KeyCode.ButtonA.Name == "ButtonA")
assert(Enum.KeyCode.ButtonA.Value == 1002)
assert(rawequal(Enum.KeyCode.ButtonA.EnumType, Enum.KeyCode))
assert(Enum.UserInputState.Begin.Name == "Begin")
assert(Enum.UserInputState.Begin.Value == 0)

-- Items compare by identity: the same item is equal, a different one is not

assert(rawequal(Enum.KeyCode.ButtonA, Enum.KeyCode.ButtonA))
assert(Enum.KeyCode.ButtonA ~= Enum.KeyCode.ButtonB)
assert(Enum.UserInputState.Begin ~= Enum.UserInputState.End)

-- It is the roblox library's Enum, so an item from either is the same item

assert(rawequal(Enum, roblox.Enum), "The global should be the roblox library's Enum")
assert(rawequal(Enum.KeyCode, roblox.Enum.KeyCode))
assert(rawequal(Enum.KeyCode.ButtonA, roblox.Enum.KeyCode.ButtonA))
assert(Enum.KeyCode.ButtonA == roblox.Enum.KeyCode.ButtonA)
assert(Enum.KeyCode.ButtonA ~= roblox.Enum.KeyCode.ButtonB)

local part = roblox.Instance.new("Part")
part.Material = Enum.Material.Wood
assert(rawequal(part.Material, Enum.Material.Wood), "A property should read back the global's item")

-- GetEnumItems gives the items of the enum, and only those

local items = Enum.KeyCode:GetEnumItems()
assert(#items > 0, "GetEnumItems should give items")
local foundButtonA = false
for _, item in items do
	assert(rawequal(item.EnumType, Enum.KeyCode), `{item} is not a KeyCode`)
	if rawequal(item, Enum.KeyCode.ButtonA) then
		foundButtonA = true
	end
end
assert(foundButtonA, "GetEnumItems did not contain ButtonA for Enum.KeyCode")
assert(#Enum.UserInputState:GetEnumItems() ~= #items)

-- An unknown enum or item errors, with the roblox library's message

local function assertErrors(read: () -> any, message: string)
	local ok, err = pcall(read)
	assert(not ok, `Reading an unknown member should error with: {message}`)
	assert(string.find(tostring(err), message, 1, true), `Expected '{message}', got: {err}`)
end

local unknownItem = "The enum item 'NotAKeyCode' does not exist for enum 'KeyCode'"
assertErrors(function()
	return Enum.KeyCode.NotAKeyCode
end, unknownItem)
assertErrors(function()
	return roblox.Enum.KeyCode.NotAKeyCode
end, unknownItem)

local unknownEnum = "The enum 'NotAnEnum' does not exist"
assertErrors(function()
	return Enum.NotAnEnum
end, unknownEnum)
assertErrors(function()
	return roblox.Enum.NotAnEnum
end, unknownEnum)

-- FromName finds nothing for an unknown name, rather than erroring

assert(rawequal(Enum.KeyCode:FromName("ButtonA"), Enum.KeyCode.ButtonA))
assert(Enum.KeyCode:FromName("NotAKeyCode") == nil)

-- Enum cannot be written to, as on Roblox

assert(not pcall(function()
	(Enum :: any).KeyCode = 1
end))
