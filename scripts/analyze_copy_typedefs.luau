--!strict
local fs = require("@lune/fs")

fs.writeDir("./types")

-- Mirrors `luneblox setup`: a library's typedefs are either a single
-- `types.d.luau` file, or a `types/` directory module with an `init.luau`
for _, dir in fs.readDir("./crates") do
	local std = string.match(dir, "^lune%-std%-(%w+)$")
	if std ~= nil then
		local file = `./crates/{dir}/types.d.luau`
		local directory = `./crates/{dir}/types`
		if fs.isFile(file) then
			fs.copy(file, `./types/{std}.luau`, true)
		elseif fs.isDir(directory) then
			if fs.isFile(`./types/{std}.luau`) then
				fs.removeFile(`./types/{std}.luau`)
			end
			fs.copy(directory, `./types/{std}`, true)
		end
	end
end
