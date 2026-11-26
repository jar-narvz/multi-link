#!/bin/sh
# Tests for tools/check-links.py on throwaway trees (Step 26.3).
set -eu
repo=$(CDPATH='' cd -- "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d "${TMPDIR:-/tmp}/multi-link-links-test.XXXXXX")
trap 'rm -rf "$work"' 0
trap 'exit 1' HUP INT TERM
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
pass() { printf 'PASS: %s\n' "$*"; }
check="$repo/tools/check-links.py"
[ -f "$check" ] || fail 'tools/check-links.py is missing'

# new_tree NAME: an empty tree with one target file and one sub-folder.
new_tree() {
    t="$work/$1"
    mkdir -p "$t/docs/history" "$t/img"
    printf '# target\n' > "$t/docs/target.md"
    printf 'png' > "$t/img/pic.png"
    printf '%s' "$t"
}
run() { # run TREE: runs the checker, keeps stdout/stderr, sets $code
    if python3 -I "$check" "$1" > "$work/out" 2> "$work/err"; then code=0; else code=$?; fi
}

t=$(new_tree good)
cat > "$t/README.md" <<'EOF'
See [target](docs/target.md) and [its anchor](docs/target.md#target) and [the folder](docs/history/).
EOF
cat > "$t/docs/history/a.md" <<'EOF'
Up two: [target](../target.md). Self anchor: [top](#top).
EOF
run "$t"
[ "$code" -eq 0 ] || { cat "$work/out" "$work/err" >&2; fail 'good links must pass'; }
grep -qx 'check-links: 4 links ok' "$work/out" || { cat "$work/out" >&2; fail 'good tree summary'; }
pass 'good links (file, anchor, directory, ../) pass with a count'

t=$(new_tree broken)
printf 'line one\nline two has [gone](docs/missing.md#x)\n' > "$t/README.md"
run "$t"
[ "$code" -eq 1 ] || fail "broken link must exit 1, got $code"
grep -qx 'README.md:2: docs/missing.md#x' "$work/out" || { cat "$work/out" >&2; fail 'broken link not reported as file:line: dest'; }
pass 'broken link fails with file:line: dest'

t=$(new_tree nested)
printf '[gone](nope.md)\n' > "$t/docs/history/b.md"
run "$t"
[ "$code" -eq 1 ] || fail 'broken link in a sub-folder must fail'
grep -qx 'docs/history/b.md:1: nope.md' "$work/out" || { cat "$work/out" >&2; fail 'nested link resolves from its own file'; }
pass 'links resolve from the file that holds them'

t=$(new_tree code)
cat > "$t/README.md" <<'EOF'
Inline code `[x](missing-inline.md)` is ignored, and so is ``a `[y](missing-double.md)` b``.
```
[z](missing-fence.md)
```
~~~sh
![w](missing-tilde.png)
~~~
After the fence: [ok](docs/target.md)
EOF
run "$t"
[ "$code" -eq 0 ] || { cat "$work/out" >&2; fail 'links in inline code or fences must be ignored'; }
grep -qx 'check-links: 1 links ok' "$work/out" || { cat "$work/out" >&2; fail 'code tree summary'; }
pass 'links inside backticks and fences are ignored'

t=$(new_tree image)
printf '![pic](img/pic.png)\n![gone](img/missing.png)\n' > "$t/README.md"
run "$t"
[ "$code" -eq 1 ] || fail 'broken image must fail'
grep -qx 'README.md:2: img/missing.png' "$work/out" || { cat "$work/out" >&2; fail 'broken image not reported'; }
[ "$(wc -l < "$work/out" | tr -d ' ')" -eq 1 ] || { cat "$work/out" >&2; fail 'good image must not be reported'; }
pass 'image links are checked'

t=$(new_tree external)
cat > "$t/README.md" <<'EOF'
[site](https://example.com/nope.md) [plain](http://example.com/x) [mail](mailto:a@example.com) [frag](#nowhere)
EOF
run "$t"
[ "$code" -eq 0 ] || { cat "$work/out" >&2; fail 'external, mailto and fragment links must be skipped'; }
grep -qx 'check-links: 0 links ok' "$work/out" || { cat "$work/out" >&2; fail 'skipped links must not be counted'; }
pass 'https, http, mailto and fragment-only links are skipped'

t=$(new_tree skipdirs)
for d in fixtures target .claude apps/menubar/.build; do
    mkdir -p "$t/$d"
    printf '[gone](missing.md)\n' > "$t/$d/x.md"
done
run "$t"
[ "$code" -eq 0 ] || { cat "$work/out" >&2; fail 'fixtures/, target/, .claude/ and apps/menubar/.build/ must be skipped'; }
pass 'excluded folders are skipped'

t=$(new_tree untracked)
printf '[target](target.md)\n' > "$t/docs/new.md"
run "$t"
[ "$code" -eq 0 ] || fail 'a new file on disk must be read'
grep -qx 'check-links: 1 links ok' "$work/out" || { cat "$work/out" >&2; fail 'new file not counted'; }
pass 'files on disk are read (no git needed)'

t=$(new_tree edges)
printf '# a(b)\n' > "$t/docs/a(b).md"
printf '# a&b\n' > "$t/docs/a&b.md"
cat > "$t/README.md" <<'EOF2'
Escaped backticks are text: \` [gone1](missing-escaped.md) \`
A title is not a link: [ok](docs/target.md "see [y](missing-title.md)") and [ok2](docs/target.md 'x](nope.md)')
Escapes and entities: [p](docs/a\(b\).md) [amp](docs/a&amp;b.md) [angle](<docs/a(b).md>)
```text`not-a-fence`
[gone2](missing-after-bad-fence.md)
EOF2
run "$t"
[ "$code" -eq 1 ] || { cat "$work/out" >&2; fail 'edge tree must fail on the two real broken links'; }
printf 'README.md:1: missing-escaped.md\nREADME.md:5: missing-after-bad-fence.md\n' > "$work/want"
diff -u "$work/want" "$work/out" >&2 || fail 'edge tree: wrong report'
pass 'escaped backticks, titles, escapes, entities and a backtick info string'
