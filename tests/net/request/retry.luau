--!strict
local net = require("@lune/net")
local task = require("@lune/task")

local util = require("./util")
local passEventually = util.passEventually

-- What `passEventually` promises to the tests that ask servers we do not run: it asks again only
-- after an answer that may change, and it never turns a server that stays down into a pass.

local PORT = 8866
local URL = `http://127.0.0.1:{PORT}`

-- Each path answers from its own list of statuses, one per request; the last one repeats
local scripted: { [string]: { number } } = {
	["/ok"] = { 200 },
	["/recovers"] = { 503, 429, 200 },
	["/down"] = { 503 },
	["/moved"] = { 404, 200 },
	["/refused"] = { 403, 200 },
	["/slow"] = { 408, 500, 502, 504, 200 },
}
local served: { [string]: number } = {}

local handle = net.serve(PORT, function(request)
	local statuses = scripted[request.path]
	local count = (served[request.path] or 0) + 1
	served[request.path] = count
	local status = statuses[math.min(count, #statuses)]
	return { status = status, body = `answer {count}` }
end)

local function failsWith(path: string, attempts: number, expected: string)
	local success, message = pcall(passEventually, URL .. path, path, attempts, 0)
	assert(not success, `{path} passed, but the server never answered ok`)
	assert(
		string.find(tostring(message), expected, 1, true) ~= nil,
		`{path} failed with the wrong message: {tostring(message)}`
	)
end

-- A failed check must not leave the server running: the process would wait for it and the
-- test would hang where it should fail
local passed, failure = pcall(function()
	-- An ok answer is taken at once
	assert(passEventually(URL .. "/ok", "/ok", 4, 0) == 1)
	assert(served["/ok"] == 1)

	-- An answer that may change is asked for again until it does
	assert(passEventually(URL .. "/recovers", "/recovers", 4, 0) == 3)
	assert(served["/recovers"] == 3)
	assert(passEventually(URL .. "/slow", "/slow", 5, 0) == 5)
	assert(served["/slow"] == 5)

	-- A server that stays down fails, after exactly the requests it was given, with its last answer
	failsWith("/down", 4, "failed on request 4 of 4")
	assert(served["/down"] == 4)
	failsWith("/down", 1, "Status code: 503")
	assert(served["/down"] == 5)

	-- One request short of the ok answer is still a failure
	served["/recovers"] = 0
	failsWith("/recovers", 2, "Status code: 429")
	assert(served["/recovers"] == 2)

	-- An answer that will not change fails on the first request, though the next would have been ok
	failsWith("/moved", 4, "failed on request 1 of 4")
	assert(served["/moved"] == 1)
	failsWith("/refused", 4, "Status code: 403")
	assert(served["/refused"] == 1)

	-- The wait doubles: 0.1 s and 0.2 s before the third request
	served["/recovers"] = 0
	local started = os.clock()
	assert(passEventually(URL .. "/recovers", "/recovers", 4, 0.1) == 3)
	local waited = os.clock() - started
	assert(waited >= 0.3, `waited {waited} s, expected at least 0.3`)
	assert(waited < 2, `waited {waited} s, expected about 0.3`)
end)

handle.stop()
task.wait()
if not passed then
	error(failure, 0)
end

-- A request that cannot be sent at all is retried the same way, and reported as an error
local refused, message = pcall(passEventually, URL .. "/ok", "stopped server", 2, 0)
assert(not refused, "a stopped server passed")
assert(
	string.find(tostring(message), "errored on request 2 of 2", 1, true) ~= nil,
	`a stopped server failed with the wrong message: {tostring(message)}`
)
