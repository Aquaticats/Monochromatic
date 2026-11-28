# the-examples-checked-against-their-files.py
# Checks, for every break the proposal quotes, that the line the proposal names is the line the break
# ends at, that the quotation the proposal prints is that file's own lines, and measures the class,
# the clause width, the forced flag and the word counts.
# The quoted text is read from the file here, never retyped, so this output is the text to quote.
# Reads copies of the files at the survey commit: make them with
#   git show bc5b4ca2ba:<path> > <root>/<path>
# and pass <root>. Run: python3 -I the-examples-checked-against-their-files.py <root>
import os
import re
import sys

root = sys.argv[1]
CLOSERS = "\"')]}”’»›"
BREAK_POINTS = ",.;:?!"
CONJUNCTIONS = set(
    "and but or nor yet so for because although though while when where which who whom whose that if unless"
    " until since as before after whether than then plus versus via".split()
)
LIMIT = 120


def ends_at_break_point(text: str) -> bool:
    stripped = text.rstrip()
    while stripped and stripped[-1] in CLOSERS:
        stripped = stripped[:-1]
    return bool(stripped) and stripped[-1] in BREAK_POINTS


def prose(line: str, is_rust: bool) -> str:
    if is_rust:
        line = re.sub(r"^\s*///\s?", "", line)
    return line


def classify(next_prose: str, previous_prose: str) -> str:
    text = next_prose.strip()
    if text[:1] in ">#|":
        return "block-marker"
    if re.match(r"^(```|~~~|[-*+] |\d+[.)] )", text):
        return "block-marker"
    if text.startswith("("):
        return "parenthesis"
    if text[:1] in "`*_~[!":
        return "inline-span"
    first = re.match(r"[A-Za-z]+", text)
    if first and first.group(0).lower() in CONJUNCTIONS:
        return "conjunction"
    if previous_prose.rstrip().endswith(("--", "—", "–")):
        return "dash"
    return "ordinary"


# label, path, the line the break ends at, the first and last line quoted, then the figures the
# proposal prints (None where it prints none).
EXAMPLES = [
    ("A, doc", "doc/philosophy/agents.md", 313, 312, 314, "ordinary", 107, False, 11, 2),
    ("A, Rust doc", "package/git-policy/cli/src/native/service/wait_ceilings.rs", 71, 70, 72, "ordinary", 48, False, 8, 1),
    ("B, doc", "doc/philosophy/agents.md", 606, 605, 607, "ordinary", 41, False, 2, 5),
    ("B, Rust doc", "package/git-policy/cli/src/native/service/wait_ceilings.rs", 192, 191, 193, "ordinary", 54, False, 2, 7),
    ("D leaves (forced)", "doc/philosophy/agents.md", 180, 180, 181, None, 129, True, None, None),
    ("F1", ".agents/skills/android-emulator/SKILL.md", 22, 22, 23, "inline-span", 162, True, None, 1),
    ("F2", ".agents/skills/css/SKILL.md", 170, 170, 171, "conjunction", 83, False, 2, 8),
    ("kind block-marker", "doc/audit/tech-mold-linker-vet-2026-10-05.md", 786, 786, 787, "block-marker", None, None, None, None),
    ("kind parenthesis", ".agents/skills/choosing-technology/SKILL.md", 151, 151, 152, "parenthesis", 128, True, None, 3),
    ("kind inline-span", ".agents/skills/android-emulator/SKILL.md", 22, 22, 23, "inline-span", 162, True, None, None),
    ("kind conjunction", ".agents/skills/css/SKILL.md", 170, 170, 171, "conjunction", 83, False, None, None),
    ("kind dash", ".agents/skills/project-code-review/PHILOSOPHY.md", 24, 24, 25, "dash", None, None, None, None),
    ("kind ordinary", ".agents/skills/css/SKILL.md", 54, 54, 55, "ordinary", 94, False, None, None),
    ("calibration ordinary", ".agents/skills/choosing-technology/SKILL.md", 688, 688, 689, "ordinary", 122, True, None, None),
]

problems = 0
for label, path, number, first, last, want_class, want_width, want_forced, want_own, want_next in EXAMPLES:
    is_rust = path.endswith(".rs")
    lines = open(os.path.join(root, path), encoding="utf-8").read().split("\n")
    own_raw, next_raw = lines[number - 1], lines[number]
    own, nxt = prose(own_raw, is_rust), prose(next_raw, is_rust)

    # the clause: from the previous line ending at a break point through the next one
    start = number - 1
    while start > 0 and prose(lines[start - 1], is_rust).strip() and not ends_at_break_point(prose(lines[start - 1], is_rust)):
        start -= 1
    end = number
    while end < len(lines) and prose(lines[end], is_rust).strip() and not ends_at_break_point(prose(lines[end], is_rust)):
        end += 1
    clause = [prose(line, is_rust).strip() for line in lines[start:end + 1]]
    width = sum(len(item) for item in clause) + (len(clause) - 1) + (1 if is_rust else 0)
    kind = classify(nxt, own)
    own_starts_clause = start == number - 1

    print(f"== {label}  {path}:{number}")
    print("   the quoted text, read from the file:")
    for index in range(first - 1, last):
        print(f"   |{lines[index]}")
    print(f"   the break ends at line {number} (the quotation's line {number - first + 1} of {last - first + 1})")
    print(f"   class {kind}; clause {width} characters (lines {start + 1}..{end + 1}); forced {width >= LIMIT}")
    print(f"   own line {len(own.split())} words; next line {len(nxt.split())} words; the clause opens on the break line: {own_starts_clause}")
    for what, measured, printed in (
        ("class", kind, want_class),
        ("clause width", width, want_width),
        ("forced", width >= LIMIT, want_forced),
        ("own-line words", len(own.split()), want_own),
        ("next-line words", len(nxt.split()), want_next),
    ):
        if printed is not None and measured != printed:
            problems += 1
            print(f"   DIFFERS: {what} measured {measured!r}, the proposal printed {printed!r}")
    print()

print(f"measurements that differ from the proposal: {problems}")
print("the parenthesis class's clause width is the one figure the review did not check; it is measured here")
