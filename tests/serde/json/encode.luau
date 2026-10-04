--!strict
local serde = require("@lune/serde")
local source = require("./source")

local decoded = serde.decode("json", source.pretty)
local encoded = serde.encode("json", decoded, false)
assert(encoded == source.encoded, "JSON round-trip did not produce the same result")

local encodedPretty = serde.encode("json", decoded, true)
assert(encodedPretty == source.pretty, "JSON round-trip did not produce the same result (pretty)")

-- Nesting is limited, so that a value too deep to encode is an error

local function nested(depth: number): { any }
	local root: { any } = {}
	local current = root
	for _ = 2, depth do
		local child: { any } = {}
		current[1] = child
		current = child
	end
	return root
end

local function encodes(depth: number): boolean
	return (pcall(function()
		return serde.encode("json", nested(depth))
	end))
end

assert(encodes(64), "JSON encode did not accept 64 nested tables")
assert(not encodes(256), "JSON encode accepted 256 nested tables")
