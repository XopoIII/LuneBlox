--!strict
local database = require("@self/database")
local instance = require("@self/instance")

export type DatabaseScriptability = database.DatabaseScriptability
export type DatabasePropertyTag = database.DatabasePropertyTag
export type DatabaseClassTag = database.DatabaseClassTag
export type DatabaseProperty = database.DatabaseProperty
export type DatabaseClass = database.DatabaseClass
export type DatabaseEnum = database.DatabaseEnum
export type Database = database.Database
export type Instance = instance.Instance
export type DataModelProperties = instance.DataModelProperties
export type DataModelMetatable = instance.DataModelMetatable
export type DataModel = instance.DataModel

--[=[
	@class Roblox

	Built-in library for manipulating Roblox place & model files

	### Example usage

	```lua
	local fs = require("@lune/fs")
	local roblox = require("@lune/roblox")

	-- Reading a place file
	local placeFile = fs.readFile("myPlaceFile.rbxl")
	local game = roblox.deserializePlace(placeFile)

	-- Manipulating and reading instances - just like in Roblox!
	local workspace = game:GetService("Workspace")
	for _, child in workspace:GetChildren() do
		print("Found child " .. child.Name .. " of class " .. child.ClassName)
	end

	-- Writing a place file
	local newPlaceFile = roblox.serializePlace(game)
	fs.writeFile("myPlaceFile.rbxl", newPlaceFile)
	```
]=]
local roblox = {}

--[=[
	@within Roblox
	@tag must_use

	Deserializes a place into a DataModel instance.

	This function accepts a string of contents, *not* a file path.
	If reading a place file from a file path is desired, `fs.readFile`
	can be used and the resulting string may be passed to this function.

	### Example usage

	```lua
	local fs = require("@lune/fs")
	local roblox = require("@lune/roblox")

	local placeFile = fs.readFile("filePath.rbxl")
	local game = roblox.deserializePlace(placeFile)
	```

	@param contents The contents of the place to read
]=]
function roblox.deserializePlace(contents: string): DataModel
	return nil :: any
end

--[=[
	@within Roblox
	@tag must_use

	Deserializes a model into an array of instances.

	This function accepts a string of contents, *not* a file path.
	If reading a model file from a file path is desired, `fs.readFile`
	can be used and the resulting string may be passed to this function.

	### Example usage

	```lua
	local fs = require("@lune/fs")
	local roblox = require("@lune/roblox")

	local modelFile = fs.readFile("filePath.rbxm")
	local instances = roblox.deserializeModel(modelFile)
	```

	@param contents The contents of the model to read
]=]
function roblox.deserializeModel(contents: string): { Instance }
	return nil :: any
end

--[=[
	@within Roblox
	@tag must_use

	Serializes a place from a DataModel instance.

	This string can then be written to a file, or sent over the network.

	### Example usage

	```lua
	local fs = require("@lune/fs")
	local roblox = require("@lune/roblox")

	local placeFile = roblox.serializePlace(game)
	fs.writeFile("filePath.rbxl", placeFile)
	```

	@param dataModel The DataModel for the place to serialize
	@param xml If the place should be serialized as xml or not. Defaults to `false`, meaning the place gets serialized using the binary format and not xml.
]=]
function roblox.serializePlace(dataModel: DataModel, xml: boolean?): string
	return nil :: any
end

--[=[
	@within Roblox
	@tag must_use

	Serializes one or more instances as a model.

	This string can then be written to a file, or sent over the network.

	### Example usage

	```lua
	local fs = require("@lune/fs")
	local roblox = require("@lune/roblox")

	local modelFile = roblox.serializeModel({ instance1, instance2, ... })
	fs.writeFile("filePath.rbxm", modelFile)
	```

	@param instances The array of instances to serialize
	@param xml If the model should be serialized as xml or not. Defaults to `false`, meaning the model gets serialized using the binary format and not xml.
]=]
function roblox.serializeModel(instances: { Instance }, xml: boolean?): string
	return nil :: any
end

--[=[
	@within Roblox
	@tag must_use

	Gets the current auth cookie, for usage with Roblox web APIs.

	Note that this auth cookie is formatted for use as a "Cookie" header,
	and that it contains restrictions so that it may only be used for
	official Roblox endpoints. To get the raw cookie value without any
	additional formatting, you can pass `true` as the first and only parameter.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")
	local serde = require("@lune/serde")
	local net = require("@lune/net")

	local cookie = roblox.getAuthCookie()
	assert(cookie ~= nil, "Failed to get roblox auth cookie")

	local myPrivatePlaceId = 1234567890

	local response = net.request({
		url = "https://assetdelivery.roblox.com/v2/assetId/" .. tostring(myPrivatePlaceId),
		headers = {
			Cookie = cookie,
		},
	})

	local responseTable = serde.decode("json", response.body)
	local responseLocation = responseTable.locations[1].location
	print("Download link to place: " .. responseLocation)
	```

	@param raw If the cookie should be returned as a pure value or not. Defaults to false
]=]
function roblox.getAuthCookie(raw: boolean?): string?
	return nil :: any
end

--[=[
	@within Roblox
	@tag must_use

	Gets the bundled reflection database.

	This database contains information about Roblox enums, classes, and their properties.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	local db = roblox.getReflectionDatabase()

	print("There are", #db:GetClassNames(), "classes in the reflection database")

	print("All base instance properties:")

	local class = db:GetClass("Instance")
	for name, prop in class.Properties do
		print(string.format(
			"- %s with datatype %s and default value %s",
			prop.Name,
			prop.Datatype,
			tostring(class.DefaultProperties[prop.Name])
		))
	end
	```
]=]
function roblox.getReflectionDatabase(): Database
	return nil :: any
