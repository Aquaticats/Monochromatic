# forced-clauses-a-rewrap-cannot-clear.py
# Existence check: a forced clause (120 characters or wider once joined) of three words or fewer has no
# rewrap that clears a finding of the short-line or the short-next-line predicate, because every line of
# such a rewrap holds one or two words and ends at no boundary. If every position a break could take in
# the clause is of a class some answer to the kinds question permits, then that answer clears it.
# Reads copies of the files at the survey commit under <root>/doc-audit (made with git show, one file a
# basename). Run: python3 -I forced-clauses-a-rewrap-cannot-clear.py <root>
import os
import re
import sys

root = os.path.join(sys.argv[1], "doc-audit")
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


def starts_block(text: str) -> bool:
    stripped = text.strip()
    return stripped[:1] in ">#|" or bool(re.match(r"^(```|~~~|[-*+] |\d+[.)] )", stripped))


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


# A paragraph is a run of non-blank lines whose first line is not a heading, a fence, a table row or a
# list marker; a clause is the run of lines from one ending at a break point to the next one.
clauses = []
files = sorted(os.listdir(root))
for name in files:
    lines = open(os.path.join(root, name), encoding="utf-8").read().split("\n")
    in_paragraph = False
    for index, line in enumerate(lines):
        stripped = line.strip()
        block = starts_block(stripped) or stripped.startswith("---")
        if not stripped:
            in_paragraph = False
            continue
        if block:
            in_paragraph = False
            continue
        if not in_paragraph:
            in_paragraph = True
            clause = []
        clause.append(line)
        if ends_at_break_point(line):
            if len(clause) >= 2:
                clauses.append((name, index - len(clause) + 2, clause))
            clause = []
    if in_paragraph and len(clause) >= 2:
        clauses.append((name, len(lines) - len(clause) + 1, clause))

forced = []
for name, first_line, clause in clauses:
    texts = [item.strip() for item in clause]
    width = sum(len(item) for item in texts) + (len(texts) - 1)
    if width < LIMIT:
        continue
    words = " ".join(texts).split()
    forced.append((name, first_line, width, words, texts))

print(f"files read: {len(files)}; clauses of two or more lines: {len(clauses)}; of those forced (width at least {LIMIT}): {len(forced)}")
short = [item for item in forced if len(item[3]) <= 3]
five = [item for item in forced if len(item[3]) <= 5]
print(f"forced clauses of three words or fewer: {len(short)}; of five words or fewer: {len(five)}")
print()
print("each forced clause of three words or fewer: the class of the break it would hold at each position")
for name, first_line, width, words, texts in short:
    positions = []
    for cut in range(1, len(words)):
        # the clause as text, with the break before word `cut` of `cut + 1`
        start = len(" ".join(words[:cut]))
        joined = " ".join(words)
        next_text = joined[start + 1:]
        previous_text = joined[:start]
        positions.append(classify(next_text, previous_text))
    only_ordinary = all(position == "ordinary" for position in positions)
    print(f"{name}:{first_line} width {width}, {len(words)} words: {positions}"
          + ("   NO REWRAP CLEARS IT UNDER ANY ANSWER" if only_ordinary else ""))
print()
print(f"of the {len(short)}, clauses whose every position is ordinary-class: {sum(1 for item in short if len(item[3]) > 1 and all(classify(' '.join(item[3])[len(' '.join(item[3][:cut])) + 1:], ' '.join(item[3])[:len(' '.join(item[3][:cut]))]) == 'ordinary' for cut in range(1, len(item[3]))))}")
