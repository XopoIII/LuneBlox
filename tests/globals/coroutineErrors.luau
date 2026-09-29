--!strict
-- A coroutine's error comes back as the value it threw, as on Roblox and in plain Luau

local tag = { code = 7 }

-- coroutine.resume returns a string error exactly as thrown

local stringThread = coroutine.create(function()
	error("boom", 0)
end)
local ok1, err1 = coroutine.resume(stringThread)
assert(ok1 == false, "Resume should return false when the coroutine errors")
assert(err1 == "boom", `Resume should return the thrown string unchanged, got {err1}`)
assert(coroutine.status(stringThread) == "dead", "An errored coroutine should be dead")

-- A string thrown with a level carries only its position, no traceback

local levelThread = coroutine.create(function()
	error("located")
end)
local _, err2 = coroutine.resume(levelThread)
assert(type(err2) == "string", "Resume should return a string error as a string")
assert(string.find(err2, ":%d+: located$") ~= nil, `Error should end in position info, got {err2}`)
assert(string.find(err2, "traceback", 1, true) == nil, "Error should not carry a traceback")

-- coroutine.resume returns a table error as the same table

local tableThread = coroutine.create(function()
	error(tag)
end)
local ok3, err3 = coroutine.resume(tableThread)
assert(ok3 == false, "Resume should return false when the coroutine errors")
assert(rawequal(err3, tag), "Resume should return the thrown table itself")
assert(coroutine.status(tableThread) == "dead", "An errored coroutine should be dead")

-- An error after a yield comes back the same way

local yieldingThread = coroutine.create(function()
	coroutine.yield(1)
	error(tag)
end)
local ok4, value4 = coroutine.resume(yieldingThread)
assert(ok4 == true and value4 == 1, "Resume should return the yielded value")
local ok5, err5 = coroutine.resume(yieldingThread)
assert(ok5 == false, "Resume should return false when the coroutine errors")
assert(rawequal(err5, tag), "Resume should return the table thrown after a yield")
assert(coroutine.status(yieldingThread) == "dead", "An errored coroutine should be dead")

-- Resuming a dead coroutine fails with the message Luau gives

local ok6, err6 = coroutine.resume(yieldingThread)
assert(ok6 == false, "Resume on a dead coroutine should return false")
assert(err6 == "cannot resume dead coroutine", `Unexpected dead coroutine message: {err6}`)

-- coroutine.close on an errored coroutine returns false and the thrown value

local ok7, err7 = coroutine.close(tableThread)
assert(ok7 == false, "Close on an errored coroutine should return false")
assert(rawequal(err7, tag), "Close should return the table the coroutine threw")
assert(coroutine.status(tableThread) == "dead", "A closed coroutine should be dead")

local ok8, err8 = coroutine.close(stringThread)
assert(ok8 == false, "Close on an errored coroutine should return false")
assert(err8 == "boom", `Close should return the thrown string unchanged, got {err8}`)

-- coroutine.wrap propagates the thrown value to the caller

local ok9, err9 = pcall(coroutine.wrap(function()
	error(tag)
end))
assert(ok9 == false, "A wrapped coroutine's error should propagate")
assert(rawequal(err9, tag), "A wrapped coroutine should propagate the thrown table itself")

local ok10, err10 = pcall(coroutine.wrap(function()
	error("boom", 0)
end))
assert(ok10 == false, "A wrapped coroutine's error should propagate")
assert(err10 == "boom", `A wrapped coroutine should propagate the string unchanged, got {err10}`)