end

--[=[
	@within Roblox

	Implements a property for all instances of the given `className`.

	This takes into account class hierarchies, so implementing a property
	for the `BasePart` class will also implement it for `Part` and others,
	unless a more specific implementation is added to the `Part` class directly.

	### Behavior

	The given `getter` callback will be called each time the property is
	indexed, with the instance as its one and only argument. The `setter`
	callback, if given, will be called each time the property should be set,
	with the instance as the first argument and the property value as second.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	local part = roblox.Instance.new("Part")

	local propertyValues = {}
	roblox.implementProperty(
		"BasePart",
		"CoolProp",
		function(instance)
			if propertyValues[instance] == nil then
				propertyValues[instance] = 0
			end
			propertyValues[instance] += 1
			return propertyValues[instance]
		end,
		function(instance, value)
			propertyValues[instance] = value
		end
	)

	print(part.CoolProp) --> 1
	print(part.CoolProp) --> 2
	print(part.CoolProp) --> 3

	part.CoolProp = 10

	print(part.CoolProp) --> 11
	print(part.CoolProp) --> 12
	print(part.CoolProp) --> 13
	```

	@param className The class to implement the property for.
	@param propertyName The name of the property to implement.
	@param getter The function which will be called to get the property value when indexed.
	@param setter The function which will be called to set the property value when indexed. Defaults to a function that will error with a message saying the property is read-only.
]=]
function roblox.implementProperty<T>(
	className: string,
	propertyName: string,
	getter: (instance: Instance) -> T,
	setter: ((instance: Instance, value: T) -> ())?
)
	return nil :: any
end

--[=[
	@within Roblox

	Implements a method for all instances of the given `className`.

	This takes into account class hierarchies, so implementing a method
	for the `BasePart` class will also implement it for `Part` and others,
	unless a more specific implementation is added to the `Part` class directly.

	### Behavior

	The given `callback` will be called every time the method is called,
	and will receive the instance it was called on as its first argument.
	The remaining arguments will be what the caller passed to the method, and
	all values returned from the callback will then be returned to the caller.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	local part = roblox.Instance.new("Part")

	roblox.implementMethod("BasePart", "TestMethod", function(instance, ...)
	    print("Called TestMethod on instance", instance, "with", ...)
	end)

	part:TestMethod("Hello", "world!")
	--> Called TestMethod on instance Part with Hello, world!
	```

	@param className The class to implement the method for.
	@param methodName The name of the method to implement.
	@param callback The function which will be called when the method is called.
]=]
function roblox.implementMethod(
	className: string,
	methodName: string,
	callback: (instance: Instance, ...any) -> ...any
)
	return nil :: any
end

--[=[
	@within Roblox

	Registers a new class that the reflection database does not know about.

	This is useful for classes that only exist in specific Roblox builds, and
	are therefore not part of the reflection database that Lune ships with.

	Once a class has been registered it can be created using `Instance.new`,
	and have methods and properties implemented for it using `implementMethod`
	and `implementProperty`, just like any built-in class.

	The given `superClassName` must be a valid class name - either a built-in
	class, or another class registered beforehand - and defaults to `"Instance"`
	when not provided. Registering a class that already exists will error.

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	roblox.registerClass("MyClass", "Instance")

	roblox.implementMethod("MyClass", "DoSomething", function(instance, ...)
		print("Called DoSomething on", instance, "with", ...)
	end)

	local instance = roblox.Instance.new("MyClass")
	instance:DoSomething("Hello", "world!")
	```

	@param className The name of the class to register.
	@param superClassName The name of the superclass to inherit from. Defaults to `"Instance"`.
]=]
function roblox.registerClass(className: string, superClassName: string?)
	return nil :: any
end

--[=[
	@within Roblox

	Registers a new service that the reflection database does not know about.

	This behaves just like `registerClass`, but additionally marks the class as
	a service so that it can be retrieved using `DataModel:GetService`. This is
	useful for services that only exist inside specific Roblox builds, such as
	`FileSystemService` in roblox-cli, and are therefore not part of the
	reflection database that Lune ships with.

	### Example usage

	```lua
	local fs = require("@lune/fs")
	local roblox = require("@lune/roblox")

	roblox.registerService("FileSystemService")

	roblox.implementMethod("FileSystemService", "ReadFile", function(_, path: string)
		return fs.readFile(path)
	end)

	local game = roblox.Instance.new("DataModel")
	local fs = game:GetService("FileSystemService")
	print(fs:ReadFile("foo.txt")) --> contents of foo.txt
	```

	@param serviceName The name of the service to register.
]=]
function roblox.registerService(serviceName: string)
	return nil :: any
end

-- TODO: Make typedefs for all of the datatypes as well...
roblox.Instance = (nil :: any) :: {
	new: ((className: "DataModel") -> DataModel) & ((className: string) -> Instance),
}

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
function roblox.studioApplicationPath(): string
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
function roblox.studioContentPath(): string
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
function roblox.studioPluginPath(): string
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
function roblox.studioBuiltinPluginPath(): string
	return nil :: any
end

return roblox
