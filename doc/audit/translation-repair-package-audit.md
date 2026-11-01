# Translation-repair package audit

Status:
IN PROGRESS,
opened 2026-08-26 as task `#236`.
Findings are appended as they are verified;
a finding listed here was re-read at the cited line by the main session,
never taken from a reviewer's report on trust.

Owner's instruction,
verbatim:
"Do an exhaustive audit of the package so far.
Focus on existing issues rather than branching into new ones for now,
until you're confident there are no big existing issues."

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Package audit: findings that cross the package](translation-repair-package-audit-findings-package-wide.md):
  The A findings,
  A-1 to A-6:
  the roster sent to a provider that cannot serve it,
  and what the README,
  the open decisions,
  the handover and the test names got wrong.
  Entries:
  A-1 to A-6.
- [Package audit: findings of the calibrate and probes slices](translation-repair-package-audit-findings-calibrate-and-probes-slices.md):
  The findings of the calibrate slice and of the probes slice,
  the instruments that measure the pipeline.
  Entries:
  calibrate-1 to calibrate-11,
  probes-1 to probes-6.
- [Package audit: findings of the provider slice](translation-repair-package-audit-findings-provider-slice.md):
  The findings of the provider slice:
  routing,
  clients,
  locks,
  meters and the catalog.
  Entries:
  provider-1 to provider-14.
- [Package audit: findings of the translate, repair and consolidate slices](translation-repair-package-audit-findings-translate-repair-and-consolidate-slices.md):
  The findings of the slices that hold the pipeline's three stages:
  translate,
  repair and consolidate.
  Entries:
  consolidate-1 to consolidate-2,
  repair-1 to repair-9,
  translate-1 to translate-6.
- [Package audit: findings of the document, slices, artifact and rendering slices](translation-repair-package-audit-findings-document-slices-artifact-and-rendering-slices.md):
  The findings of the slices that read the corpus and store or render what a run decides:
  document,
  slices,
  artifact and rendering.
  Entries:
  artifact-1,
  artifact-3,
  artifact-7,
  document-1 to document-3,
  document-5,
  rendering-1 to rendering-4,
  slices-1 to slices-4.

## Scope and method

Subject:
`package/module/translation-repair` (moved on 2026-10-18 to the packages under `package/translation-repair`) at
`b20120fbb`
(worktree `/var/home/user/worktrees/translation-repair`,
branch `translation-repair-rebased`).

The source is partitioned into ten slices,
every file in exactly one,
checked by script before the reading started:
`provider` (76 files,
20733 lines),
`probes` (63,
20031),
`document` (77,
19246),
`slices` (58,
18550),
`repair` (65,
17544),
`translate` (66,
16700),
`artifact` (47,
13786),
`calibrate` (33,
10042),
`consolidate` (28,
7408),
`rendering` (19,
6264).
File lists:
`~/temp/agent/audit-slices/<slice>.txt`.

Each slice is read in full by a forked reviewer under seven lenses,
in this priority order:
silent failure (the `#235` class),
wrong output,
resilience under provider failure,
corpus or key leakage,
contract drift between prose and code,
repo-rule violations the linters cannot see,
and test-coverage gaps.
Reviewer reports land in `~/temp/agent/audit-<slice>.md`.
The harness runs at most five reviewers at once,
so the second five slices start as the first five finish.

A reviewer's report is a lead.
Every BLOCKER and MAJOR is re-verified at the cited line before it enters the register;
"checked and found sound" sections are recorded as the reviewer's claim,
not as evidence.
Reviewers inherit this session's context,
so a report that omits the `#235` fallback,
the `RunConfigError` marker gap,
or the seat-silence gap in the slice that holds them is treated as a skim,
and that slice is re-read by the main session.

## Baseline, measured before anything was touched

- `mise run //package/module/translation-repair:lint:types` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:lint:types`):
  exit 0.
- `mise run //package/module/translation-repair:lint:oxlint` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:lint:oxlint`):
  0 warnings,
  0 errors,
  965 files,
  484 rules.
