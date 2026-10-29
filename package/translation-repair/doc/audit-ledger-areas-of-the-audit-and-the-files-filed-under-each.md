# Audit ledger: the areas of the audit and the files filed under each

The whole-package audit worked in areas:
the run it stopped,
the test suite,
the repair lane,
the docs and comments against the code,
what was found while fixing,
the recurring code families
and the audit's own process mistakes.
Each area names the probes it used and lists the files its findings are filed in.
The T8 entry (code no unit test runs) stays here with its census method and its open counts.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Runs

TianqiChen66620 (`.frozen-dist-2f26f440d`) was stopped by this session at 17 min,
on the owner's 2026-09-27 instruction to launch only once no further fix is due;
slice 9 was not reached.

## Test suite

Probes:
`~/temp/agent/audit-tests/`
(a runtime harness logging every `expect` that runs,
`scan-corpus.mjs` and `classify-hits.mjs` for corpus text,
`run-container.mjs` for load runs,
`magic-*.tsv`,
`untested-dist-functions.tsv`,
`coverage-holes.txt`).

Filed in:

- [Audit ledger: defects in the tests themselves](audit-ledger-test-suite-defects-in-the-tests-themselves.md)
- [Audit ledger: code no unit test runs, method, first censuses and early batches](audit-ledger-test-suite-unrun-code-method-first-censuses-and-early-batches.md)
- [Audit ledger: code no unit test runs, stretch queue batches one to ten](audit-ledger-test-suite-unrun-code-stretch-queue-batches-one-to-ten.md)
- [Audit ledger: code no unit test runs, stretch queue batches eleven to sixteen](audit-ledger-test-suite-unrun-code-stretch-queue-batches-eleven-to-sixteen.md)
- [Audit ledger: code no unit test runs, stretch queue batches seventeen to twenty-four](audit-ledger-test-suite-unrun-code-stretch-queue-batches-seventeen-to-twenty-four.md)
- [Audit ledger: code no unit test runs, agent clusters and bundles before the trial](audit-ledger-test-suite-unrun-code-agent-clusters-and-bundles-before-the-trial.md)
- [Audit ledger: code no unit test runs, trial model clusters from `group` to `coverage`](audit-ledger-test-suite-unrun-code-trial-clusters-group-through-coverage.md)
- [Audit ledger: code no unit test runs, trial model clusters from `markup` to `corpus-run/editor`](audit-ledger-test-suite-unrun-code-trial-clusters-markup-through-editor.md)
- [Audit ledger: code no unit test runs, trial model clusters from `corpus-run/roster` to `corpus-run/publish`](audit-ledger-test-suite-unrun-code-trial-clusters-roster-through-publish.md)
- [Audit ledger: code no unit test runs, re-verification of 2026-10-05](audit-ledger-test-suite-unrun-code-reverification-of-2026-10-05.md)
- [Audit ledger: code no unit test runs, census at `65bd3a645` and the runner entry files](audit-ledger-test-suite-unrun-code-census-at-65bd3a645-and-runner-entry-files.md)
- [Audit ledger: code no unit test runs, final census and what stays cold](audit-ledger-test-suite-unrun-code-final-census-and-what-stays-cold.md)

### T8: code no unit test runs

Status:
open,
and the next launch waits for it:
under the owner's standing directive of 2026-09-28 to prefer the quality of the end result,
the launch follows the tests this entry asks for,
and the owner may veto that order and launch first.
Remeasured 2026-09-29 by block coverage mapped to source lines,
which supersedes the 2026-09-28 figures;
first recounted 2026-09-27 at 79 of 1,178 barrel-exported functions
(`~/temp/agent/audit-repair/untested-exports.mjs`,
list in `untested-exports.json`),
queued after the findings that change a run's output.
76 public functions named by no test;
a coverage sample shows `isPaymentRefusal`,
`statedWaitMsOf`,
`routedJson`,
`secondOpinionsFrom` and others never called,
and the decision reply's refusal branches never exercised.
Recounted 2026-09-28 with the same script:
94 of 1,248,
the barrels having grown;
`reseatHookFor` (X12) is among them though every hook builder's test runs it.
The count has two blind spots,
so it is neither a floor nor a ceiling of untested code:
it reads `export function` declarations only (never `export const`),
and it counts a name mentioned anywhere in a test as tested,
while a function called only through another is uncounted.
The work waits on a measurement of execution rather than names:
the unit suite under `NODE_V8_COVERAGE`,
with function entries that never ran intersected with the barrel exports,
then a case per branch of each function on that list (TCV).

