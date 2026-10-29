# Superlinear cost dismissed as acceptable: gate research

## Purpose

State of the research into how to stop agents from waving away superlinear cost with "inputs are short today".
Written 2026-10-29 for the next session;
updated the same day with the runs under `Measured results`.
The owner paused the work,
then asked to resume it.
`Open` holds what those runs left undecided.
No `AGENTS.md` edit,
decision doc,
or code change came out of this work.

## Trigger

The owner found a thinking block in an earlier session (`73f0639e`) that read:
a mild quadratic cost in the gap-finding logic is acceptable because lines are short in practice,
despite not being strictly linear for unbounded input.
A second thinking block in the same stretch called the cost negligible for short CLI rows.
The owner's reply:
lines might be short now,
but they can grow.
The owner then asked for a way to make future agents less likely to do this,
and rejected a wording-only change for this one case as a band-aid.

## The code behind it

The sentence concerns `readBinRow` in `package/file-enforcer/core/src/toolchain/providers.parse.ts`.
It tested every column gap with a `.some()` over all gaps,
inside a `.find()` over the same gaps.
The fix is commit `58d2bb414d`.
Its message records a crafted 20,000-gap row taking 2383 ms before and 4.5 ms after.
That the sentence concerns this function is an inference from the code and the commit,
because the earlier transcript was not read in full.

## Findings

### The existing rules already cover the failure in text

`RG2` requires a regex replacement to be proven O(n) for unbounded input.
The dismissal quoted that criterion and overrode it.
`doc/philosophy/agents.md` says `AGENTS.md` grows only when no rule covers the failure.
`doc/audit/chain-flatten-skewed-tree.md` reached the same conclusion for a different bug:
correct in text,
failed in application,
same trap twice.
Its follow-up added sharper text in three places,
and the class recurred.
The `PXQ` section of `doc/philosophy/agents.md` records a rule that stayed in `AGENTS.md` and still failed for a month.
A standards-directed blind review applied `RG2` to this very scan as its top finding
(see the heading `Blind review: neutral misses, standards catches`),
so the text does the work once a reviewer is pointed at it.

### The dismissal lived only in thinking

In `73f0639e` the sentence appears only in thinking blocks,
not in a message or a tool input.
The stop hook sends no thinking blocks to its detectors
(`package/claude-code-plugin/stop-reminder/README.md`,
the Luna section),
and no document I read gives a reason for that exclusion.
The first block also leaned on "passes fuzz testing" as reassurance,
which short random inputs cannot justify.

### Prior art

No published false-positive rate exists for a lint rule of this kind.
The closest is `react-doctor/js-set-map-lookups`,
an oxlint rule that flags `includes` and `indexOf` inside loops.
Its page says a match proves only the modeled source pattern.
A Sentry experiment found one real quadratic reduce callback and reported no totals.
`doc/planning/wall-clock-bound-test-remedies-review.md` measured representative quadratic string work at 40 ms to 200 ms
against a 250 ms bound,
so an absolute wall-clock bound does not catch it.
That review recommends raising the input size or adding a two-size ratio check with a positive control.

## Rejected: a nested-scan lint rule

I prototyped a detector in scratch and measured it.
It parses production TypeScript with Node's type stripping plus `acorn`
and reports a search call (`indexOf`,
`includes`,
`some`,
`find`,
and similar)
inside an iteration callback or loop body.

- Scope:
  5,404 files selected,
  5,390 parsed,
  14 failed to parse.
- Counts by cumulative tier:
  1,923 for any scan nested in a scan or loop,
  1,282 for a search call,
  653 when the searched collection is declared outside the nest,
  and 511 when that search also has no start-position argument.
- False positives:
  I classified every 12th hit of the 490 tightest-tier hits not on a module-level constant,
  41 samples.
  Seven were test fixtures or fuzz packages that the file filter missed,
  leaving 34.
  About 19 were harmless (constant character sets and rosters,
  small fixed domains,
  string searches,
  short CLI dedupe).
  About 10 looked like real superlinear work on document-scale data,
  and about 5 were unclear.
