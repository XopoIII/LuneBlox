--!strict
local fs = require("@lune/fs")
local glob = require("@tests/helpers/glob")
local roblox = require("@lune/roblox")

local modelBinaryPaths = glob.discover("tests/roblox/rbx-test-files/places/*/binary.rbxl")

for _, modelBinaryPath in modelBinaryPaths do
	local modelFileBinary = fs.readFile(modelBinaryPath)
	local modelXmlPath = string.gsub(modelBinaryPath, "binary%.rbxl$", "xml.rbxlx")
	local modelFileXml = fs.readFile(modelXmlPath)

	local modelBinary = roblox.deserializeModel(modelFileBinary)
	local modelXml = roblox.deserializeModel(modelFileXml)

	for _, modelInstance in modelBinary do
		assert(modelInstance:IsA("Instance"))
	end

	for _, modelInstance in modelXml do
		assert(modelInstance:IsA("Instance"))
	end
end
