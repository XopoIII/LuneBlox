--!strict
local net = require("@lune/net")

local encoded = net.urlEncode("🚀 This string will be encoded.")
assert(encoded == "%F0%9F%9A%80%20This%20string%20will%20be%20encoded.")
