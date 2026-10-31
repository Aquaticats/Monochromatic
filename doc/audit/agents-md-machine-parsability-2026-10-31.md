# AGENTS.md machine parsability: measured 2026-10-31

## Why

The user asked how machine-parsable `AGENTS.md` is.
This records the measurements,
the enforcement that exists today,
and the limits of what was checked.

## Verdict

The rule layer parses perfectly and is verified by nothing.
A small state machine extracts all 232 rules with zero ambiguity,
and the self-stated structural invariants hold under measurement
(`TAG` in full,
`SLF` in full,
the word half of `RLM`).
The character half of `RLM` drifts unenforced
(14 rules over 200 characters today),
the cross-file code namespace has no machine-readable definition/quote boundary,
and no tool checks any of this
(`gh issue view 566` is still open).
The one machine gate that reads the file
(`monochromatic-lint`'s markdown rules)
currently fails on it at one line.

## Method

Script:
`~/temp/agent/agents-md-audit.ts`,
run 2026-10-31 over `AGENTS.md` at 1627 lines and 37506 bytes.
A rule is read as one `[A-Z0-9]{3}:` line at column 0
plus its continuation lines up to the next blank line or heading.
Counts cross-checked with `rg -c '^[A-Z0-9]{3}:$' AGENTS.md`
(232)
and `rg -o '^[A-Z0-9]{3}:$' AGENTS.md | sort | uniq -d`
(empty).
Character counts include the code prefix
and follow the recorded convention:
`doc/planning/agents-md-gap-proposals.md` measures one proposed rule text at
"39 words and 279 characters after whitespace normalization",
which reproduces exactly with the prefix included.

## Rule segmentation: clean

232 rules,
232 unique codes,
no duplicates.
Every rule starts with a `CODE:` line alone at column 0
(232 of 232),
and the file contains no other column-0 non-heading prose:
no code fences,
no lists,
no tables.
Each rule maps to one heading path of at most 4 levels,
41 paths in total,
matching the moment-of-decision sections `ORG` promises.
No heading or the title carries a tag.

Residual parse hazard:
a continuation line that itself matches `^[A-Z0-9]{3}:$` would falsely split a rule.
None exists today,
and a checker should reject such a line.

## Self-stated invariants: measured

`TAG` holds in file:
232 unique codes.
Cross-file uniqueness is a separate finding below.

`SLF` holds mechanically.
Zero rule bodies cite another rule's code.
The only three-character tokens repeating in the file are ordinary acronyms
(`CLI`,
`API`,
`SSD`,
`PID`,
`MCP`,
`CPU`).
26 doc path pointers inside rules all resolve.
The 14 path-shaped code spans that do not resolve are command and template strings
(`mise run //package/<path>:<task>`,
`doc/<family>/<kebab-topic>.md`).

`RLM` word half holds everywhere:
median 22 words,
maximum 38,
limit 50.
Character half drifts:
median 160 characters,
maximum 277,
14 rules over 200,
3 rules exactly at 200
(`QRY`,
`SGD`,
`WR2`),
35 more between 191 and 199.
The oversized 14 as `code characters path:line`:

```text
RXI 277 AGENTS.md:657
NQH 204 AGENTS.md:681
DRR 204 AGENTS.md:795
WKP 203 AGENTS.md:68
XIC 203 AGENTS.md:230
CB1 203 AGENTS.md:417
CM3 203 AGENTS.md:752
GCG 203 AGENTS.md:1538
GAP 202 AGENTS.md:136
WC2 202 AGENTS.md:873
JCH 202 AGENTS.md:1064
NXR 201 AGENTS.md:526
NLG 201 AGENTS.md:667
THR 201 AGENTS.md:1363
```

The 2026-10-20 triage named `VRB` at 222 and `CPN` at 240
(`doc/audit/issue-triage-2026-10-20.evidence.jsonl`).
Both were since split down to 185 and 177 by hand,
so the file is repaired in drift episodes
and nothing holds it between them.

Markdown hygiene
(`MD1` through `MD8`,
`WR2`,
`WR3`):
zero lines over 120 characters,
zero tables,
zero em or en dashes,
zero italics,
no heading deeper than 4.

## The line-structure layer

Semantic line breaks at every prose break point
(`MD1`) are machine-mandated:
`monochromatic-lint` runs `markdown/semantic-line-breaks` at error severity
(`monochromatic-lint.config.jsonc:184`).
`mise run lint:monochromatic-lint -- AGENTS.md CLAUDE.md` reports exactly one finding and its mirror:
`AGENTS.md:673:41`
("A line break belongs here,
after a prose break-point character.")
and `CLAUDE.md:722:41`,
task exit 1.
The site is the comma after `--cpus` inside its parentheses in `NLG`.

The indent channel inside a rule is neither machine-mandated nor specified.
1068 body lines split as 1005 opening with one space
and 63 opening at column 0.
A probe
(`~/temp/agent/probe-2026-10-31/`)
shows the mechanism:
`continuation_prefix`
(`package/linter/monochromatic-lint/src/markdown_prose_context.rs:92`)
takes the paragraph's opening-line prefix,
so the linter's own repair for `AGENTS.md:673`
inserts a column-0 continuation line
(`default 2) instead.`)
into a rule block whose hand-wrapped lines open with one space.
No semantic reading of the 63/1005 split survives its counterexamples,
so machine readers must join rule text with whitespace normalization,
which `RLM` already mandates.

## The cross-file namespace

`TAG` promises codes unique across `AGENTS.md`,
skills,
and package docs.
Measured:
`AGENTS.md` holds 232 codes
and `.agents/skills/*/SKILL.md` holds 28 more
(`android-emulator` 6,
`visual-design-review` 20,
`project-code-review` `NIT`,
`choosing-technology` `RCO`),
with zero collisions today.
Screening note:
`NIT` is an ordinary word,
which `NCD` tells new codes to reject.

The definition/quote boundary is not machine-readable.
Every one of the 232 codes also opens a line in generated `CLAUDE.md`,
203 open lines in `doc/planning/agents-md-optimization.md`,
and 10 further docs quote 1 to 8 each.
A naive uniqueness scan therefore reports 232 false collisions,
and `gh issue view 566` has no scope rule to implement.

The one enforcement that exists:
`forbidden-strings.append.txt` reserves rejected codes as `^CODE:` labels
(`ncd-reserved-shortcodes` at line 19,
`ncd-reserved-shortcode-families` at line 121),
so the commit gate rejects reserved codes.
Zero current codes collide with the reservation.

## Consumers of the contract

- `mise.toml:282` review task hands `AGENTS.md` to a model
  and demands findings cite "the tag code of the standard it violates".
- `package/claude-code-plugin/source/src/handler/correction-reminder.ts:158` hardcodes "AGENTS.md rule CKB",
  so codes are handles other tools string-match.
- `file-enforcer.config.ts:2098` generates `CLAUDE.md` as `AGENTS.md` verbatim plus a preamble.
- `file-enforcer.config.ts:1442` scopes a Harper server to `AGENTS.md` and `CLAUDE.md` prose.
- `package/agent-harness-shared/advisor/src/project-context.ts:13` loads `/project/AGENTS.md` into advisor context.

No programmatic checker parses codes,
rule lengths,
or cross-file uniqueness.
`gh issue view 566`
("Add a checker for AGENTS.md rule tags and length limit",
open,
label `ready-for-agent`)
is exactly this gap.

## Proposals

`doc/planning/agents-md-machine-contract.md` carries three GAP proposals
in the format of `doc/planning/agents-md-gap-proposals.md`:
repair the gate finding,
pin `RLM` counting and boundary,
and mark quoted rule text off column 0.
Nothing here edits `AGENTS.md` before the user accepts.

## Limits and unknowns

- The parse script is scratch
  (`~/temp/agent/agents-md-audit.ts`),
  written for this measurement and not committed.
- `SLF` was measured only for code-token citations.
  Leaning on terms only another rule defines is a semantic property no check covered.
- `NCD` semi-meaningfulness is judgment rather than measurement.
  Only the forbidden-strings reservation list was checked.
- Whether a CI job runs `lint:monochromatic-lint` over `AGENTS.md` on every change was not verified.
  The surviving finding at `AGENTS.md:673` suggests it does not gate every commit.
- Quote-site inventory counts `CODE:` lines only.
  Prose citations like "rule CKB" were not counted.

## Commits and next action

This audit lands as `docs(audit)`.
Next action is the user's adoption decision on `doc/planning/agents-md-machine-contract.md`.
After acceptance the landing edits are `AGENTS.md`,
file-enforcer `CLAUDE.md` regeneration,
and the quote-site indents.
