--!strict
-- utf8.graphemes is a Roblox addition that luau-lsp's standard platform does not declare
local graphemes = (utf8 :: any).graphemes

local function collect(s: string, i: number?, j: number?): { string }
	local result = {}
	for first, last in graphemes(s, i, j) do
		table.insert(result, string.sub(s, first, last))
	end
	return result
end

local function same(actual: { string }, expected: { string })
	assert(#actual == #expected, `expected {#expected} clusters, got {#actual}`)
	for index, cluster in expected do
		assert(actual[index] == cluster, `cluster {index} differs`)
	end
end

assert(type(graphemes) == "function", "utf8.graphemes is missing")

-- ASCII is one cluster per byte

same(collect("abc"), { "a", "b", "c" })
same(collect(""), {})

-- A combining mark stays with its base letter

local accented = "e" .. utf8.char(0x301)
same(collect(`x{accented}y`), { "x", accented, "y" })

-- A flag is two regional indicators, a family is joined with zero-width joiners

local flag = utf8.char(0x1F1F3, 0x1F1F4)
local family = utf8.char(0x1F468, 0x200D, 0x1F469, 0x200D, 0x1F467)
same(collect(flag .. family), { flag, family })

-- CRLF is a single cluster

same(collect("a\r\nb"), { "a", "\r\n", "b" })

-- Positions are bytes in the whole string, and i and j narrow it as string.sub does

local s = `a{accented}b`
local positions = {}
for first, last in graphemes(s) do
	table.insert(positions, `{first}-{last}`)
end
assert(table.concat(positions, ",") == "1-1,2-4,5-5", table.concat(positions, ","))

same(collect(s, 2), { accented, "b" })
same(collect(s, 2, 4), { accented })
same(collect(s, -1), { "b" })
same(collect(s, 5, 2), {})

-- Invalid UTF-8 is an error

assert(not pcall(function()
	return graphemes(string.char(255))
end))
