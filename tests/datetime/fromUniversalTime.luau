--!strict
local DateTime = require("@lune/datetime")

assert(
	DateTime.fromUniversalTime({
		year = 2023,
		month = 8,
		day = 26,
		hour = 16,
		minute = 56,
		second = 28,
		millisecond = 0,
	}).unixTimestamp == 1693068988,
	"expected DateTime.fromUniversalTime() with DateTimeValues arg to return 1693068988s"
)

assert(
	DateTime.fromUniversalTime({
		year = 2023,
		month = 8,
		day = 26,
		hour = 16,
		minute = 56,
		second = 28,
		millisecond = 892,
	}).unixTimestampMillis == 1693068988892,
	"expected DateTime.fromUniversalTime() with DateTimeValues arg with millis to return 1693068988892ms"
)

assert(
	DateTime.fromUniversalTime({
		year = 2023.9,
		month = 8.5,
		day = 26.5,
		hour = 16.5,
		minute = 56.5,
		second = 28.5,
		millisecond = 892.5,
	}).unixTimestampMillis == 1693068988892,
	"expected DateTime.fromUniversalTime() to truncate fractional DateTimeValues"
)