Filed in:

- [Audit ledger: code no unit test runs, method, first censuses and early batches](audit-ledger-test-suite-unrun-code-method-first-censuses-and-early-batches.md)
- [Audit ledger: code no unit test runs, stretch queue batches one to ten](audit-ledger-test-suite-unrun-code-stretch-queue-batches-one-to-ten.md)
- [Audit ledger: code no unit test runs, stretch queue batches eleven to sixteen](audit-ledger-test-suite-unrun-code-stretch-queue-batches-eleven-to-sixteen.md)
- [Audit ledger: code no unit test runs, stretch queue batches seventeen to twenty-four](audit-ledger-test-suite-unrun-code-stretch-queue-batches-seventeen-to-twenty-four.md)
- [Audit ledger: code no unit test runs, agent clusters and bundles before the trial](audit-ledger-test-suite-unrun-code-agent-clusters-and-bundles-before-the-trial.md)
- [Audit ledger: code no unit test runs, trial model clusters from `group` to `coverage`](audit-ledger-test-suite-unrun-code-trial-clusters-group-through-coverage.md)
- [Audit ledger: code no unit test runs, trial model clusters from `markup` to `corpus-run/editor`](audit-ledger-test-suite-unrun-code-trial-clusters-markup-through-editor.md)
- [Audit ledger: code no unit test runs, trial model clusters from `corpus-run/roster` to `corpus-run/publish`](audit-ledger-test-suite-unrun-code-trial-clusters-roster-through-publish.md)
- [Audit ledger: code no unit test runs, re-verification of 2026-10-05](audit-ledger-test-suite-unrun-code-reverification-of-2026-10-05.md)
- [Audit ledger: code no unit test runs, census at `65bd3a645` and the runner entry files](audit-ledger-test-suite-unrun-code-census-at-65bd3a645-and-runner-entry-files.md)
- [Audit ledger: code no unit test runs, final census and what stays cold](audit-ledger-test-suite-unrun-code-final-census-and-what-stays-cold.md)

## Repair lane

Probes and notes:
`~/temp/agent/audit-repair/notes.md`
(`seat-probe.mjs`,
`regress-check.mjs`,
`dispute-standin.mjs`,
`ride-along.mjs`,
`preservation-vacuous.mjs`).

Filed in:

