# Lute test helpers

The modules in `difftext/`, `collections/deque.luau`, `glob.luau`, and `richterm.luau`
were adapted from [Lute](https://github.com/luau-lang/lute) at commit
`337662f51eb25666f56d6ae5911a0dc6ed570bdc`.

The original MIT license is preserved in `LUTE-LICENSE`. LuneBlox adaptations add
strict type-checking directives, replace Lute standard-library dependencies with
`@lune/fs` and local modules, and make glob discovery return normalized string
paths. The Myers diff implementation no longer depends on `tableext.reverse`.
