--!strict
local difftext = require("@tests/helpers/difftext")
local fs = require("@lune/fs")
local process = require("@lune/process")
local serde = require("@lune/serde")

local MARKER = "LUNEBLOX_CONFORMANCE:"

local function readResults(path: string): any
	local contents = fs.readFile(path)
	for _, line in contents:split("\n") do
		local markerStart = line:find(MARKER, 1, true)
		if markerStart then
			local encoded = line:sub(markerStart + #MARKER):gsub("\r$", "")
			return serde.decode("json", encoded)
		end
	end
	error(`No {MARKER} result found in {path}`)
end

local function deepEqual(a: any, b: any): boolean
	if typeof(a) ~= typeof(b) then
		return false
	end
	if typeof(a) ~= "table" then
		return a == b
	end
	for key, value in a do
		if not deepEqual(value, b[key]) then
			return false
		end
	end
	for key in b do
		if a[key] == nil then
			return false
		end
	end
	return true
end

local args = process.args
assert(#args == 2, "Usage: luneblox run scripts/ocale/compare local.log cloud.log")
local localResults = readResults(args[1])
local cloudResults = readResults(args[2])

if not deepEqual(localResults, cloudResults) then
	local localJson = serde.encode("json", localResults, true)
	local cloudJson = serde.encode("json", cloudResults, true)
	local diff = difftext.prettydiff(localJson, cloudJson, {
		detailed = true,
		includeLineNumbers = true,
	})
	print(`OCALE results differ (- LuneBlox, + Roblox):\n{diff}`)
	process.exit(1)
end

print(`OCALE conformance passed: {#localResults} checks match Roblox`)
