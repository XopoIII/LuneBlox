--!strict
local difftext = require("@tests/helpers/difftext")

local changes = difftext.diff("same\nold", "same\nnew", {})
assert(#changes == 3, "Expected one equal, one delete, and one add operation")
assert(changes[1].key == "EQUAL" and changes[1].text == "same")
assert(changes[2].key == "DELETE" and changes[2].text == "old")
assert(changes[3].key == "ADD" and changes[3].text == "new")

local characters = difftext.diff("abc", "adc", { byChar = true })
assert(#characters == 4, "Expected a single character replacement")
assert(characters[1].text == "a" and characters[1].key == "EQUAL")
assert(characters[2].text == "b" and characters[2].key == "DELETE")
assert(characters[3].text == "d" and characters[3].key == "ADD")
assert(characters[4].text == "c" and characters[4].key == "EQUAL")

local rendered = difftext.prettydiff("same\nold", "same\nnew", {
	detailed = true,
	includeLineNumbers = true,
})
assert(string.find(rendered, "same", 1, true), "Pretty diff omitted unchanged text")
assert(string.find(rendered, "\27[91m", 1, true), "Pretty diff omitted deletion styling")
assert(string.find(rendered, "\27[92m", 1, true), "Pretty diff omitted addition styling")
