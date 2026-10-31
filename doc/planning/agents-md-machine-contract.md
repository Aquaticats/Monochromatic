# AGENTS.md machine-contract proposals from the 2026-10-31 measurement

## Why

The 2026-10-31 machine-parsability audit
(`doc/audit/agents-md-machine-parsability-2026-10-31.md`)
found three gaps between what `AGENTS.md` claims a machine can rely on
and what any machine can actually read or check today.
Each proposal tightens an existing rule or splits one where `RLM` forces it,
and carries the measurement that produced it.
Nothing here edits `AGENTS.md` before the user accepts.
New code minting follows `NCD`:
both forbidden-strings appendixes were checked
and no reserved or existing code collides.

## Proposal 1: RLM pins its counting and its boundary

Spotted failure mode:
the character half of `RLM` is already broken by 14 rules
(over 200 characters after whitespace normalization,
maximum 277 at `RXI`),
while the counting convention exists only as recorded practice
(`doc/planning/agents-md-gap-proposals.md` measures a proposal at
"39 words and 279 characters",
which reproduces only with the code prefix included).
The boundary is ambiguous the same way:
`QRY`,
`SGD`,
`WR2` sit at exactly 200 characters,
and "under 50 words and 200 characters" read strictly flags them too.
A checker
(`gh issue view 566`)
would have to guess both points.

Proposed wording,
replacing the current `RLM` text
(26 words and 171 characters):

```text
RLM:
 Each tagged rule stays under 50 words and at most 200 characters,
 code prefix included,
 after whitespace normalization;
 split longer guidance into fresh tagged rules.
```

The boundary is a flagged choice.
The wording above ("at most 200 characters") keeps `QRY`,
`SGD`,
`WR2` compliant and matches how the limit is discussed elsewhere
("over the 200 character limit").
The strict reading ("under 50 words and 200 characters",
24 words and 163 characters) flags those 3 rules for splitting instead.

Evidence:
measured 2026-10-31 with `~/temp/agent/agents-md-audit.ts`:
14 rules over 200 characters,
3 exactly at 200,
35 between 191 and 199,
so the limit is live guidance that drifts between hand-repair episodes
(`VRB` and `CPN` were split down after the 2026-10-20 triage named them).

Risks:
wording churn in one rule;
none for behaviour,
since the counting it pins is the convention already in use.

## Proposal 2: GRM marks definitions and quotes apart

Spotted failure mode:
`TAG` promises codes unique across `AGENTS.md`,
skills,
and package docs,
but no machine can run that check
because definitions and quotations look identical.
Every one of the 232 codes also opens a line in generated `CLAUDE.md`,
203 open lines in `doc/planning/agents-md-optimization.md`,
and 10 further docs quote 1 to 8 each,
so a naive scan reports 232 false collisions.
The same measurement found the rule-block grammar unstated:
a rule runs to the next blank line,
and its continuation lines open at column 0 or one space with no recoverable semantics
(63 column-0 lines among 1005 indented ones,
and `monochromatic-lint` itself inserts column-0 lines into rule blocks
because `continuation_prefix` takes the paragraph's opening-line prefix).

Proposed wording,
a fresh rule `GRM` placed after `TAG`
(36 words and 187 characters):

```text
GRM:
 A rule is its `CODE:` line plus continuation lines to the next blank line,
 which may open at column 0 or one space;
 rule text quoted in hand-written docs keeps its code off column 0.
```

Code choice:
`GRM` reads as "grammar" and collides with nothing
(232 `AGENTS.md` codes,
28 skill codes,
both forbidden-strings appendixes).
Alternatives considered and rejected:
`SYN` (first reads as TCP SYN or syllabus),
`RBG` (first reads as initials),
`RLE` (first reads as run-length encoding).

Evidence:
quote surface and indent split measured 2026-10-31;
probe `~/temp/agent/probe-2026-10-31/` shows the linter inserting
`default 2) instead.` at column 0 into the `NLG` rule block.
`TAG` itself is 189 characters,
so the clause has no room there and `RLM` sends it to a fresh rule.

Risks:
hand-written docs quoting rule text need their quote lines indented
(about 220 lines across 12 docs,
mechanical),
and the checker must exclude generated mirrors by construction
(`file-enforcer` owns `CLAUDE.md`).
The lower-ranked alternative is a checker allowlist of known quote sites,
which avoids the churn but rots silently as new docs quote rules.

## Proposal 3: repair the gate finding at AGENTS.md:673

Spotted failure mode:
`mise run lint:monochromatic-lint -- AGENTS.md CLAUDE.md` exits 1 today
with one `markdown/semantic-line-breaks` finding and its mirror
(`AGENTS.md:673:41`,
`CLAUDE.md:722:41`,
"A line break belongs here,
after a prose break-point character.").
The site is the comma after `--cpus` inside its parentheses in `NLG`.

Proposed repair:
take the linter's own fix
(a line break after the comma),
then regenerate `CLAUDE.md` through `file-enforcer` and commit both.

Evidence:
task run 2026-10-31,
exit 1 with exactly these two findings.

Risks:
the inserted line opens at column 0
(the linter's prefix model),
which `GRM` from proposal 2 declares meaningless;
without `GRM` the repair deepens the indent inconsistency instead.
