#!/usr/bin/env python3
"""Check that relative Markdown links in the repo point at files that exist (Step 26.3).

Usage: python3 -I tools/check-links.py [ROOT]   (ROOT defaults to the repo root)

Reads every *.md on disk under ROOT (so new, untracked files count), except under fixtures/, target/,
.claude/, apps/menubar/.build/ and .git/. Finds link and image destinations (`[text](dest)`,
`![alt](dest)`) outside inline code and fenced code blocks, skips http(s):, mailto: and fragment-only
(#...) links, drops any #anchor, undoes backslash escapes and HTML entities, and resolves the rest from
the file that holds them (a leading / means ROOT). Prints `file:line: dest` for each one that is not an
existing file or directory and exits 1; otherwise prints `check-links: N links ok` and exits 0.

Line-based, not a full CommonMark parser. Known limits: reference-style links are not checked; a link
destination or code span split across lines is not seen as one; fences inside blockquotes or list items
are not recognised (links in them are checked, so the failure is loud, not silent).
"""
import html
import os
import re
import sys
from urllib.parse import unquote

EXCLUDED = ("fixtures", "target", ".claude", ".git", os.path.join("apps", "menubar", ".build"))
SKIPPED_PREFIXES = ("http://", "https://", "mailto:")
FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})")


def strip_inline_code(line):
    """Blank out code spans: a run of N backticks up to the next run of exactly N backticks."""
    out, i = [], 0
    while i < len(line):
        if line[i] == "\\" and i + 1 < len(line):  # escaped char, e.g. \` is a literal backtick
            out.append(line[i:i + 2])
            i += 2
            continue
        if line[i] != "`":
            out.append(line[i])
            i += 1
            continue
        j = i
        while j < len(line) and line[j] == "`":
            j += 1
        run = j - i
        close = re.compile(r"(?<!`)`{%d}(?!`)" % run).search(line, j)
        if close is None:  # unmatched run: literal backticks
            out.append(line[i:j])
            i = j
        else:
            out.append(" " * (close.end() - i))
            i = close.end()
    return "".join(out)


def destinations(text):
    """Yield (line_number, destination) for every inline link or image outside code."""
    fence = None  # (char, length) of the open fence
    for number, line in enumerate(text.splitlines(), 1):
        m = FENCE.match(line)
        if fence is not None:
            if m and m.group(1)[0] == fence[0] and len(m.group(1)) >= fence[1] and \
                    line.strip() == m.group(1):
                fence = None
            continue
        # A backtick fence's info string may not contain a backtick (CommonMark).
        if m and not (m.group(1)[0] == "`" and "`" in line[m.end():]):
            fence = (m.group(1)[0], len(m.group(1)))
            continue
        line = strip_inline_code(line)
        done = 0  # end of the last link read, so a title's text is never read as a link
        for k in re.finditer(r"\]\(", line):
            if k.start() < done:
                continue
            dest, done = parse_destination(line, k.end())
            if dest:
                yield number, dest


def parse_destination(line, i):
    """Read a link destination starting right after `](`.

    Returns (dest or None, index just past the link's closing parenthesis, or past dest if none)."""
    while i < len(line) and line[i] == " ":
        i += 1
    if i < len(line) and line[i] == "<":
        end = line.find(">", i + 1)
        if end == -1:
            return None, i + 1
        return line[i + 1:end] or None, skip_title(line, end + 1)
    depth, j = 0, i
    while j < len(line):
        c = line[j]
        if c == "\\" and j + 1 < len(line):
            j += 2
            continue
        if c.isspace():
            break
        if c == "(":
            depth += 1
        elif c == ")":
            if depth == 0:
                break
            depth -= 1
        j += 1
    return line[i:j] or None, skip_title(line, j)


def skip_title(line, j):
    """Skip an optional "title", 'title' or (title) and the closing parenthesis; return the index after."""
    while j < len(line) and line[j] == " ":
        j += 1
    if j < len(line) and line[j] in "\"'(":
        close = ")" if line[j] == "(" else line[j]
        k = j + 1
        while k < len(line) and line[k] != close:
            k += 2 if line[k] == "\\" else 1
        j = k + 1
        while j < len(line) and line[j] == " ":
            j += 1
    return j + 1 if j < len(line) and line[j] == ")" else j


def unescape(dest):
    """Undo Markdown backslash escapes of ASCII punctuation and HTML entities."""
    return html.unescape(re.sub(r"\\([!-/:-@\[-`{-~])", r"\1", dest))


def markdown_files(root):
    for here, dirs, files in os.walk(root):
        rel = os.path.relpath(here, root)
        dirs[:] = sorted(d for d in dirs
                         if os.path.normpath(os.path.join(rel, d)) not in EXCLUDED)
        for name in sorted(files):
            if name.endswith(".md"):
                yield os.path.join(here, name)


def main(argv):
    if len(argv) > 2:
        print("usage: check-links.py [ROOT]", file=sys.stderr)
        return 2
    root = os.path.abspath(argv[1] if len(argv) == 2 else
                           os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
    checked, broken = 0, []
    for path in markdown_files(root):
        with open(path, encoding="utf-8") as f:
            text = f.read()
        rel = os.path.relpath(path, root)
        for number, dest in destinations(text):
            if dest.startswith(SKIPPED_PREFIXES) or dest.startswith("#"):
                continue
            checked += 1
            target = unquote(unescape(dest).split("#", 1)[0])
            base = root if target.startswith("/") else os.path.dirname(path)
            if not os.path.exists(os.path.join(base, target.lstrip("/"))):
                broken.append("%s:%d: %s" % (rel, number, dest))
    for line in broken:
        print(line)
    if broken:
        return 1
    print("check-links: %d links ok" % checked)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
