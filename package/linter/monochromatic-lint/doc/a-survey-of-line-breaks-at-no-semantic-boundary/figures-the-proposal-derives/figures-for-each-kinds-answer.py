# figures-for-each-kinds-answer.py
# Recomputes every figure the proposal prints from the survey's own rows, so that a reader can check
# each one against output.txt beside this folder:
#   - each kind's count in each predicate (a kind permitted is a row subtracted from every predicate),
#   - each predicate's share of ordinary-class findings under two kinds answers,
#   - each predicate's coverage of ordinary-class breaks, and the share a modeled join clears,
#   - each scope's count in each predicate's unit,
#   - the sums that must hold (each class column adds to its predicate's row; the dash row is one row).
# Run from this folder:
#   python3 -I figures-for-each-kinds-answer.py ../output.txt
# Every figure it prints is a sum or a quotient of rows of that file; it reads no other input.
import re
import sys

output = open(sys.argv[1], encoding="utf-8").read()

KINDS = ["block-marker", "conjunction", "dash", "inline-span", "ordinary", "parenthesis"]
# The two answers to the kinds question this proposal prints figures for.
ANSWERS = {
    "permit none of the six kinds": set(),
    "permit the conjunction and parenthesis kinds": {"conjunction", "parenthesis"},
}


def one(pattern: str) -> int:
    match = re.search(pattern, output, re.M)
    if not match:
        raise SystemExit(f"row not found: {pattern}")
    return int(match.group(1))


def by_class(prefix: str) -> dict:
    found = {}
    for kind in KINDS:
        match = re.search(rf"^{re.escape(prefix)} class {re.escape(kind)}: (\d+)$", output, re.M)
        found[kind] = int(match.group(1)) if match else 0
    return found


def fmt(value: int) -> str:
    return f"{value:,}"


def pct(numerator: int, denominator: int) -> str:
    return f"{numerator / denominator * 100:.2f}"


# region the rows: each class block, then each predicate's per-class rows
A = {}
JOINABLE = {}
FORCED = {}
for kind in KINDS:
    match = re.search(
        rf"^class {re.escape(kind)}: (\d+); clause at or over 120: (\d+); shipped join removes: (\d+)$",
        output,
        re.M,
    )
    A[kind], FORCED[kind], JOINABLE[kind] = (int(match.group(1)), int(match.group(2)), int(match.group(3)))

PREDICATES = {
    "A every break": (A, one(r"A findings \(quiet non-hard breaks at no boundary\): (\d+)"), one(r"A files: (\d+)")),
    "B every joinable break": (
        JOINABLE,
        one(r"B findings \(A findings the shipped join removes\): (\d+)"),
        one(r"B files: (\d+)"),
    ),
    "D unforced clause": (
        by_class("D (clause's first break)"),
        one(r"D findings \(quiet clauses under the limit\): (\d+)"),
        one(r"D files: (\d+)"),
    ),
    "F1 short next line": (
        by_class("F1"),
        one(r"F1 findings \(break before a one-or-two-word line\): (\d+)"),
        one(r"F1 files: (\d+)"),
    ),
    "F2 short line": (
        by_class("F2"),
        one(r"F2 findings \(one-or-two-word line at no boundary\): (\d+)"),
        one(r"F2 files: (\d+)"),
    ),
}
# The ordinary breaks a predicate reports, in the break unit, so that coverage is comparable across
# predicates even where a predicate reports one finding per clause.
ORDINARY_BREAKS_COVERED = {
    "A every break": A["ordinary"],
    "B every joinable break": JOINABLE["ordinary"],
    "D unforced clause": one(r"D \(every break of a reported clause\) class ordinary: (\d+)"),
    "F1 short next line": by_class("F1")["ordinary"],
    "F2 short line": by_class("F2")["ordinary"],
}
# How many of each predicate's findings the modeled join clears. For A and B the joinable row of each
# class is printed, so a permitted kind's joinable breaks can be taken out exactly. For the two shape
# predicates only the predicate's own joinable total is printed, so the figure is given at permit none
# and not under a permitted kind.
SHAPE_JOINABLE_TOTAL = {
    "F1 short next line": one(r"F1 of those the shipped join removes: (\d+)"),
    "F2 short line": one(r"F2 of those the shipped join removes: (\d+)"),
}
# endregion

print("## the sums that must hold")
for name, (rows, total, _files) in PREDICATES.items():
    print(f"{name}: per-class rows sum to {fmt(sum(rows.values()))}, the predicate's own row says {fmt(total)}")
print(
    "block-marker + conjunction + dash + inline-span + ordinary + parenthesis = "
    f"{fmt(sum(A.values()))}; A's row says {fmt(PREDICATES['A every break'][1])}"
)
print("the dash row is subtracted once under an answer that permits it, and it is counted in no other row: 20")
print()

print("## each kind alone: the breaks it removes from each predicate's findings")
print("kind" + "".join(f"{name.split()[0]:>12}" for name in PREDICATES))
for kind in KINDS:
    print(f"{kind:<20}" + "".join(f"{fmt(rows[kind]):>12}" for rows, _t, _f in PREDICATES.values()))
