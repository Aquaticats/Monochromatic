# the-proposals-quotations-checked.py
# Checks every fenced block of the proposal that quotes a repository file.
# For each block it collects the paths the preceding twelve non-blank lines name, and the line number a
# path is named with, then looks for the block's lines, in order, among that file's lines.
# A block that matches no candidate is reported as a mismatch unless it is one of the fixtures and tables
# listed below, which quote no file.
# Run from the folder holding the copies:
#   python3 -I the-proposals-quotations-checked.py <proposal.md> <copies-root> <worktree-root>
# <copies-root> holds the files at the survey commit under their repository paths, and <worktree-root>
# is where a file quoted as it stands today is read.
import os
import re
import sys

proposal_path, copies_root, worktree_root = sys.argv[1], sys.argv[2], sys.argv[3]
lines = open(proposal_path, encoding="utf-8").read().split("\n")

# Blocks that quote no repository file: fixtures the proposal shows, tables it derives, and the
# commands and outputs it prints. Each is named by its own first line.
NOT_A_QUOTATION = (
    "paths in the kept list:",
    "block-marker      65",
    "doc/planning        45,636",
    "# from package/linter/monochromatic-lint",
    "kind              A",
    "no to all five kinds",
    "Aaa bbb",
    "running 1 test",
    "$ monochromatic-lint",
    "$ cat fixed-comma.md",
    "$ git show",
    "SURVEY_LIST=",
    "ec6828ff",
)

blocks = []
window = []
index = 0
while index < len(lines):
    line = lines[index]
    fence = re.match(r"^(\s*)```(\w*)$", line)
    if fence:
        indent = fence.group(1)
        body = []
        index += 1
        while index < len(lines) and not re.match(r"^\s*```$", lines[index]):
            body.append(lines[index][len(indent):] if lines[index].startswith(indent) else lines[index])
            index += 1
        blocks.append((index, list(window[-12:]), body))
        index += 1
        continue
    if line.strip():
        window.append(line.strip())
    index += 1

PAIR = re.compile(r"`([\w./@+-]+\.(?:md|rs)):(\d+)`")
PLAIN = re.compile(r"`([\w./@+-]+\.(?:md|rs))`")

matched = 0
skipped = 0
problems = 0
for end_line, before, body in blocks:
    wanted = [item.strip() for item in body]
    first = body[0] if body else ""
    names_a_path = any(PAIR.search(item) or PLAIN.search(item) for item in before)
    if first.startswith(NOT_A_QUOTATION) and not first.startswith("\"Put the semantic"):
        skipped += 1
        print(f"quotes no file, at proposal line {end_line}: {first[:70]!r}")
        continue
    if not names_a_path and not first.startswith("\"Put the semantic"):
        skipped += 1
        print(f"quotes no file, at proposal line {end_line}: {first[:70]!r}")
        continue
    candidates = []
    if first.startswith("\"Put the semantic"):
        # the hold's words: the paragraph above them names the record, not the lines
        candidates.append(("doc/planning/repository-linter-stays-at-zero.md", 40))
    for item in before:
        for path, number in PAIR.findall(item):
            candidates.append((path, int(number)))
        for path in PLAIN.findall(item):
            candidates.append((path, None))
    found_at = None
    for path, number in reversed(candidates):
        for root in (worktree_root, copies_root):
            full = os.path.join(root, path)
            if not os.path.exists(full):
                continue
            file_lines = [entry.strip() for entry in open(full, encoding="utf-8").read().split("\n")]
            starts = (
                [number - 1 - offset for offset in range(len(wanted))]
                if number is not None and root == copies_root
                else list(range(0, len(file_lines)))
            )
            for start in starts:
                if start >= 0 and wanted == file_lines[start:start + len(wanted)]:
                    found_at = (path, start + 1, root)
                    break
            if found_at:
                break
        if found_at:
            break
    if found_at:
        matched += 1
        path, start, root = found_at
        print(f"ok, proposal line {end_line}: {len(wanted)} lines are {path} lines {start}..{start + len(wanted) - 1}")
    else:
        problems += 1
        print(f"MISMATCH at proposal line {end_line}: candidates {sorted(set(candidates))}")
        print(f"   block begins: {first[:100]}")

print()
print(f"blocks that quote a file and match it: {matched}")
print(f"blocks that quote no file: {skipped}")
print(f"blocks that do not match: {problems}")
sys.exit(1 if problems else 0)
