<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD041 -->

<div align="center">

# LuneBlox

**A standalone Luau runtime that runs Luau the way Roblox runs it.**

[![CI](https://img.shields.io/github/actions/workflow/status/XopoIII/LuneBlox/ci.yaml?branch=main&style=flat-square&label=CI)](https://github.com/XopoIII/LuneBlox/actions)
[![License](https://img.shields.io/github/license/XopoIII/LuneBlox?style=flat-square&color=informational)](LICENSE.txt)
[![Release](https://img.shields.io/github/v/release/XopoIII/LuneBlox?style=flat-square&color=informational)](https://github.com/XopoIII/LuneBlox/releases/latest)

[Documentation](https://xopoiii.github.io/LuneBlox/) ·
[Releases](https://github.com/XopoIII/LuneBlox/releases/latest) ·
[Changelog](CHANGELOG.md)

</div>

LuneBlox is [Lune](https://github.com/lune-org/lune) with the [Luau](https://luau-lang.org) underneath
changed, so that code tested or measured here behaves like the code players run.

```sh
luneblox run script.luau
luneblox setup              # typedefs for luau-lsp, aliased as @lune
luneblox build script.luau  # a standalone executable, carrying the same Luau and flags
```

## Why LuneBlox

Lune 0.10.5 runs Luau 0.709. Roblox runs 0.740 (September 2026), and on top of the version it runs
Luau with its own set of Luau's fast flags -- Luau ships new compiler and VM work behind flags that
default to off, and Roblox turns them on remotely. A benchmark or a test run on Lune measures a VM no
game runs.

LuneBlox closes both gaps:

| | Lune 0.10.5 | LuneBlox 0.10.12 |
|---|---|---|
| Luau | 0.709 | 0.740, the version Roblox runs |
| Luau's fast flags | Luau's defaults | set as the live Roblox client sets them (`crates/lune/src/rt/roblox_fflags.rs`) |
| Luau's C++ built with | `-Os` (the workspace's size profile reached it) | `-O3` |
| `_VERSION` | `Lune 0.10.5+709` | `LuneBlox 0.10.12+740` |

## Performance

Measured on an Apple M1, median of three runs, on one Luau networking library's generated modules:
interpreted code -- what most Roblox clients run -- is 39 to 57% faster to encode and decode, and
modules that Lune's Luau left interpreted even when natively compiled now compile. Native code built
with Roblox's flags includes `DebugCodegenOptSize`, which Roblox sets and which skips a native
optimisation, so native timings here are what Roblox gets rather than the best the VM can do.

`LUNE_ROBLOX_FFLAGS=0` runs with Luau's own defaults instead, to measure what the flags change.

## Where it comes from

LuneBlox is built from [Lune](https://github.com/lune-org/lune) and keeps its whole interface: the same
`@lune/*` libraries, the same task scheduler, the same Roblox datatypes and place/model files. The
documentation covers the command line, every library and what differs from Lune:
<https://xopoiii.github.io/LuneBlox/>.

## Install

With [Rokit](https://github.com/rojo-rbx/rokit):

```sh
rokit add XopoIII/LuneBlox luneblox
```

The command is `luneblox`, so it installs beside an upstream `lune` rather than in its place. Its
typedefs, cross-compilation cache and REPL history live in `~/.luneblox`.

## Contributing

When Roblox moves to a new Luau release:

1. Point `[patch.crates-io] mlua-sys` in `Cargo.toml` at an mlua commit that pins it, or drop the patch
   once mlua-sys on crates.io carries it.
2. `luneblox run scripts/generate_roblox_fflags.luau` to regenerate the flag table from Roblox's
   published client settings, limited to the flags that Luau declares.
3. `cargo test --workspace`.

## License

LuneBlox is licensed under the [Mozilla Public License 2.0](LICENSE.txt), as Lune is. It is a
modified version of [Lune](https://github.com/lune-org/lune) by Filip Tibell and Lune's contributors;
the changes are listed in [CHANGELOG.md](CHANGELOG.md).