print(f"{'TOTAL of the six':<20}" + "".join(f"{fmt(sum(rows.values())):>12}" for rows, _t, _f in PREDICATES.values()))
print()

for answer, permitted in ANSWERS.items():
    print(f"## under: {answer}")
    print(f"the kinds permitted: {sorted(permitted) if permitted else 'none'}")
    for name, (rows, total, files) in PREDICATES.items():
        left = total - sum(rows[kind] for kind in permitted)
        ordinary = rows["ordinary"]
        line = (
            f"{name}: findings {fmt(left)} (was {fmt(total)}), files {fmt(files)} at permit none,"
            f" ordinary findings {fmt(ordinary)}, ordinary share {pct(ordinary, left)} percent,"
            f" ordinary breaks covered {fmt(ORDINARY_BREAKS_COVERED[name])}"
        )
        if name == "D unforced clause":
            line += ", no command clears a finding of it"
        elif name in SHAPE_JOINABLE_TOTAL:
            line += (
                f", cleared by the modeled join {fmt(SHAPE_JOINABLE_TOTAL[name])} of {fmt(total)},"
                " which is counted at permit none only"
            )
        else:
            cleared = sum(value for kind, value in JOINABLE.items() if kind not in permitted)
            line += f", cleared by the modeled join {fmt(cleared)} ({pct(cleared, left)} percent)"
        print(line)
    print()

print("## the order by ordinary share, under each answer")
for answer, permitted in ANSWERS.items():
    scored = []
    for name, (rows, total, _files) in PREDICATES.items():
        left = total - sum(rows[kind] for kind in permitted)
        scored.append((rows["ordinary"] / left * 100, name, rows["ordinary"], left))
    scored.sort(reverse=True)
    print(answer)
    for share, name, ordinary, left in scored:
        print(f"  {share:5.2f} percent  {name}  = {fmt(ordinary)} / {fmt(left)}")
    print()

print("## what the emphasis climb at the tree's tip can move")
# fc679b9392 makes the boundary test climb * _ and ~, so a break ending inside emphasis is no longer
# counted at no boundary. A, B and F2 have a durable row that takes it out; D and F1 have none.
print(f"quiet emphasis endings in the population: {fmt(one(r'  of the quiet, ending inside emphasis: (\d+)'))}")
print(f"emphasis endings the short-line predicate counts: {fmt(one(r'F2 of those that end inside emphasis, removable or not: (\d+)'))}")
for name, row in (
    ("A every break", r"A durable findings \(A minus the quiet emphasis ending\): (\d+)"),
    ("B every joinable break", r"B durable findings \(B minus the emphasis ending\): (\d+)"),
    ("F2 short line", r"F2 durable findings \(F2 minus the emphasis ending\): (\d+)"),
):
    left = one(row)
    ordinary = PREDICATES[name][0]["ordinary"]
    print(f"{name}: with the emphasis endings out, at most {pct(ordinary, left)} percent ({fmt(ordinary)} / {fmt(left)})")
print("D and F1 have no durable row, so neither of their shares can be moved here")
print()

print("## each scope's count, in each predicate's unit, at permit none")
families = {}
pattern = (
    r"^(doc \(top level\)|doc[^ ]*|package \(Markdown\)|package \(Rust doc comments\)|the rest)"
    r" A=(\d+) B=(\d+) D=(\d+) D2=(\d+) F1=(\d+) F2=(\d+)$"
)
for line in re.finditer(pattern, output, re.M):
    families[line.group(1)] = {
        "A": int(line.group(2)),
        "B": int(line.group(3)),
        "D": int(line.group(4)),
        "D2": int(line.group(5)),
        "F1": int(line.group(6)),
        "F2": int(line.group(7)),
    }
doc_names = [name for name in families if name.startswith("doc")]
print(f"families read: {len(families)} (doc families: {len(doc_names)})")
for predicate in ("A", "B", "D", "F1", "F2"):
    doc_total = sum(families[name][predicate] for name in doc_names)
    package = families["package (Markdown)"][predicate] + families["package (Rust doc comments)"][predicate]
    rest = families["the rest"][predicate]
    whole = doc_total + package + rest
    print(
        f"{predicate}: doc/ {fmt(doc_total)}, package/ {fmt(package)}"
        f" (Markdown {fmt(families['package (Markdown)'][predicate])},"
        f" Rust doc {fmt(families['package (Rust doc comments)'][predicate])}),"
        f" the rest {fmt(rest)}, whole tree {fmt(whole)}"
    )
print()

print("## the short-line predicate in the one package a repair route has run over")
print(
    f"G findings (F2 inside package/git-policy/cli): {fmt(one(r'G findings \(F2 inside package/git-policy/cli\): (\d+)'))}"
    f" in {fmt(one(r'G files: (\d+)'))} files"
)
print("no row of output.txt counts another predicate inside one package")
