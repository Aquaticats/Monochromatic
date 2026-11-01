# Translation repair history 2026-08-05 to 2026-08-07: round three prepared and its first runs

Archiving round two,
the stop-at-fifteen-entries policy,
the grading tools built while the pass ran,
the reproducible grading arithmetic,
the policy to land certainly-good fixes at once,
and the restarts of the pass through its second and third runs.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Round three prepared, and its first runs

### Round three preparation

Round two ARCHIVED to `round-two-archive/` inside the runs dir,
mirroring the
existing `round-one-archive/` layout:
`artifacts/`,
`attempts.json`,
`slice-cache/` moved,
and `gate-verdict.md` plus
`grading-sheet.md` COPIED so the seed-named originals stay where
`resolveSheetPath` protects them from being clobbered.
A fresh empty `artifacts/` was created in their place.

`corpus-pass -- --plan` verified at zero quota after the archive:
tip `f7943a196`,
92 pending,
0 done,
client constructed,
soft budget 12h,
hard cap 3h per entry.

`DEFAULT_SAMPLE_SEED` advanced to `milestone-three-precision-round-three`.
A new seed does NOT guarantee no already-graded issue is redrawn;
what mostly changes is the population,
since round three draws from artifacts
produced by a fresh pass and issue ids are content-derived.

`sentinel-probe` now reports accepted issues counted by repair disposition and
how many issues were refined,
so a real-model probe can tell a run that records
provenance from one that does not.
That check is the one unit tests cannot do.

### Round three run policy (user decision, 2026-08-06)

STOP AT ~15 SETTLED ENTRIES,
then draw.
User asked why the next step needed twelve hours;
it does not,
and the twelve-hour figure was the run's SOFT BUDGET rather than a wait.

The arithmetic that decided it,
recorded because it recurs every round:

- The sample is 50 ISSUES,
  not 50 entries.
  Round two's 31 entries produced 2257 accepted issues,
  so issue supply is never the constraint.
- What entry count buys is PAGE DIVERSITY.
  `selectFromBand` round-robins across entries,
  so 31 entries spread 50 issues over ~31 pages
  and 15 entries spread them over ~15,
  about 3 to 4 per page.
  Sample size is unchanged either way,
  so the confidence interval is unchanged;
  what rises is clustering,
  since issues from one page share a translator and an error style.
- `band-order.ts` interleaves bands round-robin and documents reaching
  ten per band at about thirty entries,
  so fifteen entries lands about five per band by construction.
  A plain artifact count is therefore a correct stop condition;
  no band-aware check is needed.
- At round two's measured rate (252 min for 7 entries,
  about 36 min/entry),
  fifteen entries is roughly seven to nine hours,
  against roughly eighteen for thirty.

The pass is NOT reconfigured for this.
`SOFT_BUDGET_MINUTES` stays 720;
the run is stopped by hand once the artifact count reaches fifteen.
Changing run config mid-run would not affect the running process anyway.

### Task 48 tooling, built while the pass ran

`grade-sheet-read.ts` (`parseGradedSheet`) and
`grade-agreement.ts` (`scoreGradeAgreement`,
`scoreGradedPrecision`).

The parsing rules come from the two sheets the user has ACTUALLY graded,
which are in different formats and neither of which anyone specified:

-   round one:
    `### 3. grade: Y  (Y = ...)`,
    bare,
    and `### 2. grade: N. <rationale>`.
-   round two:
    `### 4. grade: [Y]`,
    bracketed,
    and `### 7. grade: [Y, <rationale>]`.
-   both rounds:
    answers that are NO verdict,
    such as `[Not enough context to grade]` and
    `[Not sure which tense is best here...]`.

A verdict letter counts as a verdict only when a delimiter follows it.
`Not enough context to grade` begins with `N`,
and reading it as a false positive would move a question the grader DECLINED
into the precision denominator on the strength of one letter.
Both denominators exclude declined items and report their positions,
so their number is never invisible.

PRE-GRADES STAY IN THEIR OWN FILE,
never printed on the sheet.
This was decided rather than asked,
because the user's own stated plan determines it:
showing the agent's grade would anchor the human toward agreeing,
and the same sheet produces the milestone gate number,
so the calibration would be bought by corrupting the measurement it calibrates
against.
Nothing is lost,
because the agreed plan only starts FILTERING items a round later.

### Provenance verified in a live run

The first slice the round-three pass persisted
(`slice-cache/AmbeR_the_anpa/a821a954...json`) carries
`repairRegions` with one real region
(envelope id,
one issue served,
14 characters replaced by 12),
`accuracyPatchSelected: true`,
`refined: false`,
and that issue in `resolvedIssueIds`.
The slice cache is what made this checkable in minutes rather than after a whole
entry:
it serializes `ChunkRepairOutcome` after EVERY finished slice,
so provenance is inspectable long before the first artifact lands.
Use it that way next time instead of gating a long run behind a probe entry.

### Grading arithmetic is now reproducible from the sheets

`mise run //package/module/translation-repair:score-agreement -- --sheet <abs path>` (moved on 2026-10-18 to
`//package/translation-repair/probe:score-agreement`)
reads a graded sheet and prints the numbers,
so a verdict no longer depends on counting by hand.
Pass an ABSOLUTE path:
mise runs the task with the package directory as cwd,
so a repo-relative path resolves wrongly.

Validated against both sheets a human has graded.
Round two reproduces its published verdict EXACTLY:

```text
PRECISION items=50 scored=47 realDefects=37 strict=0.740 excluded=0.787 lenient=0.800 unscored=10,12,17
```

