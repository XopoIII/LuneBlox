--!strict
local luau = require("@lune/luau")

assert(type(luau.collect) == "function", "expected `luau.collect` to be a function")

local garbage = table.create(100000) :: { { number } }
for index = 1, 100000 do
	(garbage :: { { number } })[index] = { index }
end
local before = gcinfo()
garbage = nil

luau.collect()
local after = gcinfo()

assert(
	after < before / 2,
	`expected a full collection to free the garbage, went from {before} KB to {after} KB`
)
