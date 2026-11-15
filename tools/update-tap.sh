#!/bin/sh
set -eu

fail() {
    printf 'update-tap: %s\n' "$*" >&2
    exit 1
}

[ "$#" -eq 1 ] || fail 'usage: tools/update-tap.sh <clone|prepare|push>'
root=$(CDPATH='' cd -- "$(dirname "$0")/.." && pwd)
[ -n "${WORK_DIR:-}" ] || fail 'WORK_DIR is required'
[ -n "${TAP_DIR:-}" ] || fail 'TAP_DIR is required'
mkdir -p "$WORK_DIR"
WORK_DIR=$(CDPATH='' cd -- "$WORK_DIR" && pwd)
marker="$WORK_DIR/push-needed"
tap_url=https://github.com/jar-narvz/homebrew-tap.git
# Only the offline test harness can select a local repository; never another URL.
if [ -n "${TAP_URL:-}" ]; then
    [ "${UPDATE_TAP_TESTING:-}" = 1 ] || fail 'TAP_URL is for tests only'
    case "$TAP_URL" in /*) tap_url=$TAP_URL ;; *) fail 'test TAP_URL must be an absolute local path' ;; esac
fi

case "$1" in
    clone)
        git clone "$tap_url" "$TAP_DIR"
        ;;
    prepare)
        rm -f "$marker"
        [ "${PRERELEASE:-}" = false ] || fail 'prerelease releases are refused (PRERELEASE must be false)'
        version=$(ruby -e 'tag = ARGV.fetch(0); abort "update-tap: invalid release tag" unless tag.match?(/\Av[0-9]+\.[0-9]+(\.[0-9]+)?\z/); puts tag[1..-1]' "${TAG:-}") || exit 1
        [ -d "$TAP_DIR/.git" ] || fail 'tap checkout is missing; run clone first'
        TAP_DIR=$(CDPATH='' cd -- "$TAP_DIR" && pwd)
        [ -z "$(git -C "$TAP_DIR" status --porcelain)" ] || fail 'tap checkout must be clean'
        if [ -f "$TAP_DIR/Casks/multi-link.rb" ]; then
            order=$(ruby - "$TAP_DIR/Casks/multi-link.rb" "$version" <<'RUBY'
source = File.read(ARGV.fetch(0))
versions = source.scan(/^  version "([0-9]+\.[0-9]+(?:\.[0-9]+)?)"$/).flatten
abort 'update-tap: cannot read exactly one current cask version' unless versions.length == 1
parts = lambda { |value| fields = value.split('.').map(&:to_i); fields << 0 if fields.length == 2; fields }
puts(parts.call(ARGV.fetch(1)) <=> parts.call(versions.first))
RUBY
            ) || exit 1
            case "$order" in
                0) printf 'update-tap: %s already published; no-op\n' "$version"; exit 0 ;;
                -1) fail "release $version is older than the tap version" ;;
            esac
        fi
        dmg="multi-link-$version-arm64.dmg"
        download=$(mktemp -d "$WORK_DIR/download.XXXXXX")
        trap 'rm -rf "$download"' 0
        trap 'exit 1' HUP INT TERM
        gh release view "$TAG" --repo jar-narvz/multi-link --json assets --jq '.assets[].name' > "$download/assets"
        printf '%s\n%s\n' "$dmg" "$dmg.sha256" | LC_ALL=C sort > "$download/expected"
        LC_ALL=C sort "$download/assets" > "$download/actual"
        cmp -s "$download/expected" "$download/actual" || fail 'release must have exactly the expected DMG and checksum assets'
        gh release download "$TAG" --repo jar-narvz/multi-link --dir "$download"
        # Validate the checksum's basename before letting shasum read any path.
        sha=$(ruby - "$download/$dmg.sha256" "$dmg" <<'RUBY'
checksum = File.read(ARGV.fetch(0))
match = /\A([0-9a-f]{64})  #{Regexp.escape(ARGV.fetch(1))}\n\z/.match(checksum)
abort 'update-tap: invalid checksum file' unless match
puts match[1]
RUBY
        ) || exit 1
        (cd "$download" && shasum -a 256 -c "$dmg.sha256") || fail 'release checksum mismatch'
        mkdir -p "$TAP_DIR/Casks"
        sh "$root/tools/render-cask.sh" "$version" "$sha" > "$TAP_DIR/Casks/multi-link.rb"
        git -C "$TAP_DIR" add Casks/multi-link.rb
        GIT_AUTHOR_NAME='github-actions[bot]' \
        GIT_AUTHOR_EMAIL='41898282+github-actions[bot]@users.noreply.github.com' \
        GIT_COMMITTER_NAME='github-actions[bot]' \
        GIT_COMMITTER_EMAIL='41898282+github-actions[bot]@users.noreply.github.com' \
            git -C "$TAP_DIR" -c commit.gpgsign=false commit -m "Update multi-link to $version"
        git -C "$TAP_DIR" rev-parse HEAD > "$marker"
        ;;
    push)
        if [ ! -f "$marker" ]; then
            printf 'update-tap: nothing to push; no-op\n'
            exit 0
        fi
        [ -n "${TAP_TOKEN:-}" ] || fail 'TAP_TOKEN is empty; tap was not pushed'
        [ "$(git -C "$TAP_DIR" rev-parse HEAD)" = "$(cat "$marker")" ] || fail 'tap HEAD changed after prepare'
        askpass=$(mktemp "$WORK_DIR/askpass.XXXXXX")
        trap 'rm -f "$askpass"' 0
        trap 'exit 1' HUP INT TERM
        cat > "$askpass" <<'ASKPASS'
#!/bin/sh
case "$1" in
    *Username*) printf '%s\n' 'x-access-token' ;;
    *Password*) printf '%s\n' "$TAP_TOKEN" ;;
    *) exit 1 ;;
esac
ASKPASS
        chmod 700 "$askpass"
        # The secret stays in the environment, never the URL, argv, or git config.
        GIT_ASKPASS="$askpass" GIT_TERMINAL_PROMPT=0 GIT_TRACE=0 GIT_TRACE_CURL=0 GIT_CURL_VERBOSE=0 \
            git -C "$TAP_DIR" -c credential.helper= push "$tap_url" HEAD:refs/heads/main
        rm -f "$marker"
        ;;
    *) fail 'usage: tools/update-tap.sh <clone|prepare|push>' ;;
esac
