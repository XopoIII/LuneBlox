--!strict
local luau = require("@lune/luau")

-- LuneBlox: regression for https://github.com/lune-org/lune/issues/425.
-- Keep the copies and the return in the same function: forwarding eleven table fields through
-- mutable upvalues stresses lazy restore locations around conditional native barrier spills.
-- Check typed/untyped functions, open/closed upvalues, and register-pressure boundary sizes.
for _, count in { 10, 11, 13, 32 } do
	for _, typed in { false, true } do
		for _, optimizationLevel in { 1, 2 } do
			local names: { string } = {}
			local zeros = {}
			for index = 0, count - 1 do
				table.insert(names, `u{index}`)
				table.insert(zeros, "0")
			end
			local lines = {
				"return function(check)",
				`local {table.concat(names, ", ")} = {table.concat(zeros, ", ")}`,
				if typed
					then "local function f(t: { [string]: number }): ...number"
					else "local function f(t)",
			}
			for index = 0, count - 1 do
				table.insert(lines, `u{index} = t.f{index}`)
			end
			table.insert(lines, `return {table.concat(names, ", ")}`)
			table.insert(lines, "end")
			table.insert(lines, "check(f)") -- Upvalues still point into the running parent stack.
			table.insert(lines, "return f")
			table.insert(lines, "end")
			local bytecode = luau.compile(
				table.concat(lines, "\n"),
				{ optimizationLevel = optimizationLevel, debugLevel = 1 }
			)

			for _, native in { false, true } do
				local factory =
					luau.load(bytecode, { debugName = "UpvalueCodegen", codegenEnabled = native })()
				local function check(f: any)
					for call = 1, 8 do
						local fields: { [string]: number } = {}
						for index = 0, count - 1 do
							fields[`f{index}`] = (index + 1) * 100 + call - 1
						end
						local values = table.pack(f(fields))
						assert(values.n == count, `expected {count} return values, got {values.n}`)
						for index = 0, count - 1 do
							assert(
								values[index + 1] == fields[`f{index}`],
								`native={native}, typed={typed}, optimizationLevel={optimizationLevel}, count={count}, call={call}: wrong u{index}: {values[index + 1]}`
							)
						end
					end
				end
				local closed = factory(check)
				check(closed) -- The parent has returned and the upvalues are now heap-backed.
			end
		end
	end
end
