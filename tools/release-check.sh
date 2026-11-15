#!/bin/sh
set -eu

fail() {
    printf 'release-check: %s\n' "$*" >&2
    exit 1
}

[ "$#" -eq 2 ] || fail 'usage: tools/release-check.sh <tag> <commit>'
tag=$1
commit=$2
printf '%s\n' "$tag" | LC_ALL=C grep -Eq '^v[0-9]+\.[0-9]+(\.[0-9]+)?$' || fail "invalid release tag: $tag"
root=$(git rev-parse --show-toplevel) || fail 'must run inside the repository'
cd "$root"
resolved_commit=$(git rev-parse --verify --end-of-options "$commit^{commit}" 2>/dev/null) || fail "invalid commit: $commit"
plist=$(mktemp "${TMPDIR:-/tmp}/multi-link-release-plist.XXXXXX")
trap 'rm -f "$plist"' 0
trap 'exit 1' HUP INT TERM
git show "$resolved_commit:apps/menubar/Info.plist" > "$plist" 2>/dev/null || fail 'cannot read committed Info.plist'
version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$plist" 2>/dev/null) || fail 'cannot read CFBundleShortVersionString'
[ "$tag" = "v$version" ] || fail "tag $tag does not match bundle version $version"
tag_commit=$(git rev-parse --verify "refs/tags/$tag^{commit}" 2>/dev/null) || fail "tag $tag does not resolve to commit $resolved_commit"
[ "$tag_commit" = "$resolved_commit" ] || fail "tag $tag does not resolve to commit $resolved_commit"
git merge-base --is-ancestor "$resolved_commit" origin/main || fail "commit $resolved_commit is not an ancestor of origin/main"
printf 'release-check: %s matches bundle version and is on origin/main\n' "$tag"
