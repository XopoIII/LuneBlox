--!strict
local net = require("@lune/net")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local util = {}

function util.pass(method, url, message)
	local success, response = pcall(net.request, {
		method = method,
		url = url,
	})
	if not success then
		error(`{message} errored!\nError message: {tostring(response)}`)
	elseif not response.ok then
		error(
			`{message} failed, but should have passed!`
				.. `\nStatus code: {response.statusCode}`
				.. `\nStatus message: {response.statusMessage}`
				.. `\nResponse headers: {stdio.format(response.headers)}`
				.. `\nResponse body: {response.body}`
		)
	end
end

function util.fail(method, url, message)
	local success, response = pcall(net.request, {
		method = method,
		url = url,
	})
	if not success then
		error(`{message} errored!\nError message: {tostring(response)}`)
	elseif response.ok then
		error(
			`{message} passed, but should have failed!`
				.. `\nStatus code: {response.statusCode}`
				.. `\nStatus message: {response.statusMessage}`
				.. `\nResponse headers: {stdio.format(response.headers)}`
				.. `\nResponse body: {response.body}`
		)
	end
end

-- Statuses that say "not now" rather than "no": the server was reached and may answer the next
-- request. Anything else that is not ok - a page that moved, a request that is refused - is an answer,
-- and asking again does not change it.
local function answersLater(statusCode: number): boolean
	return statusCode == 408 or statusCode == 429 or statusCode >= 500
end

--[[
	Like `pass` with a GET, for a server this repository does not run.

	A request that errors, or is answered with a status from `answersLater`, is sent again after
	`backoff` seconds, then twice that, up to `attempts` requests in all. The last answer is judged as
	`pass` judges the only one: it has to be ok. A status that will not change fails at once.

	Returns the number of requests that were sent.
]]
function util.passEventually(
	url: string,
	message: string,
	attempts: number,
	backoff: number
): number
	for attempt = 1, attempts do
		local success, response = pcall(net.request, url)
		if success and response.ok then
			return attempt
		end
		local later = not success or answersLater(response.statusCode)
		if attempt == attempts or not later then
			if not success then
				error(
					`{message} errored on request {attempt} of {attempts}!`
						.. `\nError message: {tostring(response)}`
				)
			end
			error(
				`{message} failed on request {attempt} of {attempts}, but should have passed!`
					.. `\nStatus code: {response.statusCode}`
					.. `\nStatus message: {response.statusMessage}`
					.. `\nResponse headers: {stdio.format(response.headers)}`
					.. `\nResponse body: {response.body}`
			)
		end
		task.wait(backoff * 2 ^ (attempt - 1))
	end
	error("unreachable: the last request either returns or errors")
end

return util
