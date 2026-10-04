#!/usr/bin/env sh
# Every Luau script under tests/ is registered in crates/lune/src/tests.rs, or named here as
# something that is not a test.
#
# WHY IT IS ENFORCED RATHER THAN AGREED: a test script that is not registered never runs, and
# nothing says so - the suite is green with it and green without it. When this check was written two
# regression tests of fixed issues had never run.
#
# A script that is not registered is listed below with its reason: a module other tests require,
# or a test that cannot run unattended.
#
# Usage: check-tests-registered.sh   (takes no arguments: the whole tests/ tree)
set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

REGISTRY=crates/lune/src/tests.rs

# The path of every `name: "path",` entry in the registry.
registered="$(sed -n 's/^[[:space:]]*[a-z0-9_]*: "\([^"]*\)",$/\1/p' "$REGISTRY")"

status=0
for file in $(git ls-files 'tests/*.luau'); do
	name="${file#tests/}"
	name="${name%.luau}"

	case "$name" in
		# Roblox place and model fixtures, a submodule that is not ours.
		roblox/rbx-test-files/*) continue ;;
		# Modules that tests require: helpers, shared data, and the targets of the require tests.
		fs/utils | net/request/util | task/fcheck | roblox/instance/query/tree) continue ;;
		serde/json/source | serde/jsonc/source | serde/toml/source) continue ;;
		require/modules/* | require/tests/modules/*) continue ;;
		require/tests/module | require/tests/multi.ext.file) continue ;;
		require/tests/state_module | require/tests/state_second) continue ;;
		# Prompts for input, which throws or blocks when stdin is not a terminal.
		stdio/prompt) continue ;;
		# Asks a third-party website for the user agent it saw; its outage would fail the suite.
		net/request/user_agent) continue ;;
		# Kills `cat` right after writing to it and then expects the echo: it races the child, and
		# fails most runs when the suite's load delays `cat`.
		process/create/kill) continue ;;
	esac

	if ! printf '%s\n' "$registered" | grep -qxF -- "$name"; then
		echo "check-tests-registered: $file is not registered in $REGISTRY" >&2
		status=1
	fi
done

# The other direction: an entry whose script was moved or deleted.
for name in $registered; do
	if [ ! -f "tests/$name.luau" ]; then
		echo "check-tests-registered: $REGISTRY names tests/$name.luau, which does not exist" >&2
		status=1
	fi
done

if [ "$status" -ne 0 ]; then
	echo "" >&2
	echo "Register a test in $REGISTRY, or list a script that is not one in this check." >&2
fi

exit "$status"
