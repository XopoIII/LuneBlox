#!/usr/bin/env bash

set -euo pipefail

# Keep in sync with format-check.sh: `--glob` replaces stylua's default file set
stylua .lune crates scripts tests \
	--glob "**/*.luau" \
	--glob "!tests/roblox/rbx-test-files/**"

cargo fmt
