#!/bin/sh
set -eu
repo=$(CDPATH='' cd -- "$(dirname "$0")/../.." && pwd)
check="$repo/tools/release-check.sh"
work=$(mktemp -d "${TMPDIR:-/tmp}/multi-link-release-test.XXXXXX")
trap 'rm -rf "$work"' 0
trap 'exit 1' HUP INT TERM
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
export GIT_AUTHOR_NAME='Release Test' GIT_AUTHOR_EMAIL='release@example.invalid'
export GIT_COMMITTER_NAME="$GIT_AUTHOR_NAME" GIT_COMMITTER_EMAIL="$GIT_AUTHOR_EMAIL"
git init -q --bare --initial-branch=main "$work/origin.git"
git clone -q "$work/origin.git" "$work/clone" 2> "$work/clone.err"
cd "$work/clone"
mkdir -p apps/menubar
cat > apps/menubar/Info.plist <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleShortVersionString</key><string>1.19</string>
</dict></plist>
PLIST
git add apps/menubar/Info.plist
git commit -qm 'Add version'
main=$(git rev-parse HEAD)
git push -q origin main
git tag -a v1.19 -m 'Release'
expect_ok() {
    if ! sh "$check" "$1" "$2" > "$work/out" 2> "$work/err"; then
        cat "$work/err" >&2
        printf 'FAIL: %s\n' "$3" >&2
        exit 1
    fi
    printf 'PASS: %s\n' "$3"
}
expect_bad() {
    if sh "$check" "$1" "$2" > "$work/out" 2> "$work/err"; then
        printf 'FAIL: accepted %s\n' "$3" >&2
        exit 1
    fi
    if ! grep -F "$4" "$work/err" > /dev/null; then
        cat "$work/err" >&2
        printf 'FAIL: missing diagnostic for %s\n' "$3" >&2
        exit 1
    fi
    printf 'PASS: %s\n' "$3"
}
expect_ok v1.19 "$main" 'matching annotated tag on main'
git tag v1.20
expect_bad v1.20 "$main" 'mismatched version' 'does not match bundle version'
for tag in v1.19-rc1 1.19 v1.19.0.1; do
    expect_bad "$tag" "$main" "invalid tag $tag" 'invalid release tag'
done
git switch -qc other
sed 's/1.19/1.19.1/' apps/menubar/Info.plist > "$work/plist"
cp "$work/plist" apps/menubar/Info.plist
git commit -qam 'Change version'
other=$(git rev-parse HEAD)
git tag v1.19.1
expect_bad v1.19 "$other" 'version read at requested commit' 'does not match bundle version'
# The version at other matches this tag, but the tag points at main.
git tag -f v1.19.1 "$main" > /dev/null
expect_bad v1.19.1 "$other" 'tag points at another commit' 'does not resolve to commit'
git tag -f v1.19.1 "$other" > /dev/null
expect_bad v1.19.1 "$other" 'commit only on another branch' 'not an ancestor of origin/main'
git switch -q main
git merge -q --ff-only other
git push -q origin main
# Invoke below the root, with a different working-tree version.
printf 'working tree is not the tagged plist\n' > apps/menubar/Info.plist
cd apps/menubar
expect_ok v1.19.1 "$other" 'patch tag and committed plist from nested directory'

ruby -ryaml -rtmpdir -rfileutils - "$repo/.github/workflows/release.yml" <<'RUBY'
workflow = YAML.load_file(ARGV.fetch(0))
workflow.fetch('jobs').each_value do |job|
  uses = [job['uses']] + job.fetch('steps', []).map { |step| step['uses'] }
  uses.compact.each do |value|
    abort "Unpinned uses: #{value}" unless value == './.github/workflows/ci.yml' || value.match?(/@\h{40}\z/)
  end
end
puts 'PASS: all workflow uses pinned or local CI'
step = workflow.fetch('jobs').fetch('release').fetch('steps').find { |item| item['name'] == 'Create draft release' }
abort 'Missing draft release step' unless step
Dir.mktmpdir('release-gh-test') do |dir|
  bin = File.join(dir, 'bin')
  FileUtils.mkdir_p(bin)
  File.write(File.join(bin, 'gh'), "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$GH_ARGS\"\n")
  FileUtils.chmod(0755, File.join(bin, 'gh'))
  dmg = File.join(dir, 'build assets', 'multi-link-1.19-arm64.dmg')
  FileUtils.mkdir_p(File.dirname(dmg))
  File.write(dmg, 'fixture')
  File.write("#{dmg}.sha256", 'fixture checksum')
  env = { 'PATH' => "#{bin}:#{ENV.fetch('PATH')}", 'TAG' => 'v1.19', 'DMG' => dmg,
          'GH_TOKEN' => 'test-token', 'GH_ARGS' => File.join(dir, 'args') }
  abort 'gh step failed' unless system(env, 'bash', '-euo', 'pipefail', '-c', step.fetch('run'), chdir: dir)
  expected = ['release', 'create', 'v1.19', '--draft', '--verify-tag',
              '--title', 'multi-link 1.19',
              '--notes', 'Draft built by the release workflow. Check the two files, then publish.',
              dmg, "#{dmg}.sha256"]
  actual = File.readlines(env.fetch('GH_ARGS'), chomp: true)
  abort "Unexpected gh arguments: #{actual.inspect}" unless actual == expected
end
puts 'PASS: draft release receives exact tag, flags, DMG and checksum'
RUBY