- Recall:
  on the pre-fix code of 8 historical commits that fixed non-regex quadratic work,
  the tightest tier flagged the changed hunks in 3,
  a looser tier in 1,
  and missed the rest,
  including the accumulator-copy fixes.
- Cost:
  about 10 of 34 real-looking hits extrapolates to roughly 140 existing sites.
  Each suppression needs a justified disable comment and a doc (`LN3`),
  so the rule would demand hundreds of docs.

Limits of this evidence:
one rater (me),
no measurement of the data sizes behind any hit,
and a file filter that leaked tests and fuzz packages.
Conclusion:
a lint rule is not viable as a gate at a tolerable false-positive rate.
The owner called the proposal laughable without a measured false-positive rate,
and the measurement agrees.

## Measured results

Two of the three directions got one run each on 2026-10-29;
the owner has since constrained the Luna direction
(see `Decisions on record`).
Limits shared by both runs:
one bug (`readBinRow` before `58d2bb414d`),
one sample per arm,
and no false-positive rate measured.
`diff` of `parse-after.ts` against `package/file-enforcer/core/src/toolchain/providers.parse.ts` prints nothing,
so the scratch post-fix parser is byte-identical to production and `parse-before.ts` differs from it only by the fix.

### The ratio gate separates the parsers only on an exposing shape

`ratio-gate.ts` ran unchanged at sizes 2,000 to 16,000 gaps,
best of five.
All three of its generators read doubling ratios near 2 for both parsers,
so none separates them.

The reason is `gaps.find` inside `readBinRow`:
it returns at the first gap whose following word
is itself followed by a column gap.
In `'x  '.repeat(n)` and in a repeated valid row the first gap
already matches,
so the nested `gaps.some` scan runs a constant number of times.
This is the prediction about repeated valid rows under `Ratio-test gate`,
extended:
the "many short columns" shape is inert too.

`ratio-gate-2.ts` adds the shape that exposes the scan:
`z` followed by `  a b` repeated n times.
Each gap there is followed by a word that is single-space separated from the next word,
so every gap fails the version-column test,
`gaps.find` walks all n gaps,
and each walk runs `gaps.some` over all of them.
Sizes 1,000 to 8,000 gaps,
best of three:

- Failing gaps,
  then a version match (the row parses),
  pre-fix 3.28,
  3.45,
  5.15;
  post-fix 2.00,
  2.00,
  2.01.
- Failing gaps,
  no match (the row parses to nothing),
  pre-fix 3.72,
  0.77,
  5.05;
  post-fix 2.27,
  1.58,
  1.85.

A 20,000-gap row of the exposing shape takes 518.6 ms pre-fix and 2.8 ms post-fix.
`58d2bb414d` records 2383 ms and 4.5 ms on its own crafted row:
same class,
different shape and process state,
so absolute numbers do not carry over.

Two confounds a gate must control:

- Cold ratios are unreadable.
  The unwarmed pre-fix series on the no-match shape reads
  4.64,
  17.25,
  13.20,
  66.64 ms,
  non-monotonic.
  `ratio-gate-warmup.ts` discards two best-of-five
  timings at the largest size first,
  then reads 3.51,
  3.79,
  4.54 pre-fix and 1.98,
  2.08,
  2.04 post-fix.
  The non-monotonic point is a warmup or execution-order effect consistent with JIT tier-up;
  no V8 trace was taken,
  so the cause stays unproven.
- Spread and ordering.
  Ten best-of-three samples of one identical 4,000-gap pre-fix input read
  12.181 to 12.888 ms (about 5.8%),
  and the spread probe in `ratio-gate.ts`
  (post-fix,
  best of five) reads 0.293 to 0.304 ms. That separates a doubling ratio near 4
  from one near 2 under these conditions only:
  each spread covers one arm,
  one size,
  one process,
  and the arms ran in fixed pre-then-post order.

