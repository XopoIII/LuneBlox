--!strict
--[[
	NOTE: We export a couple different DateTimeValues types below to ensure
	that types are completely accurate, for method args milliseconds will
	always be optional, but for return values millis are always included

	If we figure out some better strategy here where we can
	export just a single type while maintaining accuracy we
	can change to that in a future breaking semver release
]]

type OptionalMillisecond = {
	millisecond: number?,
}

type Millisecond = {
	millisecond: number,
}

--[=[
	@interface Locale
	@within DateTime

	Enum type representing supported DateTime locales.

	Currently supported locales are:

	- `en` - English
	- `de` - German
	- `es` - Spanish
	- `fr` - French
	- `it` - Italian
	- `ja` - Japanese
	- `pl` - Polish
	- `pt-br` - Brazilian Portuguese
	- `pt` - Portuguese
	- `tr` - Turkish
]=]
export type Locale = "en" | "de" | "es" | "fr" | "it" | "ja" | "pl" | "pt-br" | "pt" | "tr"

--[=[
	@interface DateTimeValues
	@within DateTime

	Individual date & time values, representing the primitives that make up a `DateTime`.

	This is a dictionary that will contain the following values:

	- `year` - Year(s), in the range 1400 -> 9999
	- `month` - Month(s), in the range 1 -> 12
	- `day` - Day(s), in the range 1 -> 31
	- `hour` - Hour(s), in the range 0 -> 23
	- `minute` - Minute(s), in the range 0 -> 59
	- `second` - Second(s), in the range 0 -> 60, where 60 is a leap second

	An additional `millisecond` value may also be included,
	and should be within the range `0 -> 999`, but is optional.

	However, any method returning this type should be guaranteed
	to include milliseconds - see individual methods to verify.
]=]
export type DateTimeValues = {
	year: number,
	month: number,
	day: number,
	hour: number,
	minute: number,
	second: number,
}

--[=[
	@interface DateTimeValueArguments
	@within DateTime

	Alias for `DateTimeValues` with an optional `millisecond` value.

	Refer to the `DateTimeValues` documentation for additional information.
]=]
export type DateTimeValueArguments = DateTimeValues & OptionalMillisecond

--[=[
	@interface DateTimeValueReturns
	@within DateTime

	Alias for `DateTimeValues` with a mandatory `millisecond` value.

	Refer to the `DateTimeValues` documentation for additional information.
]=]
export type DateTimeValueReturns = DateTimeValues & Millisecond

return {}
