#!/usr/bin/env sh
# Keeps every recorded version in step.
#
# Three numbers are each written in more than one place, and nothing but this compares them:
#
#   the LuneBlox version   crates/lune/Cargo.toml is the source. README.md and the docs print it -
#                          in `_VERSION`, in the names of the release archives, in the typedefs
#                          directory - and CHANGELOG.md names it in its newest release.
#   the Luau version       vendor/luau0-src/Cargo.toml is the source (`+luau740`). README.md and the
#                          docs print it, and the fast flag table is generated against it.
#   the mlua version       every crate names its own, and they must be the same one.
#
# The docs also quote the fast flag table's header and its generator's output, which go stale each
# time the table is regenerated.
#
# `sh scripts/bump-version.sh <version>` rewrites the LuneBlox version everywhere this reads it.
# Prose that names a past release ("fixed since 0.10.7") matches none of the patterns below.
#
# Usage: check-versions.sh   (takes no arguments)
set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DOCS=docs/src/content/docs
FFLAGS=crates/lune/src/rt/roblox_fflags.rs

VERSION="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' crates/lune/Cargo.toml | head -1)"
LUAU="$(sed -n 's/^version = "[^"]*+luau\([0-9]*\)"$/\1/p' vendor/luau0-src/Cargo.toml | head -1)"

status=0

if [ -z "$VERSION" ]; then
	echo "check-versions: could not read a version out of crates/lune/Cargo.toml" >&2
	exit 1
fi
if [ -z "$LUAU" ]; then
	echo "check-versions: could not read a Luau version out of vendor/luau0-src/Cargo.toml" >&2
	exit 1
fi

# Reports every match of a pattern that does not carry the expected text.
#   expect <what> <pattern, as grep -E> <expected, as a fixed string> <file or directory> ...
expect() {
	what="$1"
	pattern="$2"
	expected="$3"
	shift 3
	stale="$(grep -rnoE -- "$pattern" "$@" | grep -vF -- "$expected" || true)"
	if [ -n "$stale" ]; then
		echo "check-versions: $what should read '$expected'" >&2
		printf '%s\n' "$stale" | sed 's/^/  /' >&2
		status=1
	fi
}

SEMVER='[0-9]+\.[0-9]+\.[0-9]+'

# The LuneBlox version
expect "_VERSION" "LuneBlox $SEMVER\+[0-9]+" "LuneBlox $VERSION+$LUAU" README.md "$DOCS"
expect "the comparison table's heading" "LuneBlox $SEMVER \|" "LuneBlox $VERSION |" README.md "$DOCS"
expect "a release archive's name" "luneblox-$SEMVER-" "luneblox-$VERSION-" README.md "$DOCS"
expect "the release archives' heading" "Archive for $SEMVER" "Archive for $VERSION" "$DOCS"
expect "the typedefs directory" "\.typedefs/$SEMVER/" ".typedefs/$VERSION/" README.md "$DOCS"
expect "the CLI's banner" "Lune v$SEMVER" "Lune v$VERSION" "$DOCS"

RELEASED="$(sed -n 's/^## `\([0-9][^`]*\)`.*/\1/p' CHANGELOG.md | head -1)"
if [ "$RELEASED" != "$VERSION" ]; then
	echo "check-versions: CHANGELOG.md's newest release is '$RELEASED', the version is '$VERSION'" >&2
	status=1
fi

# The Luau version
expect "the Luau version Roblox runs" "Roblox runs 0\.[0-9]+" "Roblox runs 0.$LUAU" README.md "$DOCS"
expect "the Luau version" "0\.[0-9]+, the version Roblox runs" "0.$LUAU, the version" README.md "$DOCS"
expect "the Luau the flags are limited to" "flags Luau 0\.[0-9]+ declares" "Luau 0.$LUAU declares" "$FFLAGS"

# The mlua version
MLUA="$(sed -n 's/^mlua = { version = "\([^"]*\)".*/\1/p' crates/*/Cargo.toml | sort -u)"
if [ "$(printf '%s\n' "$MLUA" | wc -l | tr -d ' ')" -ne 1 ]; then
	echo "check-versions: the crates name different mlua versions" >&2
	grep -n '^mlua = ' crates/*/Cargo.toml | sed 's/^/  /' >&2
	status=1
fi

# The fast flag table, as the docs quote it
PAGE="$DOCS/guides/fast-flags.mdx"
CLIENT="$(sed -n 's/^\/\/ Luau fast flags as the Roblox client \([0-9.]*\) sets them.*/\1/p' "$FFLAGS")"
TOTAL="$(grep -c '^    ("' "$FFLAGS" || true)"
ENABLED="$(grep -c ', true),$' "$FFLAGS" || true)"
for line in \
	"Luau fast flags as the Roblox client $CLIENT sets them, limited to the" \
	"flags Luau 0.$LUAU declares." \
	"It holds $TOTAL flags." \
	"Wrote $TOTAL flags ($ENABLED enabled) for Roblox $CLIENT and Luau 0.$LUAU"; do
	if ! grep -qF -- "$line" "$PAGE"; then
		echo "check-versions: $PAGE does not say '$line'" >&2
		echo "  It quotes $FFLAGS, which was regenerated." >&2
		status=1
	fi
done

if [ "$status" -ne 0 ]; then
	echo "" >&2
	echo "Run 'sh scripts/bump-version.sh <version>' for the LuneBlox version; fix the rest by hand." >&2
fi

exit "$status"
