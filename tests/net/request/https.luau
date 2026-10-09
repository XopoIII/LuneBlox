--!strict
local task = require("@lune/task")

local util = require("./util")
local passEventually = util.passEventually

-- These are some public APIs that have, or most likely have, different
-- certificate authorities (CAs), plus are both free to use and stable.
-- This should be enough to ensure that rustls is configured correctly.
local servers = {
	"https://www.googleapis.com/discovery/v1/apis",
	"https://api.cloudflare.com/client/v4/ips",
	-- Microsoft's own CA. The sign-in metadata, not a page of azure.microsoft.com: that one took
	-- seven seconds on a good day and answered 503 on a bad one.
	"https://login.microsoftonline.com/common/v2.0/.well-known/openid-configuration",
	"https://acme-v02.api.letsencrypt.org/directory",
	"https://ip-ranges.amazonaws.com/ip-ranges.json",
	"https://en.wikipedia.org/w/api.php",
	"https://status.godaddy.com/api/v2/summary.json",
}

-- These servers are not ours, and any of them is unavailable now and then: one 503 turned a
-- build red. Such an answer is asked for again, four requests with fourteen seconds of waiting
-- between them; every server still has to answer ok in the end, and one that moved the page or
-- refuses the request fails on the first answer (tests/net/request/retry pins both).
local ATTEMPTS = 4
local BACKOFF = 2

for _, server in servers do
	task.spawn(function()
		passEventually(server, server, ATTEMPTS, BACKOFF)
	end)
end
