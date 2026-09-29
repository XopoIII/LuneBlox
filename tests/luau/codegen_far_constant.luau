--!strict
local luau = require("@lune/luau")

-- Native code for a function with more than 256 constants must load and run.
--
-- Luau 0.740's ARM64 register allocator could evict a value it had loaded from a constant and
-- plan to reload it from that constant, even past K255, where no single load reaches it. An
-- asserting build of Luau stopped the whole process there; a release build loaded from [sp]. The
-- shape below is the smallest found to hit it: a string constant far down the table, stored into
-- an upvalue (whose write barrier is a call that evicts it), then an arithmetic instruction
-- whose type check may exit to the VM and so still needs that value. On x64 every offset is
-- encodable and this passes either way.

local PADDING = 300

local lines = {
	"local Upvalue, Decoding",
	"local Sink = {}",
	"return function(Index: number)",
	-- Keep 0.5 at a low constant index so the subtraction below compiles to SUBRK
	"\tSink.Early = 0.5",
}
for index = 1, PADDING do
	table.insert(lines, `\tSink.Field{index} = "Padding{index}"`)
end
table.insert(lines, '\tUpvalue = "Far"')
table.insert(lines, "\tDecoding = 0.5 - Index")
table.insert(lines, "\treturn Upvalue, Decoding, Sink")
table.insert(lines, "end")

local bytecode = luau.compile(table.concat(lines, "\n"), { optimizationLevel = 2, debugLevel = 1 })
local decode = luau.load(bytecode, { debugName = "FarConstant", codegenEnabled = true })()

local upvalue, decoding, sink = decode(1)
assert(upvalue == "Far", `expected the upvalue to be "Far", got {upvalue}`)
assert(decoding == -0.5, `expected 0.5 - 1 to be -0.5, got {decoding}`)
assert(sink[`Field{PADDING}`] == `Padding{PADDING}`, "expected every padding field to be set")

-- A string argument fails the number check and leaves native code for the VM mid-function
local exitedUpvalue, exitedDecoding = (decode :: any)("3")
assert(
	exitedUpvalue == "Far",
	`expected the upvalue to be "Far" after a VM exit, got {exitedUpvalue}`
)
assert(exitedDecoding == -2.5, `expected 0.5 - "3" to be -2.5, got {exitedDecoding}`)
