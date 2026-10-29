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
the owner has since closed the Luna direction
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
The owner refused it on 2026-10-29:
thinking-block text,
even mere summaries,
is off limits to external providers.
The direction is closed,
not deferred;
a future detector would have to read thinking blocks locally.

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
  never goes to an external provider;
  the Luna direction is closed.
- Accepted by the owner 2026-10-29:
  `RG2`'s scope widened to any code consuming unbounded input,
  landed in `cf88f54ed8`
  (`AGENTS.md` heading `Unbounded input`,
  `CLAUDE.md` regenerated by file-enforcer).
- Not decided:
  which gate to build.
  The question is open in the session that ran these experiments,
  waiting on the owner's read of `Measured results`.

## Next action

The owner reads `Measured results`
and then picks the gate direction in the open question.
Ranking on this evidence:
standards-review gate,
then ratio-test gate.
After the pick:
measure the standards-review gate's recall
on the eight historical fix commits before trusting it,
or build the ratio gate with warmup control and a proven exposing shape.
