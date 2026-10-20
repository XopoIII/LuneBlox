--!strict
--!nolint UnknownType
local roblox = require("@lune/roblox") :: any
local Random = roblox.Random
local Vector3 = roblox.Vector3

-- Constructors

assert(typeof(Random.new()) == "Random")
assert(typeof(Random.new(0)) == "Random")
assert(tostring(Random.new(1)) == "Random")
assert(Random.new(1) ~= Random.new(1), "Two generators should be distinct objects")

assert(not pcall(function()
	return Random.new("seed")
end))

-- The same seed gives the same sequence, and a fractional seed is truncated

local a, b, c = Random.new(1234), Random.new(1234), Random.new(1234.9)
for _ = 1, 32 do
	local n = a:NextNumber()
	assert(n == b:NextNumber())
	assert(n == c:NextNumber())
end

-- The sequence is the one math.random gives for the same seed

for _, seed in { 0, 1, 42, -7, 2147483647 } do
	math.randomseed(seed)
	local random = Random.new(seed)
	for _ = 1, 16 do
		assert(
			random:NextNumber() == math.random(),
			`NextNumber differs from math.random for seed {seed}`
		)
		assert(
			random:NextInteger(1, 6) == math.random(1, 6),
			`NextInteger differs from math.random for seed {seed}`
		)
		assert(
			random:NextInteger(-1000, 1000) == math.random(-1000, 1000),
			`NextInteger differs from math.random for seed {seed}`
		)
	end
end

-- NextInteger

local random = Random.new(99)
for _ = 1, 1000 do
	local n = random:NextInteger(-3, 3)
	assert(n >= -3 and n <= 3 and n == math.floor(n))
end
assert(random:NextInteger(5, 5) == 5)
assert(random:NextInteger(1.9, 1.9) == 1, "Fractional bounds should be truncated")
local wide = random:NextInteger(0, 2 ^ 40)
assert(wide >= 0 and wide <= 2 ^ 40)

local ok, err = pcall(function()
	return random:NextInteger(2, 1)
end)
assert(not ok and string.find(tostring(err), "interval is empty", 1, true), tostring(err))

-- NextNumber

for _ = 1, 1000 do
	local n = random:NextNumber()
	assert(n >= 0 and n < 1)
	local m = random:NextNumber(-5, 5)
	assert(m >= -5 and m < 5)
end

-- NextUnitVector

for _ = 1, 100 do
	local v = random:NextUnitVector()
	assert(typeof(v) == "Vector3")
	assert(math.abs(v.Magnitude - 1) < 1e-5, "NextUnitVector should have a length of one")
end
assert(Vector3.new(1, 0, 0).Magnitude == 1)

-- Clone continues from the same state, independently of the original

local original = Random.new(5)
original:NextNumber()
local clone = original:Clone()
assert(typeof(clone) == "Random" and clone ~= original)
local first, second = original:NextNumber(), original:NextNumber()
assert(clone:NextNumber() == first, "A clone should continue from the original's state")
assert(clone:NextNumber() == second, "Advancing the original should not advance the clone")

-- Shuffle

local items = {}
for i = 1, 50 do
	items[i] = i
end
Random.new(3):Shuffle(items)
local seen = {}
local moved = false
for i, v in items do
	assert(not seen[v], "Shuffle should keep every item once")
	seen[v] = true
	moved = moved or v ~= i
end
assert(#items == 50 and moved, "Shuffle should permute the items")

local again = {}
for i = 1, 50 do
	again[i] = i
end
Random.new(3):Shuffle(again)
for i = 1, 50 do
	assert(again[i] == items[i], "Shuffle with the same seed should give the same order")
end

Random.new(1):Shuffle({})

assert(not pcall(function()
	Random.new(1):Shuffle(table.freeze({ 1, 2, 3 }))
end))
