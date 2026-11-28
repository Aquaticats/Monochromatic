# the-rule-files-prose-walked.py
# Counts the breaks inside paragraphs of one Markdown file and classes each break that falls at no
# semantic boundary, by the first token of the next line's prose. It reads a copy of the file at the
# survey commit, made with `git show bc5b4ca2ba:AGENTS.md > <root>/AGENTS.md`.
# This is a line walker, not the linter: it reads no syntax tree and lays no paragraph out, so its
# figures are indicative and the survey's rustdoc reader is what counts for the corpus.
# Run: python3 -I the-rule-files-prose-walked.py <root>
import os
import re
import sys

path = os.path.join(sys.argv[1], "AGENTS.md")
CLOSERS = "\"')]}”’»›"
BREAK_POINTS = ",.;:?!"
CONJUNCTIONS = set(
    "and but or nor yet so for because although though while when where which who whom whose that if unless"
    " until since as before after whether than then plus versus via".split()
)


def ends_at_break_point(text: str) -> bool:
    stripped = text.rstrip()
    while stripped and stripped[-1] in CLOSERS:
        stripped = stripped[:-1]
    return bool(stripped) and stripped[-1] in BREAK_POINTS


def starts_block(text: str) -> bool:
    stripped = text.strip()
    return stripped[:1] in ">#|" or bool(re.match(r"^(```|~~~|[-*+] |\d+[.)]|[A-Z0-9]{3}:)", stripped))


def classify(next_text: str, previous_text: str) -> str:
    text = next_text.strip()
    if starts_block(text):
        return "block-marker"
    if text.startswith("("):
        return "parenthesis"
    if text[:1] in "`*_~[!":
        return "inline-span"
    first = re.match(r"[A-Za-z]+", text)
    if first and first.group(0).lower() in CONJUNCTIONS:
        return "conjunction"
    if previous_text.rstrip().endswith(("--", "—", "–")):
        return "dash"
    return "ordinary"


lines = open(path, encoding="utf-8").read().split("\n")
paragraph = []
for line in lines:
    stripped = line.strip()
    paragraph.append(bool(stripped) and not starts_block(stripped) and not stripped.startswith("---"))
breaks = 0
no_boundary = []
dashes = sum(1 for line in lines if line.strip().endswith(("--", "—", "–")))
for index in range(1, len(lines)):
    # the break between line index and the line before it, when both are paragraph lines
    if not (paragraph[index] and paragraph[index - 1]):
        continue
    breaks += 1
    if not ends_at_break_point(lines[index - 1]):
        no_boundary.append((index, classify(lines[index], lines[index - 1]), lines[index - 1].strip()))

counts = {}
for _, kind, _text in no_boundary:
    counts[kind] = counts.get(kind, 0) + 1

print(f"file: {path}")
print(f"breaks inside paragraphs: {breaks}")
print(f"breaks at no boundary: {len(no_boundary)}")
for kind in ("block-marker", "conjunction", "dash", "inline-span", "ordinary", "parenthesis"):
    print(f"  {kind}: {counts.get(kind, 0)}")
print(f"lines ending in a dash: {dashes}")
print()
print("each break at no boundary, with the line it ends:")
for number, kind, text in no_boundary:
    print(f"  line {number} [{kind}] {text[:110]}")
print()
print("a break before a conjunction or a parenthesis is not a break the rule's fixer writes: it writes only")
print("after one of its six characters, so those two kinds cannot be its output in this file")
