#!/usr/bin/env bash

set -euo pipefail

# `--glob` replaces stylua's default file set, so it must name every Luau file,
# not only the tests: it used to be `tests/**/*.luau`, which left `.lune`,
# `crates` and `scripts` unchecked
stylua .lune crates scripts tests \
	--glob "**/*.luau" \
	--glob "!tests/roblox/rbx-test-files/**" \
	--check

cargo fmt --check
