#!/usr/bin/env sh
# Writes a new LuneBlox version everywhere it is recorded.
#
# The version lives in crates/lune/Cargo.toml and is printed by README.md and the docs in several
# spellings; scripts/check-versions.sh fails when one of them is left behind. This rewrites exactly
# the spellings that check reads, so the two cannot disagree about what a version looks like.
#
# CHANGELOG.md is left alone: its entry is written by hand, and the check reports it until the
# `## Unreleased` heading becomes the new version's.
#
# Usage: bump-version.sh <version>   (for example: bump-version.sh 0.10.13)
set -e

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [ "$#" -ne 1 ] || ! printf '%s' "$1" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
	echo "bump-version: expected one version, such as 0.10.13" >&2
	exit 2
fi

OLD="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' crates/lune/Cargo.toml | head -1)"
NEW="$1"
export OLD NEW

if [ -z "$OLD" ]; then
	echo "bump-version: could not read a version out of crates/lune/Cargo.toml" >&2
	exit 1
fi

# The package version is the first `version = ` line of the manifest; dependencies come later.
perl -0pi -e 's/^version = "\Q$ENV{OLD}\E"$/version = "$ENV{NEW}"/m' crates/lune/Cargo.toml

find README.md docs/src/content/docs -type f \( -name '*.md' -o -name '*.mdx' \) -exec perl -pi -e '
	s/LuneBlox \Q$ENV{OLD}\E\+/LuneBlox $ENV{NEW}+/g;
	s/LuneBlox \Q$ENV{OLD}\E \|/LuneBlox $ENV{NEW} |/g;
	s/luneblox-\Q$ENV{OLD}\E-/luneblox-$ENV{NEW}-/g;
	s/Archive for \Q$ENV{OLD}\E/Archive for $ENV{NEW}/g;
	s/\.typedefs\/\Q$ENV{OLD}\E\//.typedefs\/$ENV{NEW}\//g;
	s/Lune v\Q$ENV{OLD}\E/Lune v$ENV{NEW}/g;
' {} +

# Cargo.lock records the version of every workspace member.
cargo update --workspace --quiet

echo "bump-version: $OLD -> $NEW"
echo "  Now give CHANGELOG.md's '## Unreleased' the heading of $NEW, then run scripts/check-versions.sh."
