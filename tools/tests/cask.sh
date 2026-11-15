#!/bin/sh
set -eu
repo=$(CDPATH='' cd -- "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d "${TMPDIR:-/tmp}/multi-link-cask-test.XXXXXX")
trap 'rm -rf "$work"' 0
trap 'exit 1' HUP INT TERM
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
pass() { printf 'PASS: %s\n' "$*"; }
sha=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
render="$repo/tools/render-cask.sh"
update="$repo/tools/update-tap.sh"
[ -f "$render" ] || fail 'cask renderer is missing'
sh "$render" 1.18 "$sha" > "$work/cask.rb"
diff -u "$repo/tools/tests/cask-1.18.rb" "$work/cask.rb"
ruby -c "$work/cask.rb"
pass 'rendered 1.18 cask equals expected fixture'
bad_render() {
    if sh "$render" "$@" > "$work/out" 2> "$work/err"; then fail 'accepted bad render input'; fi
    [ ! -s "$work/out" ] && [ -s "$work/err" ] || fail 'bad render must only print a diagnostic on stderr'
}
bad_render
bad_render 1.18
bad_render '' "$sha"
bad_render 1.18 ''
bad_render 1.18 abc
bad_render 1.18 g123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
bad_render 1.18 ABCDEF6789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
for version in v1.18 1 1.18-rc1 1.18.0.1 '1.18
2.0'; do bad_render "$version" "$sha"; done
bad_render 1.18 "$sha" --url
bad_render 1.18 "$sha" --url ''
bad_render 1.18 "$sha" --url http://example.com/x
bad_render 1.18 "$sha" --url http://127.0.0.10/x
bad_render 1.18 "$sha" --other https://example.com/x
bad_render 1.18 "$sha" extra
bad_render 1.18 "$sha" --url https://example.com/x extra
sh "$render" 1.18.1 "$sha" --url http://127.0.0.1:8766/test.dmg > "$work/local.rb"
ruby -c "$work/local.rb"
grep -F 'url "http://127.0.0.1:8766/test.dmg"' "$work/local.rb" > /dev/null
pass 'render input validation and local URL override'
sh "$render" 1.18.1 "$sha" --url 'http://127.0.0.1:8766/test-#{version}.dmg' > "$work/versioned.rb"
ruby -c "$work/versioned.rb"
grep -Fx '  url "http://127.0.0.1:8766/test-#{version}.dmg"' "$work/versioned.rb" > /dev/null || fail 'version interpolation was escaped'
pass 'local URL preserves literal version interpolation'
for url in 'https://example.com/test-".dmg' 'https://example.com/test-\.dmg' \
    'https://example.com/test-#{arch}.dmg' 'https://example.com/test-#{system}.dmg' \
    'https://example.com/test-#{version}-#{arch}.dmg' 'https://example.com/test-#{version.dmg' \
    'https://example.com/test file.dmg' 'https://example.com/test
file.dmg'; do
    bad_render 1.18 "$sha" --url "$url"
done
bad_render 1.18 "$sha" --url "$(printf 'https://example.com/test\tfile.dmg')"
bad_render 1.18 "$sha" --url "$(printf 'https://example.com/test\rfile.dmg')"
pass 'unsafe URL forms rejected with stderr only'

export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
export GIT_AUTHOR_NAME='Cask Test' GIT_AUTHOR_EMAIL='cask@example.invalid'
export GIT_COMMITTER_NAME="$GIT_AUTHOR_NAME" GIT_COMMITTER_EMAIL="$GIT_AUTHOR_EMAIL"
export UPDATE_TAP_TESTING=1 TAP_URL="$work/tap.git" TAP_DIR="$work/tap" WORK_DIR="$work/state"
export TAG=v1.19 PRERELEASE=false TAP_TOKEN=''
export RELEASE_DIR="$work/release" ASSET_MODE=good
mkdir -p "$work/bin" "$RELEASE_DIR"
git init -q --bare --initial-branch=main "$TAP_URL"
cat > "$work/bin/gh" <<'GH'
#!/bin/sh
set -eu
[ "${GH_DISABLED:-false}" = false ] || exit 90
[ "$1" = release ] && [ "$3" = "$TAG" ] || exit 91
case "$2" in
    view)
        [ "$#" -eq 9 ] && [ "$4" = --repo ] && [ "$5" = jar-narvz/multi-link ] &&
            [ "$6" = --json ] && [ "$7" = assets ] && [ "$8" = --jq ] && [ "$9" = '.assets[].name' ] || exit 92
        printf 'multi-link-%s-arm64.dmg\n' "${TAG#v}"
        [ "$ASSET_MODE" = missing ] || printf 'multi-link-%s-arm64.dmg.sha256\n' "${TAG#v}"
        [ "$ASSET_MODE" != extra ] || printf 'unexpected.zip\n'
        ;;
    download)
        [ "$#" -eq 7 ] && [ "$4" = --repo ] && [ "$5" = jar-narvz/multi-link ] && [ "$6" = --dir ] || exit 93
        cp "$RELEASE_DIR/"* "$7/"
        ;;
    *) exit 94 ;;