That is the strongest available check on the reader,
since it recovers a measurement nobody told it.
It also settles what the three published numbers meant,
which was never written down:
strict counts a declined item as a false positive (37/50),
excluded drops declined items from the denominator (37/47),
and lenient counts them as real defects (40/50).

Round one,
scored by the same tool for the first time:

```text
PRECISION items=50 scored=44 realDefects=28 strict=0.560 excluded=0.636 lenient=0.680 unscored=12,16,21,33,34,48
```

So the fix rounds moved excluded precision 0.636 -> 0.787,
and round three's target remains 0.9.
Both rounds left a similar share undecided
(six of fifty,
then three of fifty),
which is worth watching:
the undecided share is itself a signal about how gradable the sheet is.

Output carries counts and sheet POSITIONS only,
never a quote,
a claim,
or a grader's rationale,
so it is safe to paste into a verdict or a message
even though the sheets hold unlicensed corpus text.

### Policy change: land certainly-good pipeline fixes immediately (user, 2026-08-06)

User instruction,
verbatim intent:
land all certainly-good pipeline changes immediately and restart the runs as
many times as needed;
there is no need to save tokens on this provider.

This REVERSES the sequencing used earlier in the session,
where pipeline fixes were held so the round-three measurement would run against
frozen code.
Restarting is cheap;
measuring a pipeline you already know is wrong is not.

"Certainly-good" still does work in that sentence.
Landed under it:

-    Task 52,
     credit only served issues.
    The defect is indefensible rather than debatable:
    a patch could win on credit for an issue nothing touched.
-    Task 56,
     cache key covers run shape.
    Same character:
    a resumed slice could carry another roster's outcome silently.

NOT landed under it,
because each is a design choice rather than a defect with one right answer:

-    Task 53's REBUILD half.
    Renaming `regressionCount` and `changedCharCount` to what they measure is
    safe;
    building the measurements their names promise changes what selection ranks
    by and needs a decision.
-    Task 31,
     judge crosscheck.
    A new stage with its own cost and failure modes.
-    Task 54.
    Purely additive telemetry,
     so it is safe,
     but it was not needed to unblock
    the run and can land beside 53's rename.

### Round three, run 002

Run 001 was stopped after one entry and three slices.
Its slice cache was DELETED rather than kept:
the pipeline changed under it,
and although the new run-shape key would have missed those entries anyway,
leaving them invites the exact confusion the key exists to prevent.
Artifacts were still empty,
so nothing settled was lost.

`pass8-run-002.log` is the live run.
Stop condition remains fifteen settled entries.

### Findings from run 001 that survive the restart

REPAIR PROVENANCE WORKS END TO END ON LIVE DATA.
Three real slice outcomes yielded seven accepted issues,
all `shipped`,
all carrying regions.
Rendering both sheets from them showed:
seven grade boxes and seven headings OUTSIDE fenced blocks,
no replacement text outside a fence,
and no replacement text anywhere in the detection sheet.

READING THE SHEET AS A GRADER FOUND WHAT COUNTS COULD NOT.
The SHARED line printed five full 64-character issue ids,
about a third of a kilobyte of hash a grader cannot look up,
burying the one fact they can act on:
the same before and after text is about to repeat under other items.
It now names those items by SHEET POSITION,
and says plainly when a sibling was not drawn into the sample.
Structural checks over that same file all passed,
so nothing but reading it would have caught this.
Generate a sheet and READ it before every round.

MEASURED THROUGHPUT,
corrected.
An earlier claim that the run was slower than round two was WRONG:
it divided wall clock from PASS start,
which includes corpus fetch and setup,
rather than from when `repairTranslation` began.
Measured from the log timestamps on a large-band entry:
7 slices declared,
finished slices at 1.52 and 7.35 minutes,
mean 4.44 min/slice,
against round two's 8.56 min/slice for large.
Per-slice speed is fine.
The genuinely new cost is the naturalness lane,
which did not exist when round two's 36 min/entry was measured
and adds a rewriter call,
a judge round,
and a recheck per eligible slice.

### Run 003, and the rest of the certainly-good backlog

Landed after run 002 started,
so the pass restarted again as `pass8-run-003`:

TASK 54,
`candidateResolvedIssueIds` on `ChunkRepairOutcome`.
`resolvedIssueIds` discards two different things and both are worth auditing:
verdicts on issues no operation served,
which must not earn selection credit but are still what the checkers SAID,
and every verdict at all when the unchanged text won,
so a rejected candidate left no trace of how it was judged.
The new field records them and decides nothing.
`SLICE_CACHE_VERSION` went 2 -> 3 for the shape change.

The served-issue derivation moved to `chunk-measure.ts` as
`selectCreditableIssues`,
because adding the telemetry pushed `repair-chunk.ts` to 301 lines.
Splitting rather than raising the budget,
as MXL requires;
`chunk-measure.ts` is where the other selection inputs already live.

STILL NOT LANDED,
and still deliberately:

-    Task 53's rebuild half.
    The RENAME half is safe and pending only because it changes a public type
    for no measurement gain mid-run;
    it can land any time.
-    Task 31,
     judge crosscheck.
    A new stage with its own cost and failure modes.

RESTART DISCIPLINE that emerged,
worth keeping:
stop the pass,
CONFIRM no `corpus-pass` process survives
(`ps --no-headers -eo pid,args | rg corpus-pass | rg --invert-match 'rg |pgrep'`,
since a bare `pgrep --full` matches its own command line and reads as a false
positive),
delete the slice cache,
then start the next numbered log.
Deleting the cache is belt and braces now that the key covers run shape,
but a stale directory invites exactly the confusion the key exists to prevent.
