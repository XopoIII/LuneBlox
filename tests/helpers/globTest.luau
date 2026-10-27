--!strict
local fs = require("@lune/fs")
local glob = require("@tests/helpers/glob")

assert(glob.match("src/*.luau", "src/main.luau"))
assert(not glob.match("src/*.luau", "src/nested/main.luau"))
assert(glob.match("src/**/*.luau", "src/main.luau"))
assert(glob.match("src/**/*.luau", "src/nested/deep/main.luau"))
assert(glob.match("file-[a-c]?.luau", "file-b1.luau"))
assert(glob.match("file-[!a-c].luau", "file-z.luau"))
assert(not glob.match("file-[!a-c].luau", "file-b.luau"))
assert(glob.match("literal-[*].luau", "literal-*.luau"))

local root = "bin/glob_test"
fs.writeDir(root .. "/a/nested")
fs.writeDir(root .. "/b")
fs.writeFile(root .. "/root.luau", "return 0")
fs.writeFile(root .. "/a/one.luau", "return 1")
fs.writeFile(root .. "/a/nested/two.luau", "return 2")
fs.writeFile(root .. "/a/nested/ignore.txt", "ignored")
fs.writeFile(root .. "/b/three.luau", "return 3")

local all = glob.discover(root .. "/**/*.luau")
assert(#all == 4, `Expected four Luau files, found {#all}`)
assert(all[1] == root .. "/a/nested/two.luau")
assert(all[2] == root .. "/a/one.luau")
assert(all[3] == root .. "/b/three.luau")
assert(all[4] == root .. "/root.luau")

local direct = glob.discover(root .. "/a/*.luau")
assert(#direct == 1 and direct[1] == root .. "/a/one.luau")
local literal = glob.discover(root .. "/a/nested/ignore.txt")
assert(#literal == 1 and literal[1] == root .. "/a/nested/ignore.txt")
assert(#glob.discover(root .. "/missing.luau") == 0)

fs.removeDir(root)
