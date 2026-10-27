--!strict
--[[
    Glob pattern matching and filesystem discovery.

    `glob.match(pattern, input)` tests whether a path matches a glob pattern.
    `glob.discover(pattern)` walks the filesystem returning all matching string paths.

    The implementation follows the Rust glob crate. The rules are:

    - `?` matches any single character (not `/`)
    - `*` matches any (possibly empty) sequence of characters (not `/`)
    - `**` matches the current directory and arbitrary subdirectories; it must
      form an entire path component on its own (e.g. `a/**/b` is valid, but
      `a**b` is not)
    - `[...]` matches any character inside the brackets; ranges like `[a-z]`
      are supported, ordered by Unicode code point
    - `[!...]` matches any character NOT inside the brackets
    - Metacharacters can be matched literally by wrapping them in brackets,
      e.g. `[?]` matches a literal `?`, `[*]` matches a literal `*`
]]
local fs = require("@lune/fs")

local function matchesSegment(
	pattern: string,
	name: string,
	patternStart: number?,
	nameStart: number?
): boolean
	local patternIndex = patternStart or 1
	local nameIndex = nameStart or 1
	while patternIndex <= #pattern do
		local patternChar = string.sub(pattern, patternIndex, patternIndex)
		if patternChar == "*" then
			patternIndex += 1
			for candidateIndex = nameIndex, #name + 1 do
				if matchesSegment(pattern, name, patternIndex, candidateIndex) then
					return true
				end
			end
			return false
		elseif patternChar == "?" then
			if nameIndex > #name then
				return false
			end
			patternIndex += 1
			nameIndex += 1
		elseif patternChar == "[" then
			if nameIndex > #name then
				return false
			end
			local nameChar = string.sub(name, nameIndex, nameIndex)
			patternIndex += 1
			local negate = string.sub(pattern, patternIndex, patternIndex) == "!"
			if negate then
				patternIndex += 1
			end
			local found = false
			while
				patternIndex <= #pattern
				and string.sub(pattern, patternIndex, patternIndex) ~= "]"
			do
				local rangeLow = string.sub(pattern, patternIndex, patternIndex)
				patternIndex += 1
				if
					string.sub(pattern, patternIndex, patternIndex) == "-"
					and patternIndex + 1 <= #pattern
					and string.sub(pattern, patternIndex + 1, patternIndex + 1) ~= "]"
				then
					patternIndex += 1
					local rangeHigh = string.sub(pattern, patternIndex, patternIndex)
					patternIndex += 1
					if nameChar >= rangeLow and nameChar <= rangeHigh then
						found = true
					end
				else
					if nameChar == rangeLow then
						found = true
					end
				end
			end
			patternIndex += 1
			if negate then
				found = not found
			end
			if not found then
				return false
			end
			nameIndex += 1
		else
			if nameIndex > #name or string.sub(name, nameIndex, nameIndex) ~= patternChar then
				return false
			end
			patternIndex += 1
			nameIndex += 1
		end
	end
	return nameIndex > #name
end

type MatchResult = "match" | "no_match" | "partial"

-- Matches input path segments against pattern segments, returning:
--   "match"    -- full match (all pattern and input segments consumed)
--   "partial"  -- input consumed but pattern segments remain (worth descending)
--   "no_match" -- input diverges from pattern (prune)
local function matchStep(
	patternSegments: { string },
	inputSegments: { string },
	patternIndex: number,
	inputIndex: number
): MatchResult
	while patternIndex <= #patternSegments do
		local segment = patternSegments[patternIndex]
		if segment == "**" then
			local best: MatchResult = "no_match"
			for candidateIndex = inputIndex, #inputSegments + 1 do
				local result =
					matchStep(patternSegments, inputSegments, patternIndex + 1, candidateIndex)
				if result == "match" then
					return "match"
				elseif result == "partial" then
					best = "partial"
				end
			end
			return best
		else
			if inputIndex > #inputSegments then
				return "partial"
			end
			if not matchesSegment(segment, inputSegments[inputIndex]) then
				return "no_match"
			end
			patternIndex += 1
			inputIndex += 1
		end
	end
	if inputIndex > #inputSegments then
		return "match"
	end
	return "no_match"
end
local function normalize(path: string): string
	return (string.gsub(path, "\\", "/"))
end

local function split(path: string): (string, { string })
	local normalized = normalize(path)
	local prefix = ""
	if string.sub(normalized, 1, 1) == "/" then
		prefix = "/"
		normalized = string.sub(normalized, 2)
	elseif string.match(normalized, "^%a:/") then
		prefix = string.sub(normalized, 1, 3)
		normalized = string.sub(normalized, 4)
	end
	local parts = {}
	for part in string.gmatch(normalized, "[^/]+") do
		table.insert(parts, part)
	end
	return prefix, parts
end

local function join(directory: string, name: string): string
	if directory == "" or directory == "." then
		return name
	elseif string.sub(directory, -1) == "/" then
		return directory .. name
	end
	return directory .. "/" .. name
end

local function walk(
	directory: string,
	segments: { string },
	segmentIndex: number,
	results: { string }
)
	if segmentIndex > #segments then
		return
	end
	local success, entries = pcall(fs.readDir, directory)
	if not success then
		return
	end
	local segment = segments[segmentIndex]
	local isLast = segmentIndex == #segments
	if segment == "**" then
		if not isLast then
			walk(directory, segments, segmentIndex + 1, results)
		end
		for _, entryName in entries do
			local entryPath = join(directory, entryName)
			if isLast then
				table.insert(results, entryPath)
			end
			if fs.isDir(entryPath) then
				walk(entryPath, segments, segmentIndex, results)
			end
		end
	else
		for _, entryName in entries do
			if matchesSegment(segment, entryName) then
				local entryPath = join(directory, entryName)
				if isLast then
					table.insert(results, entryPath)
				elseif fs.isDir(entryPath) then
					walk(entryPath, segments, segmentIndex + 1, results)
				end
			end
		end
	end
end

local glob = {}

function glob.match(pattern: string, input: string): boolean
	local _, patternParts = split(pattern)
	local _, inputParts = split(input)
	return matchStep(patternParts, inputParts, 1, 1) == "match"
end

function glob.discover(input: string): { string }
	local normalized = normalize(input)
	local prefix, parts = split(normalized)
	local firstGlobIndex = #parts + 1
	for index, part in parts do
		if string.find(part, "[%*%?%[]") then
			firstGlobIndex = index
			break
		end
	end
	if firstGlobIndex > #parts then
		if fs.metadata(normalized).exists then
			return { normalized }
		end
		return {}
	end
	local fixedParts = { table.unpack(parts, 1, firstGlobIndex - 1) }
	local directory = prefix .. table.concat(fixedParts, "/")
	if directory == "" then
		directory = "."
	end
	local segments = { table.unpack(parts, firstGlobIndex) }
	local results: { string } = {}
	walk(directory, segments, 1, results)
	table.sort(results)
	return results
end

return glob
