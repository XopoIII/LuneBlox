--!strict
--[[
	Measures the Luau VM on a handful of small kernels, so that a Luau upgrade or a fast
	flag change is checked for performance regressions rather than discovered by feel.

	Run from the repository root:

		luneblox run scripts/benchmark.luau              -- JIT, the default
		LUNE_LUAU_JIT=0 luneblox run scripts/benchmark.luau

	One line per kernel: name, then median milliseconds per run. Machine-readable, so the
	benchmark workflow can store the output as an artifact and compare it across releases.
]]

local RUNS = 5

type Kernel = {
	name: string,
	run: () -> (),
}

local function makeCounter(): () -> number
	local count = 0
	return function()
		count += 1
		return count
	end
end

local MethodTable = {}
MethodTable.__index = MethodTable

function MethodTable.double(self: { value: number }): number
	return self.value * 2
end

local function encode(value: any): string
	local kind = type(value)
	if kind == "table" then
		local parts = { "{" }
		for key, entry in value :: { [string]: any } do
			table.insert(parts, string.format("%q:%s,", tostring(key), encode(entry)))
		end
		table.insert(parts, "}")
		return table.concat(parts)
	elseif kind == "string" then
		return string.format("%q", value)
	else
		return tostring(value)
	end
end

local function makePayload(depth: number): { [string]: any }
	if depth == 0 then
		return { leaf = "value", count = depth }
	end
	return { left = makePayload(depth - 1), right = makePayload(depth - 1), count = depth }
end

local function fib(limit: number): number
	if limit < 2 then
		return limit
	end
	return fib(limit - 1) + fib(limit - 2)
end

local kernels: { Kernel } = {
	{
		name = "fib(24) recursive calls",
		run = function()
			assert(fib(24) == 46368)
		end,
	},
	{
		name = "array build and sum",
		run = function()
			local values = table.create(100_000, 0)
			for index = 1, 100_000 do
				values[index] = index * 3
			end
			local sum = 0
			for _, value in values do
				sum += value
			end
			assert(sum == 15000150000)
		end,
	},
	{
		name = "string format and concat",
		run = function()
			local parts = table.create(10_000, "")
			for index = 1, 10_000 do
				parts[index] = string.format("part-%d", index)
			end
			assert(#table.concat(parts, ",") > 0)
		end,
	},
	{
		name = "closures over upvalues",
		run = function()
			local advanceA = makeCounter()
			local advanceB = makeCounter()
			local advanceC = makeCounter()
			local total = 0
			for _ = 1, 50_000 do
				total += advanceA() + advanceB() + advanceC()
			end
			assert(total > 0)
		end,
	},
	{
		name = "coroutine churn",
		run = function()
			for _ = 1, 10_000 do
				local worker = coroutine.create(function(value: number)
					coroutine.yield(value + 1)
					return value + 2
				end)
				local _, mid = coroutine.resume(worker, 1)
				local _, final = coroutine.resume(worker)
				assert(mid == 2 and final == 3)
			end
		end,
	},
	{
		name = "method calls via __index",
		run = function()
			local object = setmetatable({ value = 1 }, MethodTable) :: any
			local total = 0
			for _ = 1, 100_000 do
				total += object:double()
			end
			assert(total == 200_000)
		end,
	},
	{
		name = "recursive serializer",
		run = function()
			assert(#encode(makePayload(12)) > 0)
		end,
	},
}

local function measure(kernel: Kernel): number
	local times = table.create(RUNS, 0)
	for run = 1, RUNS do
		local start = os.clock()
		kernel.run()
		times[run] = os.clock() - start
	end
	table.sort(times)
	return times[(RUNS + 1) // 2]
end

print(string.format("_VERSION: %s", _VERSION))
print(string.format("%-28s %12s", "kernel", "median ms"))
for _, kernel in kernels do
	local median = measure(kernel)
	print(string.format("%-28s %12.2f", kernel.name, median * 1000))
end