`bin-list-scaling.ts` never ran as a comparison:
its "old" arm `parse-old.ts` is byte-identical
to `parse-new.ts` and `parse-after.ts`,
so it would compare the fix with itself.

### Blind review: neutral misses, standards catches

Two fresh subagents read only `blind-review/providers.parse.ts`,
the pre-fix module,
with the fix,
this handover,
and git history unavailable to them:

- Neutral review prompt:
  six findings (1 major,
  2 minor,
  3 nits),
  all correctness and robustness.
  The cost of the nested `gaps.some` scan is not among them,
  and the arm reported
  no findings in `findColumnGaps`.
- Standards review prompt against `AGENTS.md`:
  its first and only major finding is the target,
  `readBinRow` under `RG2`,
  naming the `gaps.find` over `gaps.some` per candidate
  and "probe stdout is unbounded".

So the rule text does the work once a reviewer is pointed at it,
and a neutral reviewer
shares the author's blind spot.
`RG2` is worded for code replacing a regex;
the reviewer generalized its criterion,
"proven O(n) for unbounded input",
to this scan.

By-products of the neutral arm,
all in gap semantics the fix left untouched and so present
in production:
a path column containing a two-character space run parses its tail as the version;
`parseBinList` splits on `\n` while `readBinRow` guards other line terminators;
`parseCargoBinstallList` normalizes `cargoVer` but not `binstallVer`.
Not investigated here,
recorded so they are not lost.

## Open

The owner chose three directions after the lint rule failed.
The runs under `Measured results` narrow what each one would mean;
none is built.

### Ratio-test gate

The idea is a helper that runs a function at sizes n and 2n on adversarial input and fails when
the time ratio is far above 2,
with a self-test proving a synthetic quadratic trips it.
The first session predicted,
unmeasured,
that repeating valid rows would not expose the bug,
because the first matching gap returns early;
`Measured results` confirms that and extends it to the "many short columns" shape.
Two roles remain possible and they differ sharply:

- A regression gate for a known bug:
  run the function at n and 2n on the exposing shape,
  require the ratio near 2,
  and prove the shape exercises the suspected path.
  This works,
  but only where someone has already understood the bug well enough
  to write the exposing shape.
- A discovery gate for unknown bugs:
  take the maximum ratio over a portfolio of generated shapes,
  so nobody has to know the shape in advance.
  Untried.
  The inert generators are the demonstration of what one hand-authored shape costs:
  two "adversarial" inputs were written in this research that never reached the nested scan.

Untouched:
where the gate runs (a package task,
a sidecar,
CI) and what it demands of callers.

### Separate-reviewer gate

The test ran:
an independent review shown only the code and not the author's reasoning
(`doc/agent/self-review.md` says same-session review is not independent review).
A standards-directed arm caught the bug;
a neutral arm missed it,
so the gate means
routing candidate diffs through `project-code-review` rather than a general review.
Untouched:
recall on a corpus (the eight historical fix commits used for the lint rule are
the natural set),
false-positive rate,
per-diff cost,
and determinism.

### Luna

Luna is `openai/gpt-6-luna-decisions`,
the model the `ccsr` Stop hook uses to label the situation of the final message
(`doc/handover/stop-reminder-luna.md`).
Findings so far:
the hook sends the final message only,
with no thinking blocks,
and runs at the end of the turn,
long after a mid-turn dismissal.
The handover records that a Luna label for uncertainty is not built and needs its own calibration under #1772.
Thinking blocks are readable in the session transcript,
so a detector could read them.
An experiment would replay thinking blocks through Luna,
with the two dismissal blocks as positives and a sample of other blocks as negatives.
That sends reasoning text to an external provider,
which the hook does not do today,
so it needed the owner's authorization first.
On 2026-10-29 the owner ruled thinking-block text off limits to external providers,
even mere summaries,
which blocks the replay-through-Luna shape as proposed.
The direction stays open:
the owner corrected a wider reading that day,
and any detector it yields must keep thinking text on this machine.