- `mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
  `//package/translation-repair/<package>:buildAndTest`):
  exit 0;
  750 suites carry the runner's `] PASS ` prefix,
  0 carry `] FAIL `
  (log `~/temp/agent/audit-buildAndTest-20260826T024842Z.log`).
- Build alone:
  2.0 s wall (`rolldown` 539 ms),
  which sets the cost of each GFP mutate-and-rebuild cycle.
- Source:
  532 files,
  150304 lines (tests and benches excluded).
  Tests:
  432 files,
  146083 lines.
  Benches: 0.
- Node:
  `v26.7.0` under mise,
  so the `import.meta.main` guard every CLI uses is live.

## Mechanical layers, swept over the whole package

Each sweep was run with `rg` over `src` excluding `*.test.ts`,
and each null result was checked
against a pattern known to match,
so a zero is a measured zero rather than a silent one.

- Provider request fields the owner forbade (`thinking`,
  `budget_tokens`,
  `reasoning_effort`):
  0 in code.
  The two textual matches are comments in `anthropic-request.ts:438` and `synthetic-client.ts:260`
  recording the prohibition.
- Inline debt markers (`TODO`,
  `FIXME`,
  `HACK`,
  `XXX`): 0.
  The single textual match is `U+XXXX` in a TSDoc line of `mask-invisible-lines.ts:157`.
- Lint suppressions:
  62 `oxlint-disable` lines,
  every one carrying a `--` justification;
  59 are `no-await-in-loop` (sequential awaits on purpose),
  2 `typescript/strict-void-return`,
  1 `no-restricted-syntax/no-regex`.
  `@ts-ignore` and `@ts-expect-error`: 0.
- Banned constructs:
  `switch`:
  0;
  `process.exit(`:
  0;
  `string | undefined`:
  0;
  non-null `!` assertions:
  0;
  `try...finally`:
  0 in code
  (three textual matches are comments explaining why a disposable is used instead).
- Raw `console.*` outside `src/corpus-run`:
  10,
  all inside TSDoc `@example` blocks.
- Quorum arithmetic:
  exactly one site,
  `stage-quorum.ts:167` (`Math.ceil(modelIds.length / 2)`),
  tested at `:219` and `:327` with `>=`.
- CLI refusal boundary:
  38 files call `reportingRefusals(` under `if (import.meta.main)`,
  matching the `#226` census;
  the four other files that mention `process.argv` are argument helpers with no `import.meta.main`.
- `package.json` declares no `bin`,
  so `AP4` (shebang) does not apply;
  every CLI runs through a mise task.
- Tests:
  0 files without `expect(`;
  0 `.skip`/`.only`/`todo`;
  4 test names carry an all-caps verdict word (see finding `A-6`);
  4 files hold more `it({` cases than `expect(` calls
  (`compare-candidates.unit.test.ts` 8 against 3,
  `checker-roster.unit.test.ts` 9 against 8,
  `wording-coherence.unit.test.ts` 5 against 4,
  `slice-coverage.unit.test.ts` 5 against 4),
  to be read rather than assumed to be gaps.

## How the test mass is covered

The 146083 test lines sit outside every slice,
and that exclusion is deliberate and named here
rather than silent.
Branch coverage of the implementation was measured by mutation in `#233` (73 branches,
19 gaps closed)
and `#234` (twenty depth-two rounds,
eight gaps closed),
reachability from the built bundle in `#208`,
`#209`,
`#231` and `#232`,
and throw assertions were rewritten to check classes in `#127`.
This audit reads a test file when a finding lands in the module it covers,
when a reviewer names a branch as untested,
or when the mechanical sweeps single it out (the four files listed under the mechanical layer).
A full test-side reading pass is not part of this audit unless a slice report shows the prior
measurements to be wrong,
in which case it is scheduled as its own item.

## Documentation and decisions layers

- `doc/planning/translation-repair-open-decisions.md` (2046 lines) is titled as still open,
  while `doc/decision/translation-repair-question-answers.md` records the owner's answers to
  questions 1 through 7 and the planning document itself marks question 8 answered.
  Questions 9 and 10 have decision records of their own
  (`translation-repair-straggler-grace.md`,
  `translation-repair-runaway-call-termination.md`).
  Drift,
  not a defect:
  a reader starting from the planning document is told to decide things
  already decided.
