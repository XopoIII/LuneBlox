#!/usr/bin/env sh
# Keeps source files at a size a person can hold in their head.
#
# The limit is 500 lines for Rust and Luau files, with no exceptions. A file that reaches it is
# split into modules; it is not listed here.
#
# Usage: check-file-size.sh [<file> ...]   (no arguments: every tracked .rs and .luau file)
set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

LIMIT=500

if [ "$#" -eq 0 ]; then
	set -- $(git ls-files '*.rs' '*.luau')
fi

status=0
for file in "$@"; do
	[ -f "$file" ] || continue
	case "$file" in
		*.rs | *.luau) ;;
		*) continue ;;
	esac

	case "$file" in
		# Roblox place and model fixtures, a submodule that is not ours.
		tests/roblox/rbx-test-files/*) continue ;;
	esac

	lines=$(wc -l < "$file" | tr -d ' ')
	if [ "$lines" -gt "$LIMIT" ]; then
		echo "check-file-size: $file is $lines lines, over the $LIMIT line limit" >&2
		echo "  Split it into modules." >&2
		status=1
	fi
done

exit "$status"