## Scratch artifacts

All in `/var/home/user/temp/agent/`,
uncommitted,
and safe to delete:

- `nested-scan-detect.ts` (detector and repo scan),
  `sample-hits.ts` and `sample-12.txt`
  (the classified sample),
  `recall.ts` (historical fix-commit test),
  `tier3-hits.json` (the 511 hits).
- `parse-before.ts` (pre-fix parser) and `parse-after.ts` (post-fix,
  byte-identical to production);
  `parse-old.ts` and `parse-new.ts`,
  both byte-identical to `parse-after.ts`,
  which is why `bin-list-scaling.ts` is not a comparison.
- `ratio-gate.ts` (run),
  `ratio-gate-2.ts` (run,
  exposing shapes),
  `ratio-gate-warmup.ts` (run),
  `bin-list-scaling.ts` (never run),
  and `blind-review/providers.parse.ts` (pre-fix copy given to the two reviewers).
- `recall.ts` (the lint rule's historical test),
  `recall-extract.ts` (corpus extractor),
  and `blind-recall/`
  (13 hash-free case snippets,
  `manifest.tsv` mapping cases to commits,
  `verdicts.tsv` with the scored reviews).

`nested-scan-detect.ts` needs Node's experimental `stripTypeScriptTypes` and the `acorn` copy in `node_modules/.pnpm`.

## Decisions on record

- Rejected by the owner:
  a wording-only change for this single case.
- Rejected by measurement:
  the nested-scan lint rule.
- Measured 2026-10-29:
  the ratio gate separates the parsers only on the exposing shape and after warmup;
  a neutral blind review misses the cost bug;
  a standards review catches it under `RG2`.
- Decided by the owner 2026-10-29:
  thinking-block text,
  even mere summaries,
  never goes to an external provider.
  The Luna direction stays open;
  only routes that carry thinking text off this machine are ruled out,
  and the owner corrected a wider reading the same day.
- Accepted by the owner 2026-10-29:
  `RG2`'s scope widened to any code consuming unbounded input,
  landed in `cf88f54ed8`
  (`AGENTS.md` heading `Unbounded input`,
  `CLAUDE.md` regenerated by file-enforcer).
- Picked by the owner 2026-10-29:
  the standards-review gate,
  measured at 6 of 8 recall on the reconstructed historical corpus
  (see `Recall measurement results`).
- Decided by the owner 2026-10-29:
  the gate runs in shadow,
  observe and record and never block,
  and no `AGENTS.md` rule backs it in that phase.
  Built as `doc/runbook/standards-review-gate.md`
  with `doc/audit/standards-review-gate-shadow.md`
  (commit `d6c6777974`).
- Subject named 2026-10-29 at the owner's question "shadowing who":
  the gate shadows agent landings in this repo,
  Claude Code first and every harness that lands here,
  triggered by the landing session on its own diffs while shadowing.
  An empty log means no run happened,
  never that there was nothing to catch.

## Next action

Build the standards-review gate as the procedure
that routes candidate diffs through a standards review before landing.
Incumbent check done 2026-10-29:
no repo owner carries a pre-landing review duty
(nothing in `doc/agent/`,
`doc/runbook/`,
or git-policy package docs),
`doc/agent/self-review.md` only obliges delegating to a separate reviewer when asked,
git-policy owns landing without a review requirement,
and `project-code-review` is a harness skill invoked on request.
Where the gate lives is decided:
it runs in shadow,
built as `doc/runbook/standards-review-gate.md`
with its log at `doc/audit/standards-review-gate-shadow.md`,
per the owner's 2026-10-29 ruling
that everything runs as shadow
and that no `AGENTS.md` rule backs it in that phase,
since a mandate would not count as shadow.
Next:
run it on real candidate diffs as they appear
and append each run to the log,
which measures per-diff cost and determinism.
Graduation then goes back to the owner
(`AGENTS.md` rule versus git-policy landing check).
Then wire the two miss shapes into the gate:
cost must be accounted across call boundaries,
and an inline complexity claim is never proof
(`RG2` already demands proof for unbounded input).
The ratio-test gate stays ranked second
as the regression tool for known exposing shapes;
the Luna direction stays open under the thinking-text constraint.

## Recall measurement results

Scoring rule,
written before the first verdict landed:
a case is a catch when the review names the superlinear cost of the changed region
(nested scan,
repeated work per item,
accumulator growth) at any severity;
a miss is a review with no cost finding;
partial when cost is gestured at but not named.
One blind standards reviewer per case read only its hash-free snippet and `AGENTS.md`
(after the `RG2` widening of `cf88f54ed8`);
the corpus is the scratch `blind-recall/`
(extracted by `recall-extract.ts`),
with verdicts in `blind-recall/verdicts.tsv`.

### Which commits the 8 were

The first session counted 8 qualifying commits
but never named them.
Fix-commit bodies reconstruct them:
the 8 non-regex superlinear fixes are `58d2bb414d`,
`3d49942409`,
`eb93c91bcf`,
`86d98c8284`,
`566de2e511`,
`4008eeb99b`,
`be0897f009`,
and `725c6df76c`.
The other 5 candidates
(`25c469152d`,
`d16977a5d9`,
`c436b4c48e`,
`adf0c5f9a6`,
`304ae283d5`)
are audit-driven linearization work,
so they double as specificity probes.

### Result: 6 of 8 caught

Catches,
each naming the cost at major or blocker severity under `RG2`
(and `ITR` for the accumulator case):

- `58d2bb414d`:
  `gaps.some` per candidate gap,
  O(n^2) when nothing matches;
  the reviewer scoped `findColumnGaps` correctly as linear.
- `3d49942409`:
  work growing with anchor-and-place pairs,
  quadratic on repeated words.
- `eb93c91bcf`:
  per-offset re-comparison and per-place slicing,
  quadratic in page words.
- `86d98c8284`:
  placings times places double loop with per-pair slicing.
- `566de2e511`:
  an O(n^2) census,
  `fences.slice(0, index + 1).filter(...)` recomputed per line.
- `be0897f009`:
  blocker,
  spread-copy accumulators per element,
  with the module's own "one linear pass" claim falsified.

Misses:

- `4008eeb99b`:
  the review cleared `RG2` as "monotone-cursor single linear passes"
  and trusted the code's own "ONE LINEAR PASS" comment,
  while `nearestScheme` re-read to the text's end at every run
  (the fix's measured defect).
- `725c6df76c`:
  no cost finding at all,
  although the excerpt holds the quadratic site
  (per-entry spread copies of the whole pending record
  plus a whole-journal `writeJournal` per entry,
  the twice-per-entry rewrite the fix measured at 3.46 GB for 4403 entries).

Specificity probes:
no superlinear claim on `25c469152d`,
`c436b4c48e`,
`adf0c5f9a6`,
or `304ae283d5`
(each review either passed `RG2` explicitly
or judged the pattern O(n)),
and one cost claim on `d16977a5d9`
that is correct:
its body measures the re-export matcher quadratic at
4.2/17/73/250/1010 ms per doubling.
Two probes also named the exact duplication their fix unified (`UTL`).

### Reading the number

The lint rule's tightest tier caught 3 of the same 8
(a looser tier 4),
missing the accumulator-copy class;
the standards review caught 6 of 8,
including that class at blocker severity.
Both misses share a shape:
the cost sat behind an inline complexity claim
or in composition across a call boundary
(`writeJournal` per entry),
where each function alone reads as a linear pass.

Limits of this evidence:
one reviewer route per case (fresh subagents on the default child route),
a prompt equivalent to `project-code-review`
rather than the skill's own procedure,
hunk-window excerpts rather than full diffs
(case 13 is 1339 lines across 14 files),
no per-diff cost or run-to-run determinism measured,
and specificity probed on 4 non-superlinear cases only.