- `doc/handover/translation-repair.md` is 3440 lines against the 2000-line cap it sets for itself.
- `package/module/translation-repair/README.md:418-423` (moved on 2026-10-18 to `package/translation-repair/README.md`)
  documents the `#235` defect as a feature:
  the Charm Hyper key is "OPTIONAL,
  and its absence is not an error".
  Rewritten in the same commit as the `#235` fix.
- `doc/troubleshooting/translation-repair-unread-signals.md` still lists two signals nothing acts on:
  `alignment.findings` (recorded,
  reaches no stage) and the quote-anchoring findings.
  That document calls it a design question and does not settle it;
  this audit does not either,
  and records it so it is not rediscovered.
- `doc/audit/translation-repair-debt-layers.md` (2026-08-19) reported the same zero-debt result
  for layers four and five at 610 files;
  this audit's sweep at 532 source files agrees.

## Findings register

Tally as of 2026-08-26,
after all ten slices reported:
2 BLOCKER (A-1 fixed,
probes-1 open as `#247`),
21 MAJOR of which 1 fixed (A-2),
14 open as their own tasks (`#237` to `#246`,
`#248` to `#251`,
`#252` to `#257`),
and 6 folded into those tasks (slices-1 to slices-3,
artifact-1,
artifact-2,
rendering-2);
by the end of 2026-08-26 every MAJOR task,
`#237` to `#257`,
is fixed in place,
each marked FIXED under its entry,
and roughly seventy MINOR,
verified where cited and queued after the MAJORs.

Closing tally,
measured 2026-08-26 after the last landing (`8bffaba9b`):
every entry under this heading carries a
FIXED,
CLOSED,
folded or tracked marker;
counted by heading,
3 BLOCKER,
28 MAJOR and 35 MINOR (a grouped heading
such as `document-5 to document-12` counts once).
The measurement found three entries without a marker,
all fixed
in fact and recorded since:
`provider-1` (with A-1),
`provider-14` (CLOSED by the MAJOR landings) and `calibrate-1`,
whose pointer half was the one piece still owed and landed in `8bffaba9b`.
Whole suite after `50ffffc07`:
814 PASS,
0 FAIL,
exit 0.
Guards recorded as owed inside FIXED blocks (repair-8's typography half,
consolidate-5,
probes-10
and probes-11,
calibrate-5 and calibrate-8,
repair-2,
document-11,
slices-5 and slices-8 as type-checked only) are
not defects;
they are the places where the landing is proven by the type check or by measurement rather than by a
mutation,
and each says so under its entry.

Severity:
BLOCKER is wrong or missing output,
silent failure,
data loss,
or corpus/key leakage;
MAJOR is a contract violation or resilience gap likely to bite in a real run;
MINOR is a rule violation or coverage gap with no output effect.
IDs prefixed `A-` come from the main session's own reading;
IDs prefixed with a slice name come from that slice's reviewer and were re-verified.

Filed in:

- [Package audit: findings that cross the package](translation-repair-package-audit-findings-package-wide.md)
- [Package audit: findings of the calibrate and probes slices](translation-repair-package-audit-findings-calibrate-and-probes-slices.md)
- [Package audit: findings of the provider slice](translation-repair-package-audit-findings-provider-slice.md)
- [Package audit: findings of the translate, repair and consolidate slices](translation-repair-package-audit-findings-translate-repair-and-consolidate-slices.md)
- [Package audit: findings of the document, slices, artifact and rendering slices](translation-repair-package-audit-findings-document-slices-artifact-and-rendering-slices.md)

## Slice reports

Each entry records the reviewer's coverage claim,
then what the main session verified.

- `provider`:
  reviewer read 76 of 76 files (20733 lines),
  none unread;
  reported 1 BLOCKER,
  5 MAJOR,
  8 MINOR;
  all fourteen re-verified at the cited lines and recorded;
  it named the `#235` fallback,
  the missing
  catalog check,
  and the `RunConfigError` gap unprompted.
