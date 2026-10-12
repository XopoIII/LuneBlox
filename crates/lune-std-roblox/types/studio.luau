--!strict
-- The paths of the system's Roblox Studio install, which `init.luau` adds to the library
local studio = {}

--[=[
	@within Roblox
	@tag must_use

	Returns the path to the system's Roblox Studio executable.

	There is no guarantee that this will exist, but if Studio is installed this
	is where it will be.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	local pathToStudio = roblox.studioApplicationPath()
	print("Studio is located at:", pathToStudio)
	```
]=]
function studio.studioApplicationPath(): string
	return nil :: any
end

--[=[
	@within Roblox
	@tag must_use

	Returns the path to the `Content` folder of the system's current Studio
	install.

	This folder will always exist if Studio is installed.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	local pathToContent = roblox.studioContentPath()
	print("Studio's content folder is located at:", pathToContent)
	```
]=]
function studio.studioContentPath(): string
	return nil :: any
end

--[=[
	@within Roblox
	@tag must_use

	Returns the path to the `plugin` folder of the system's current Studio
	install. This is the path where local plugins are installed.

	This folder may not exist if the user has never installed a local plugin.
	It will also not necessarily take into account custom plugin directories
	set from Studio.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	local pathToPluginFolder = roblox.studioPluginPath()
	print("Studio's plugin folder is located at:", pathToPluginFolder)
	```
]=]
function studio.studioPluginPath(): string
	return nil :: any
end

--[=[
	@within Roblox
	@tag must_use

	Returns the path to the `BuiltInPlugin` folder of the system's current
	Studio install. This is the path where built-in plugins like the ToolBox
	are installed.

	This folder will always exist if Studio is installed.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	local pathToPluginFolder = roblox.studioBuiltinPluginPath()
	print("Studio's built-in plugin folder is located at:", pathToPluginFolder)
	```
]=]
function studio.studioBuiltinPluginPath(): string
	return nil :: any
end

return studio
