--!strict
type InstanceProperties = {
	Parent: Instance?,
	ClassName: string,
	Name: string,
	-- FIXME: This breaks intellisense, but we need some way to access
	-- instance properties without casting the entire instance to any...
	-- [string]: any,
}

type InstanceMetatable = {
	Clone: (self: Instance) -> Instance,
	Destroy: (self: Instance) -> (),
	ClearAllChildren: (self: Instance) -> (),

	GetChildren: (self: Instance) -> { Instance },
	GetDebugId: (self: Instance) -> string,
	GetDescendants: (self: Instance) -> { Instance },
	GetFullName: (self: Instance) -> string,
	QueryDescendants: (self: Instance, selector: string) -> { Instance },

	FindFirstAncestor: (self: Instance, name: string) -> Instance?,
	FindFirstAncestorOfClass: (self: Instance, className: string) -> Instance?,
	FindFirstAncestorWhichIsA: (self: Instance, className: string) -> Instance?,
	FindFirstChild: (self: Instance, name: string, recursive: boolean?) -> Instance?,
	FindFirstChildOfClass: (self: Instance, className: string, recursive: boolean?) -> Instance?,
	FindFirstChildWhichIsA: (self: Instance, className: string, recursive: boolean?) -> Instance?,

	IsA: (self: Instance, className: string) -> boolean,
	IsAncestorOf: (self: Instance, descendant: Instance) -> boolean,
	IsDescendantOf: (self: Instance, ancestor: Instance) -> boolean,

	GetAttribute: (self: Instance, name: string) -> any,
	GetAttributes: (self: Instance) -> { [string]: any },
	SetAttribute: (self: Instance, name: string, value: any) -> (),

	GetTags: (self: Instance) -> { string },
	HasTag: (self: Instance, name: string) -> boolean,
	AddTag: (self: Instance, name: string) -> (),
	RemoveTag: (self: Instance, name: string) -> (),
}

export type Instance = typeof(setmetatable(
	(nil :: any) :: InstanceProperties,
	(nil :: any) :: { __index: InstanceMetatable }
))

export type DataModelProperties = {}
export type DataModelMetatable = {
	GetService: (self: DataModel, name: string) -> Instance,
	FindService: (self: DataModel, name: string) -> Instance?,
}

export type DataModel =
	Instance
	& typeof(setmetatable(
		(nil :: any) :: DataModelProperties,
		(nil :: any) :: { __index: DataModelMetatable }
	))

return {}
