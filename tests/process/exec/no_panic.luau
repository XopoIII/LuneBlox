--!strict
local process = require("@lune/process")

-- Executing a non existent command as a child process
-- should not panic, but should error

-- Wrapped in a function: the new solver checks pcall's arguments against every overload and
-- demands full arity, which a closure sidesteps without changing what is asserted.
local success = pcall(function()
	process.exec("someProgramThatDoesNotExist")
end)
assert(not success, "Spawned a non-existent program")