- [Audit ledger: the repair lane](audit-ledger-repair-lane.md)
- [Audit ledger: the repair lane's smaller items](audit-ledger-repair-lane-smaller-items.md)

## Docs and comments against code

Probe scripts:
`~/temp/agent/audit-docs2/`
(`backticks.mjs`,
`repo-wide.mjs`,
`examples.mjs`,
`owner-quotes.mjs`,
`hygiene.mjs`,
`class-dates.mjs`,
`pairing.mjs`).
Roster values were computed from `src/corpus-run/run-config.ts` (moved on 2026-10-18 to
`package/translation-repair/run/src/corpus-run/run-config.ts`) itself:
`RUN_ROSTER` 9 seats,
`RUN_READER_MODELS` 6,
7 wide seats,
8 late judges.
Paths in this section are package-relative.

Filed in:

- [Audit ledger: docs and comments that disagreed with the code](audit-ledger-docs-against-code.md)
- [Audit ledger: doc conventions and their guards](audit-ledger-doc-conventions-and-their-guards.md)
- [Audit ledger: the Markdown linter's gate and the split of long docs](audit-ledger-markdown-linter-gate-and-the-split-of-long-docs.md)

## Found while fixing

Filed in:

- [Audit ledger: found while fixing, stages, seats and pairing keys](audit-ledger-found-while-fixing-stages-seats-and-pairing-keys.md)
- [Audit ledger: found while fixing, tests, source text and live defaults](audit-ledger-found-while-fixing-tests-source-text-and-live-defaults.md)
- [Audit ledger: found while fixing, names, headings and math on the page](audit-ledger-found-while-fixing-page-names-headings-and-math.md)

## Recurring code families

Audit area six:
code kept in more than one place,
which can drift until two parts of the pipeline
read one page two ways.
`~/temp/agent/audit-glossary-fix/duplicate-bodies.mjs` parses every non-test source file with rolldown's parser
and groups function bodies with comments and whitespace removed;
on 2026-09-28 it found 31 groups of 80 normalized characters or more spanning two or more files.
`duplicate-triage.mjs` and `duplicate-texts.mjs` print each copy's full text and the module-level names it reads,
so a copy whose body matches but whose constants differ shows up as such.

The rule applied to each group:
merge when both copies are live code that must agree;
keep a copy that is an artifact generation's frozen rule,
which the reader recomputes and refuses to disagree with,
and say so where it stands.
`artifact-two-lane-comparison.ts` states this for the lane verdict (`judgeTwoLaneSlice` beside `judgeSlice`).
A reader of one artifact version never imports from a live stage;
two readers of the same version may share.

Filed in:

- [Audit ledger: copies of shared code in production](audit-ledger-copies-in-production-code.md)
- [Audit ledger: code nothing reaches](audit-ledger-unreferenced-code.md)
- [Audit ledger: comments, TSDoc and source text](audit-ledger-comments-tsdoc-and-source-text.md)
- [Audit ledger: text by code point, and words inside words](audit-ledger-code-points-and-word-boundaries.md)
- [Audit ledger: folds and quote forms](audit-ledger-folds-and-quote-forms.md)
- [Audit ledger: composed operations and checks on a part](audit-ledger-composed-operations-and-partial-checks.md)
- [Audit ledger: which seats a quorum counts](audit-ledger-which-seats-a-quorum-counts.md)
- [Audit ledger: short benches and the last live seat](audit-ledger-short-benches-and-the-last-live-seat.md)
- [Audit ledger: short bench floors and the waits that cross them](audit-ledger-short-bench-floors-and-the-waits-that-cross-them.md)
- [Audit ledger: futile returns replayed and the memory of skipped slices removed](audit-ledger-futile-returns-replayed-and-the-skipped-slice-memory-removed.md)
- [Audit ledger: holds a test registers and the seat reading sleeps](audit-ledger-holds-a-test-registers-and-the-seat-reading-sleeps.md)
- [Audit ledger: the every-provider-dry refusal, what it stops and what it says](audit-ledger-the-every-provider-dry-refusal.md)
- [Audit ledger: a stage that heard nobody, recorded as data](audit-ledger-silent-stages-recorded-as-data.md)
- [Audit ledger: what each sheet shows](audit-ledger-what-each-sheet-shows.md)
- [Audit ledger: the polish gate's claims and the order of reason and vote](audit-ledger-polish-gate-claims-and-reason-order.md)
- [Audit ledger: defaults that stand in for an input](audit-ledger-defaults-standing-in-for-an-input.md)
- [Audit ledger: test cases that cannot fail](audit-ledger-test-cases-that-cannot-fail.md)
- [Audit ledger: survivors of the guard demonstrations and the tests that pin them](audit-ledger-guard-demonstration-survivors-and-their-pins.md)
- [Audit ledger: caught text carried into messages](audit-ledger-caught-text-carried-into-messages.md)
- [Audit ledger: values read back from text](audit-ledger-values-read-back-from-text.md)
- [Audit ledger: labels and counts in messages](audit-ledger-labels-and-counts-in-messages.md)
- [Audit ledger: tags, nesting and paired marks](audit-ledger-tags-nesting-and-marks.md)
- [Audit ledger: outcome rules and stage records](audit-ledger-outcome-rules-and-stage-records.md)
- [Audit ledger: text that shows nothing](audit-ledger-text-that-shows-nothing.md)
- [Audit ledger: blocks and quotes read off the parse](audit-ledger-blocks-and-quotes-read-off-the-parse.md)
- [Audit ledger: references to titles](audit-ledger-title-references.md)
- [Audit ledger: the source scans and the coverage census](audit-ledger-source-scans-and-coverage-census.md)
- [Audit ledger: readers that accept what no writer writes](audit-ledger-readers-accept-shapes-writers-never-write.md)
- [Audit ledger: refusals that name too little](audit-ledger-refusals-that-name-too-little.md)
- [Audit ledger: loops that copy what they built](audit-ledger-loops-that-copy-what-they-built.md)
- [Audit ledger: command lines read by each runner](audit-ledger-command-lines-read-by-each-runner.md)
- [Audit ledger: clocks and durations](audit-ledger-clocks-and-durations.md)
- [Audit ledger: tests touching the real world](audit-ledger-tests-touching-the-real-world.md)
- [Audit ledger: positions compared across texts](audit-ledger-positions-compared-across-texts.md)
- [Audit ledger: provider streams, usage counts and retries](audit-ledger-provider-streams-usage-and-retries.md)
- [Audit ledger: NeuralWatt replies cut at the default effort and the picture energy estimate](audit-ledger-neuralwatt-replies-cut-at-the-default-effort-and-the-energy-estimate.md)
- [Audit ledger: copies of shared code in tests and scans](audit-ledger-copies-in-tests-and-scans.md)
- [Audit ledger: files a later run reads back](audit-ledger-files-a-later-run-reads-back.md)
- [Audit ledger: footnote markers and link destinations](audit-ledger-footnote-markers-and-link-destinations.md)
- [Audit ledger: repeated wording and ids on a page](audit-ledger-repeated-wording-and-ids-within-one-text.md)
- [Audit ledger: block pairings that do not fit their text](audit-ledger-block-pairings-that-do-not-fit-their-text.md)
- [Audit ledger: branches no input reaches, kept as quiet answers](audit-ledger-unreachable-branches-kept-as-quiet-answers.md)
- [Audit ledger: lists that must name every member of a type](audit-ledger-lists-that-must-name-every-member-of-a-type.md)
- [Audit ledger: repeated keys in stored records and rows](audit-ledger-repeated-keys-in-stored-records-and-rows.md)
- [Audit ledger: operator refusals printed as faults](audit-ledger-operator-refusals-printed-as-faults.md)
- [Audit ledger: output decided by arrival order](audit-ledger-output-decided-by-arrival-order.md)
- [Audit ledger: credentials masked at the transport](audit-ledger-credentials-masked-at-the-transport.md)
- [Audit ledger: runner reports and probes](audit-ledger-runner-reports-and-probes.md)
- [Audit ledger: the pass's lifecycle line stamps (B961 to B964)](audit-ledger-lifecycle-line-stamps-round-two.md)
- [Audit ledger: the cap census and the Xiaomi cap](audit-ledger-the-cap-census-and-the-xiaomi-cap.md)
- [Audit ledger: NeuralWatt's day start is never waited for](audit-ledger-neuralwatt-day-start-never-waited-for.md)
- [Audit ledger: the environment handed to child processes](audit-ledger-child-process-environment.md)
- [Audit ledger: failures under a caller's stop](audit-ledger-failures-under-a-callers-stop.md)
- [Audit ledger: work merged from parallel briefs](audit-ledger-work-merged-from-parallel-briefs.md)
- [Audit ledger: ambient state touched by library code](audit-ledger-ambient-state-touched-by-library-code.md)
- [Audit ledger: building the tense check](audit-ledger-tense-check.md)
- [Audit ledger: building the consistency pass, repeat groups and the carry](audit-ledger-consistency-pass-repeat-groups-and-the-carry.md)
- [Audit ledger: the consistency pass, the span and the marks beside it](audit-ledger-consistency-pass-span-and-marks-beside-it.md)
- [Audit ledger: the consistency pass, apostrophes and block floors](audit-ledger-consistency-pass-apostrophes-and-block-floors.md)
- [Audit ledger: the consistency pass, the page-wide proof, forms, floors and limits](audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md)
- [Audit ledger: the consistency pass, stack depth, the floors fixed point and the page form](audit-ledger-consistency-pass-stack-depth-floors-fixed-point-and-page-form.md)
- [Audit ledger: the consistency pass, one reading of every mark](audit-ledger-consistency-pass-one-reading-of-every-mark.md)
- [Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md)
- [Audit ledger: the consistency pass, parse boundaries, the letter test and the proof's page](audit-ledger-consistency-pass-parse-boundaries-letter-test-and-proof-page.md)
- [Audit ledger: the consistency pass, the ninth review](audit-ledger-consistency-pass-ninth-review.md)
- [Audit ledger: the consistency pass, the independent oracle and the rules for marks](audit-ledger-consistency-pass-independent-oracle-and-mark-rules.md)
- [Audit ledger: the consistency pass, the tenth review](audit-ledger-consistency-pass-tenth-review.md)
- [Audit ledger: the consistency pass, the re-review and the refusal run](audit-ledger-consistency-pass-re-review-and-the-refusal-run.md)
- [Audit ledger: the consistency pass, what the refusal run steps over](audit-ledger-consistency-pass-what-the-refusal-run-steps-over.md)
- [Audit ledger: the consistency pass, span edges as whole units](audit-ledger-consistency-pass-span-edges-as-whole-units.md)
- [Audit ledger: the consistency pass, what the word test reads as shown](audit-ledger-consistency-pass-what-the-word-test-reads-as-shown.md)
- [Audit ledger: the consistency pass, elisions, quoted numbers and pairing](audit-ledger-consistency-pass-elisions-quoted-numbers-and-pairing.md)
- [Audit ledger: the consistency pass, the prose mask and slices parsed alone](audit-ledger-consistency-pass-the-prose-mask-and-slices-parsed-alone.md)
- [Audit ledger: the consistency pass, definitions inside containers, fragment landings and the mask memory](audit-ledger-consistency-pass-definitions-landings-and-the-mask-memory.md)
- [Audit ledger: the consistency pass, voices lost and places left unlocated](audit-ledger-consistency-pass-voices-lost-and-places-left-unlocated.md)
- [Audit ledger: the consistency pass, page definitions recorded and carries gated by generation](audit-ledger-consistency-pass-page-definitions-recorded-and-carries-gated.md)
- [Audit ledger: follow-ups on emphasis definitions, cache accounts and the Synthetic quota shape](audit-ledger-follow-ups-emphasis-definitions-cache-accounts-and-synthetic-quota.md)
- [Audit ledger: the package split, configs, scans and messages](audit-ledger-package-split-configs-scans-and-messages.md)

## Process mistakes in this audit

These are the agent's own mistakes while fixing,
recorded for the prevention doc.

Filed in:

- [Audit ledger: process mistakes, shell commands and tasks](audit-ledger-process-shell-commands-and-tasks.md)
- [Audit ledger: process mistakes, launch checks and cache versions](audit-ledger-process-launch-checks-and-cache-versions.md)
- [Audit ledger: process mistakes, docs that grow or go stale](audit-ledger-process-docs-that-grow-or-go-stale.md)
- [Audit ledger: process mistakes, guards and red cases](audit-ledger-process-guards-and-red-cases.md)
- [Audit ledger: process mistakes, claims and commit messages](audit-ledger-process-claims-and-commit-messages.md)
- [Audit ledger: process mistakes, checks run before a commit](audit-ledger-process-commit-gates.md)
- [Audit ledger: process mistakes, searches and censuses](audit-ledger-process-searches-and-censuses.md)
- [Audit ledger: process mistakes, lint, edits and tests](audit-ledger-process-lint-edits-and-tests.md)
- [Audit ledger: process mistakes, stopping and waiting](audit-ledger-process-stopping-and-waiting.md)
- [Audit ledger: process mistakes, agents and briefs](audit-ledger-process-agents-and-briefs.md)
- [Audit ledger: process mistakes, agents and briefs in the consistency fix rounds](audit-ledger-process-agents-and-briefs-consistency-fix-rounds.md)
- [Audit ledger: process mistakes, agents and briefs in the reference, edge and prose mask rounds](audit-ledger-process-agents-and-briefs-reference-edge-and-prose-mask-rounds.md)
- [Audit ledger: process mistakes, agents and briefs in the review, docs, archive and empty link rounds](audit-ledger-process-agents-and-briefs-review-docs-archive-and-empty-link-rounds.md)
- [Audit ledger: process mistakes, agents and briefs in the rounds after the split](audit-ledger-process-agents-and-briefs-rounds-after-the-split.md)
- [Audit ledger: process mistakes, the lead's slips by batch of work](audit-ledger-process-the-leads-slips-by-batch.md)
- [Audit ledger: process mistakes, the package split](audit-ledger-process-the-package-split.md)
- [Audit ledger: work merged from parallel briefs](audit-ledger-work-merged-from-parallel-briefs.md)
- [Audit ledger: process mistakes, questions and handoffs to the owner](audit-ledger-process-questions-to-the-owner.md)
