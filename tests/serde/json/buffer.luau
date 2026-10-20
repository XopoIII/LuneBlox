--!strict
local serde = require("@lune/serde")

-- A buffer is written as Roblox's HttpService:JSONEncode writes one

local small = buffer.fromstring("hi")
assert(
	serde.encode("json", { b = small }) == [[{"b":{"base64":"aGk=","m":null,"t":"buffer"}}]],
	"A small buffer should be written as base64"
)

-- Bytes that compress well are written with zstd, as zbase64

local repeated = buffer.fromstring(string.rep("LuneBlox", 1000))
local decodedRepeated = serde.decode("json", serde.encode("json", repeated))
assert(
	string.find(serde.encode("json", repeated), [["zbase64":"]], 1, true),
	"zbase64 was not used"
)
assert(typeof(decodedRepeated) == "buffer", "A zbase64 buffer did not decode to a buffer")
assert(
	buffer.tostring(decodedRepeated) == buffer.tostring(repeated),
	"zbase64 round trip changed the bytes"
)

-- Round trips, nested in arrays and objects, with every byte value

local bytes = {}
for i = 0, 255 do
	bytes[#bytes + 1] = string.char(i)
end
local everyByte = buffer.fromstring(table.concat(bytes))
local list: { any } = { 1, everyByte, "three" }
local value = {
	name = "save",
	list = list,
	nested = { inner = { small } },
	empty = buffer.create(0),
}
local roundTrip = serde.decode("json", serde.encode("json", value))
assert(roundTrip.name == "save")
assert(roundTrip.list[1] == 1 and roundTrip.list[3] == "three")
assert(typeof(roundTrip.list[2]) == "buffer")
assert(
	buffer.tostring(roundTrip.list[2]) == buffer.tostring(everyByte),
	"Bytes changed in a round trip"
)
assert(buffer.tostring(roundTrip.nested.inner[1]) == "hi")
assert(typeof(roundTrip.empty) == "buffer" and buffer.len(roundTrip.empty) == 0)

-- The table given to encode is not changed, and a buffer reached twice is written twice

local shared = { a = small, b = small }
assert(
	serde.encode("json", shared)
		== [[{"a":{"base64":"aGk=","m":null,"t":"buffer"},"b":{"base64":"aGk=","m":null,"t":"buffer"}}]]
)
assert(shared.a == small and typeof(shared.b) == "buffer", "Encoding changed the given table")

-- What Roblox wrote decodes, in either form and with the keys in any order

local fromRoblox = serde.decode("json", [[{"t":"buffer","base64":"aGVsbG8=","m":null}]])
assert(typeof(fromRoblox) == "buffer" and buffer.tostring(fromRoblox) == "hello")

local compressed = serde.encode("base64", serde.compress("zstd", "hello hello hello"))
local fromRobloxCompressed =
	serde.decode("json", `\{"m":null,"t":"buffer","zbase64":"{compressed}"}`)
assert(typeof(fromRobloxCompressed) == "buffer")
assert(buffer.tostring(fromRobloxCompressed) == "hello hello hello")

-- JSONC reads them too

assert(
	typeof(serde.decode("jsonc", [[{"m":null,"t":"buffer","base64":"aGk="} // a buffer]]))
		== "buffer"
)

-- Objects that only look alike stay tables

local notBuffer = serde.decode("json", [[{"t":"buffer","base64":"aGk=","extra":1}]])
assert(type(notBuffer) == "table" and notBuffer.extra == 1)
local wrongType = serde.decode("json", [[{"t":"Buffer","base64":"aGk="}]])
assert(type(wrongType) == "table")
local both = serde.decode("json", [[{"t":"buffer","base64":"aGk=","zbase64":"aGk="}]])
assert(type(both) == "table")

-- Bad base64 in a buffer is an error

assert(not pcall(function()
	return serde.decode("json", [[{"m":null,"t":"buffer","base64":"not base64!"}]])
end))

-- A string that is not valid UTF-8 is still an error, not a buffer

assert(not pcall(function()
	return serde.encode("json", { s = string.char(255, 254) })
end))

-- Other formats still refuse buffers

assert(not pcall(function()
	return serde.encode("yaml", { b = small })
end))