- `translate`:
  reviewer read 66 of 66 files (16700 lines),
  none unread;
  reported 2 MAJOR,
  4 MINOR,
  all six re-verified and recorded.
  Answered calibrate's question:
  `producer-calibrate.ts:120` also builds
  a client per slice.
- `repair`:
  reviewer read 65 of 65 files (17544 lines),
  none unread;
  reported 3 MAJOR,
  6 MINOR;
  eight re-verified as reported,
  one (repair-2) downgraded after the abort path was traced through
  `stage-round.ts`,
  `stage-call.ts`,
  and `stream-drain.ts`.
- `probes`:
  reviewer read 63 of 63 files (20031 lines),
  none unread;
  reported 1 BLOCKER,
  4 MAJOR,
  8 MINOR;
  the BLOCKER and every MAJOR re-verified at the cited lines;
  MINORs spot-verified.
- `calibrate`:
  reviewer read 33 of 33 files (10042 lines),
  none unread;
  reported 2 MAJOR and 9 MINOR,
  all eleven re-verified at the cited lines and recorded;
  the reviewer named the `#235` fallback and the `RunConfigError` gap unprompted
  and corrected the silence premise,
  which is the opposite of a skim.
  Its open questions,
  handed to other slices:
  whether `candidate-select.ts:347` `judgesAvailable` counts seated or reachable judges (`consolidate`),
  whether `producer-calibrate.ts` also builds a client per slice (`translate`),
  and whether any consumer prints the `reason` field `pass-schema-census.ts` stores from a `SyntaxError`.
- `consolidate`:
  reviewer read 28 of 28 files (7408 lines),
  none unread;
  reported 1 MAJOR,
  9 MINOR;
  the MAJOR re-verified;
  it answered calibrate's `judgesAvailable` question (seated roster).
- `document`:
  reviewer read 77 of 77 files (19246 lines),
  none unread;
  reported 3 MAJOR,
  9 MINOR;
  the three MAJORs re-verified at the cited lines,
  the population note in `markdown-blocks.ts` included;
  MINORs spot-verified.
  Its open questions go to the fix queue:
  whether `screenNonTranslationVotes` findings joined into `l.warn` can quote text (`#237`),
  whether `reindexSlicePair` restamps both sides of an insertion pair,
  whether `#232`'s reachability measure counted test-only reach for `group-source-first.ts` and `reflow-orphans.ts`,
  and whether the reading cache's generation marker is the pipeline digest (`#253`).
- `artifact`:
  reviewer read 47 of 47 files,
  none unread;
  reported 3 MAJOR and 4 MINOR
  (artifact-1 filed MAJOR by the reviewer's own call,
  BLOCKER by the brief's letter);
  all three MAJORs re-verified at the cited lines;
  it answered both extra questions:
  no per-stage quorum evidence in the owned artifact shape,
  and the round-three draw is legacy artifacts the two-lane parser refuses.
- `slices`:
  reviewer read 58 of 58 files,
  none unread;
  reported 3 MAJOR,
  6 MINOR;
  all three MAJORs re-verified at the cited lines;
  none needed a new task,
  they fold into `#238`,
  `#246`,
  `#249`.
  Its open questions:
  whether `PreparationIdentity` is a branded primitive
  (`lane-comparison.ts:403` compares with identity),
  whether a full-pass ledger reaches the open-file limit
  under one `Promise.all`,
  and whether an entry id may appear in a boundary refusal at all.
- `rendering`:
  reviewer read 32 of 32 files (19 source,
  13 tests),
  none unread;
  reported 3 MAJOR,
  9 MINOR;
  all three MAJORs re-verified at the cited lines.
  Its open questions:
  the population share of `verification=refused` (a kind-field count,
  `#255`),
  and whether the writer-side mismatch errors that `translating()` wraps can quote slice text (`#237`).

## What this audit does not cover

- Live behaviour against the providers.
  No model call is made by the audit;
  the `#235` fix will be verified live at the user boundary as its own step.
- The corpus itself,
  and the quality of shipped translations.
  Those are measured by the instruments in the `probes` and `rendering` slices,
  whose honesty is what the audit checks.
