--!strict
--[[
	richterm
	terminal ansi formatter
]]

export type Formatter = (s: string, nocolor: boolean?) -> string

-- @noinline (if it ever exists)
local function format(s: string, open: string, close: string, replace: string): string
	local index = string.find(s, close, #open + 1, true)
	local middle = s

	if index then
		local closeLen = #close
		local middleTbl = {}
		local cursor = 1

		while index do
			table.insert(middleTbl, string.sub(s, cursor, index - 1))
			cursor = index + closeLen
			index = string.find(s, close, cursor, true)
		end

		middle = table.concat(middleTbl, replace)
	end

	return open .. middle .. close
end

local function formatter(open: string, close: string, replace: string?): Formatter
	replace = replace or open

	return function(s, nocolor)
		return if nocolor then s else format(s, open, close, replace :: string)
	end
end

local richterm = {
	reset = formatter("\27[0m", "\27[0m"),
	bold = formatter("\27[1m", "\27[22m", "\27[22m\27[1m"),
	dim = formatter("\27[2m", "\27[22m", "\27[22m\27[2m"),
	italic = formatter("\27[3m", "\27[23m"),
	underline = formatter("\27[4m", "\27[24m"),
	inverse = formatter("\27[7m", "\27[27m"),
	hidden = formatter("\27[8m", "\27[28m"),
	strikethrough = formatter("\27[9m", "\27[29m"),

	black = formatter("\27[30m", "\27[39m"),
	red = formatter("\27[31m", "\27[39m"),
	green = formatter("\27[32m", "\27[39m"),
	yellow = formatter("\27[33m", "\27[39m"),
	blue = formatter("\27[34m", "\27[39m"),
	magenta = formatter("\27[35m", "\27[39m"),
	cyan = formatter("\27[36m", "\27[39m"),
	white = formatter("\27[37m", "\27[39m"),
	gray = formatter("\27[90m", "\27[39m"),

	bgBlack = formatter("\27[40m", "\27[49m"),
	bgRed = formatter("\27[41m", "\27[49m"),
	bgGreen = formatter("\27[42m", "\27[49m"),
	bgYellow = formatter("\27[43m", "\27[49m"),
	bgBlue = formatter("\27[44m", "\27[49m"),
	bgMagenta = formatter("\27[45m", "\27[49m"),
	bgCyan = formatter("\27[46m", "\27[49m"),
	bgWhite = formatter("\27[47m", "\27[49m"),

	brightBlack = formatter("\27[90m", "\27[39m"),
	brightRed = formatter("\27[91m", "\27[39m"),
	brightGreen = formatter("\27[92m", "\27[39m"),
	brightYellow = formatter("\27[93m", "\27[39m"),
	brightBlue = formatter("\27[94m", "\27[39m"),
	brightMagenta = formatter("\27[95m", "\27[39m"),
	brightCyan = formatter("\27[96m", "\27[39m"),
	brightWhite = formatter("\27[97m", "\27[39m"),

	bgBrightBlack = formatter("\27[100m", "\27[49m"),
	bgBrightRed = formatter("\27[101m", "\27[49m"),
	bgBrightGreen = formatter("\27[102m", "\27[49m"),
	bgBrightYellow = formatter("\27[103m", "\27[49m"),
	bgBrightBlue = formatter("\27[104m", "\27[49m"),
	bgBrightMagenta = formatter("\27[105m", "\27[49m"),
	bgBrightCyan = formatter("\27[106m", "\27[49m"),
	bgBrightWhite = formatter("\27[107m", "\27[49m"),
}

function richterm.combine(...: Formatter): Formatter
	local formatters = { ... }

	return function(s, nocolor)
		if not nocolor then
			for _, applyFormatter in formatters do
				s = applyFormatter(s)
			end
		end

		return s
	end
end

return table.freeze(richterm)