esac
GH
chmod +x "$work/bin/gh"
export PATH="$work/bin:$PATH"
make_release() {
    rm -f "$RELEASE_DIR/"*
    dmg="multi-link-${TAG#v}-arm64.dmg"
    printf 'test release %s\n' "$TAG" > "$RELEASE_DIR/$dmg"
    (cd "$RELEASE_DIR" && shasum -a 256 "$dmg" > "$dmg.sha256")
}
run() { sh "$update" "$1" > "$work/out" 2> "$work/err"; }
ok() { if ! run "$1"; then cat "$work/out" "$work/err" >&2; fail "$2"; fi; pass "$2"; }
bad() {
    if run "$1"; then fail "accepted $2"; fi
    grep -F "$3" "$work/err" > /dev/null || { cat "$work/err" >&2; fail "missing diagnostic: $2"; }
    pass "$2"
}
make_release
ok clone 'public tap clone without token'
(PRERELEASE=true; bad prepare 'prerelease rejected' 'prerelease')
(TAG=v1.19-rc1; bad prepare 'bad tag rejected' 'invalid release tag')
(ASSET_MODE=missing; bad prepare 'missing asset rejected' 'exactly')
(ASSET_MODE=extra; bad prepare 'extra asset rejected' 'exactly')
printf 'tampered\n' >> "$RELEASE_DIR/$dmg"
bad prepare 'checksum mismatch rejected' 'checksum'
make_release
ok prepare 'first publish prepares a commit'
[ "$(git -C "$TAP_DIR" log -1 --format='%an <%ae>|%cn <%ce>')" = 'github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>|github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>' ] || fail 'bot identity'
bad push 'missing TAP_TOKEN rejected' 'TAP_TOKEN'
[ -z "$(git --git-dir="$TAP_URL" for-each-ref --format='%(refname)')" ] || fail 'pushed without token'
(TAP_TOKEN=test-only; ok push 'first publish pushes to local bare tap')
first=$(git --git-dir="$TAP_URL" rev-parse main)
(GH_DISABLED=true; export GH_DISABLED; ok prepare 'same-version rerun prepares no-op without gh')
grep -F 'no-op' "$work/out" > /dev/null || fail 'no no-op diagnostic'
ok push 'same-version rerun needs no token'
[ "$(git --git-dir="$TAP_URL" rev-parse main)" = "$first" ] || fail 'same-version changed tap'
(TAG=v1.18; bad prepare 'older version rejected' 'older')
(TAG=v1.9; bad prepare 'numeric older version rejected' 'older')
TAG=v1.19.1
make_release
ok prepare 'newer patch release prepares a commit'
(TAP_TOKEN=test-only; ok push 'newer release pushes to local bare tap')
[ "$(git --git-dir="$TAP_URL" show main:Casks/multi-link.rb | sed -n 's/^  version "\([^"]*\)"/\1/p')" = 1.19.1 ] || fail 'wrong pushed version'
release_sha=$(shasum -a 256 "$RELEASE_DIR/$dmg" | cut -d ' ' -f 1)
sh "$render" 1.19.1 "$release_sha" > "$work/expected.rb"
git --git-dir="$TAP_URL" show main:Casks/multi-link.rb > "$work/pushed.rb"
diff -u "$work/expected.rb" "$work/pushed.rb"
pass 'pushed cask matches the verified release'
TAG=v1.100
make_release
ok prepare 'numeric newer release prepares a commit'
(TAP_TOKEN=test-only; ok push 'numeric newer release pushes to local bare tap')
[ "$(git --git-dir="$TAP_URL" show main:Casks/multi-link.rb | sed -n 's/^  version "\([^"]*\)"/\1/p')" = 1.100 ] || fail 'numeric version order'
pass 'version ordering compares numeric components'

ruby -ryaml - "$repo" <<'RUBY'
repo = ARGV.fetch(0)
%w[ci release cask].each do |name|
  workflow = YAML.load_file("#{repo}/.github/workflows/#{name}.yml")
  workflow.fetch('jobs').each_value do |job|
    ([job['uses']] + job.fetch('steps', []).map { |step| step['uses'] }).compact.each do |value|
      abort "Unpinned uses: #{value}" unless value == './.github/workflows/ci.yml' || value.match?(/@[0-9a-f]{40}\z/)
    end
  end
end
workflow = YAML.load_file("#{repo}/.github/workflows/cask.yml")
abort 'Wrong release trigger' unless workflow.fetch('on', workflow[true]) == { 'release' => { 'types' => ['published'] } }
abort 'Wrong permissions' unless workflow['permissions'] == { 'contents' => 'read' }
abort 'Wrong concurrency' unless workflow['concurrency'] == { 'group' => 'cask-tap', 'cancel-in-progress' => false }
steps = workflow.fetch('jobs').fetch('cask').fetch('steps')
checkout = steps.first
abort 'Checkout must use default branch without credentials' unless checkout.fetch('with') == {
  'ref' => '${{ github.event.repository.default_branch }}', 'persist-credentials' => false
}
abort 'Token outside final step' if YAML.dump(workflow.reject { |key, _| key == 'jobs' }).include?('TAP_TOKEN')
workflow.fetch('jobs').each_value do |job|
  abort 'Token in job env' if YAML.dump(job.reject { |key, _| key == 'steps' }).include?('TAP_TOKEN')
end
steps.each_with_index do |step, index|
  abort 'Event interpolated in run' if step.fetch('run', '').include?('${{ github.event')
  abort 'Token outside final step' if index < steps.length - 1 && YAML.dump(step).include?('TAP_TOKEN')
  abort 'Token in run' if step.fetch('run', '').include?('TAP_TOKEN')
end
abort 'Missing final push token' unless steps.last.fetch('env')['TAP_TOKEN'] == '${{ secrets.TAP_TOKEN }}'
abort 'Wrong final command' unless steps.last['run'] == 'sh tools/update-tap.sh push'
release_steps = steps.select { |step| step.fetch('env', {}).key?('GH_TOKEN') }
abort 'Release token scope' unless release_steps.length == 1 && release_steps.first['run'] == 'sh tools/update-tap.sh prepare'
abort 'Missing event env' unless workflow.fetch('jobs').fetch('cask').fetch('env').values_at('TAG', 'PRERELEASE') == [
  '${{ github.event.release.tag_name }}', '${{ github.event.release.prerelease }}'
]
puts 'PASS: workflow triggers, pins, default branch, event and token boundaries'
RUBY
