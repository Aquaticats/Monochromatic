# Handover: the lane outcome vocabulary and what is mid-flight

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/cli-open-code-review-issue` is now `@monochromatic-dev/open-code-review-issue-cli`.

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Lane outcome vocabulary handover: flagged-pass findings and fixes, 2026-08-18](two-lane-outcome-vocabulary-flagged-pass-findings-and-fixes-2026-08-18.md):
  The measured findings and landed fixes that followed the flagged pass on 2026-08-18:
  relocation locality and line structure loss,
  the semantic wrap and the bundling trap it exposed,
  the translate lane deleting English and the quote guard that replaced the classifier,
  and the reader that joins flagged slices to their replacements.
  Sections:
  - [TWO NEW MEASURED FINDINGS, BOTH COMMITTED](two-lane-outcome-vocabulary-flagged-pass-findings-and-fixes-2026-08-18.md#two-new-measured-findings-both-committed)
  - [THE SEMANTIC WRAP LANDED, AND IT EXPOSED A BUNDLING TRAP (2026-08-18 evening)](two-lane-outcome-vocabulary-flagged-pass-findings-and-fixes-2026-08-18.md#the-semantic-wrap-landed-and-it-exposed-a-bundling-trap-2026-08-18-evening)
  - [THE TRANSLATE LANE WAS DELETING ENGLISH, AND THE FIX CHANGED SHAPE (2026-08-18 evening)](two-lane-outcome-vocabulary-flagged-pass-findings-and-fixes-2026-08-18.md#the-translate-lane-was-deleting-english-and-the-fix-changed-shape-2026-08-18-evening)
  - [The `#107` join reader is built, validated, and waiting](two-lane-outcome-vocabulary-flagged-pass-findings-and-fixes-2026-08-18.md#the-107-join-reader-is-built-validated-and-waiting)
- [Lane outcome vocabulary handover: what the stream guards saw in production traffic, 2026-08-18](two-lane-outcome-vocabulary-stream-guard-traffic-readings-2026-08-18.md):
  What the stream idle windows and the straggler grace looked like on real traffic during the flagged pass,
  including the re-read across every pass log that carries the stream columns,
  the voice-loss shapes,
  and the defects tracked from them.
  Sections:
  - [The idle windows rest on a premise the current traffic contradicts](two-lane-outcome-vocabulary-stream-guard-traffic-readings-2026-08-18.md#the-idle-windows-rest-on-a-premise-the-current-traffic-contradicts)
  - [What the first production traffic showed, written up in full](two-lane-outcome-vocabulary-stream-guard-traffic-readings-2026-08-18.md#what-the-first-production-traffic-showed-written-up-in-full)
- [Lane outcome vocabulary handover: the 2026-08-17 corpus pass, its freeze and the stop at six entries](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md):
  The sittings around the 2026-08-17 corpus pass,
  newest first:
  the two settled pools and what six entries measured,
  the night after the freeze lifted,
  the runaway-call guard built during the freeze,
  the evening grace measurement,
  the freeze itself and the decision to stop at six entries,
  and the state recorded at 23:22Z.
  Sections:
  - [Two pools now exist, and they must not be confused](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#two-pools-now-exist-and-they-must-not-be-confused)
  - [What the six-entry pool measured](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#what-the-six-entry-pool-measured)
  - [After the freeze lifted, 2026-08-17 night](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#after-the-freeze-lifted-2026-08-17-night)
  - [Evening sitting, 2026-08-17: runaway calls](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#evening-sitting-2026-08-17-runaway-calls)
  - [What the evening measurement said, and what it did not](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#what-the-evening-measurement-said-and-what-it-did-not)
  - [Pass state as of 2026-08-17 evening](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#pass-state-as-of-2026-08-17-evening)
  - [The 2026-08-17 corpus pass: freeze, interim state and the stop at six entries](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#the-2026-08-17-corpus-pass-freeze-interim-state-and-the-stop-at-six-entries)
  - [The GLM claim retracted, and the order of work when the freeze lifts](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#the-glm-claim-retracted-and-the-order-of-work-when-the-freeze-lifts)
  - [READ THIS FIRST AFTER A COMPACTION (written 2026-08-17 23:22Z)](two-lane-outcome-vocabulary-corpus-pass-2026-08-17-freeze-and-stop-at-six.md#read-this-first-after-a-compaction-written-2026-08-17-2322z)
- [Lane outcome vocabulary handover: the vocabulary change, the delivery ledger and the version 2 artifact writer](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md):
  The vocabulary change that added the fifth outcome member,
  the preparation identity,
  the comparison moved onto the delivery ledger,
  the version 2 artifact and its builder,
  the corpus pass settling from both lanes,
  and the repair lane's unheard critics.
  Sections:
  - [Landed, pushed, safe](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#landed-pushed-safe)
  - [The vocabulary change landed](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#the-vocabulary-change-landed)
  - [The decision taken while the user was away](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#the-decision-taken-while-the-user-was-away)
  - [Also landed since](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#also-landed-since)
  - [The preparation identity landed](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#the-preparation-identity-landed)
  - [The comparison moved onto the ledger](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#the-comparison-moved-onto-the-ledger)
  - [A note on stripping guards while the trial runs](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#a-note-on-stripping-guards-while-the-trial-runs)
  - [The ledger boundary was strengthened again after a second review](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#the-ledger-boundary-was-strengthened-again-after-a-second-review)
  - [The version 2 artifact landed](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#the-version-2-artifact-landed)
  - [Two holes in the version 2 builder, found by review and closed](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#two-holes-in-the-version-2-builder-found-by-review-and-closed)
  - [The corpus pass now settles from both lanes](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#the-corpus-pass-now-settles-from-both-lanes)
  - [Found while testing it, and FIXED: the repair lane's unheard critics (`#112`)](two-lane-outcome-vocabulary-lane-vocabulary-ledger-and-version-2-artifact-writer.md#found-while-testing-it-and-fixed-the-repair-lanes-unheard-critics-112)
- [Lane outcome vocabulary handover: the version 2 reader, its schema guards and the reviews that closed the invariants work](two-lane-outcome-vocabulary-version-2-reader-schema-guards-and-closing-reviews.md):
  The corrected parser contract,
  the frozen comparison rules and the version 2 parser,
  the build order and the next-actions list with its landed items,
  the closing of the invariants review (#103) and the translate driver guard trap (#95),
  and the independent review of both guards.
  Sections:
  - [The parser contract was corrected before the parser was written](two-lane-outcome-vocabulary-version-2-reader-schema-guards-and-closing-reviews.md#the-parser-contract-was-corrected-before-the-parser-was-written)
  - [The comparison rules are frozen now, which they were not](two-lane-outcome-vocabulary-version-2-reader-schema-guards-and-closing-reviews.md#the-comparison-rules-are-frozen-now-which-they-were-not)
  - [The version 2 parser landed, all eight steps](two-lane-outcome-vocabulary-version-2-reader-schema-guards-and-closing-reviews.md#the-version-2-parser-landed-all-eight-steps)
  - [Next actions, in order](two-lane-outcome-vocabulary-version-2-reader-schema-guards-and-closing-reviews.md#next-actions-in-order)
  - [The invariants review is fully closed (`#103`)](two-lane-outcome-vocabulary-version-2-reader-schema-guards-and-closing-reviews.md#the-invariants-review-is-fully-closed-103)
  - [A GFP trap worth knowing about in the translate driver (`#95` closed)](two-lane-outcome-vocabulary-version-2-reader-schema-guards-and-closing-reviews.md#a-gfp-trap-worth-knowing-about-in-the-translate-driver-95-closed)
  - [An independent review of both guards, and what it changed](two-lane-outcome-vocabulary-version-2-reader-schema-guards-and-closing-reviews.md#an-independent-review-of-both-guards-and-what-it-changed)
- [Lane outcome vocabulary handover: the post-trial queue and the two-lane cost run](two-lane-outcome-vocabulary-post-trial-queue-and-two-lane-cost-run.md):
  The run queue decided in advance after the window trial,
  the two-lane cost run and its sublinear cost finding,
  the sweep owed when that run exited and the work that followed it,
  the lifting of the launch gate,
  and the afternoon sitting of 2026-08-17 that closed several queue items.
  Sections:
  - [Where the 2026-08-17 afternoon sitting got to](two-lane-outcome-vocabulary-post-trial-queue-and-two-lane-cost-run.md#where-the-2026-08-17-afternoon-sitting-got-to)
  - [The post-trial run queue, decided in advance](two-lane-outcome-vocabulary-post-trial-queue-and-two-lane-cost-run.md#the-post-trial-run-queue-decided-in-advance)
  - [The two-lane cost run, launched 2026-08-16](two-lane-outcome-vocabulary-post-trial-queue-and-two-lane-cost-run.md#the-two-lane-cost-run-launched-2026-08-16)
  - [The cost run finished, and a seven hour cap clears the corpus (`#114`)](two-lane-outcome-vocabulary-post-trial-queue-and-two-lane-cost-run.md#the-cost-run-finished-and-a-seven-hour-cap-clears-the-corpus-114)
  - [What is owed the moment the cost run exits](two-lane-outcome-vocabulary-post-trial-queue-and-two-lane-cost-run.md#what-is-owed-the-moment-the-cost-run-exits)
  - [The launch gate has lifted](two-lane-outcome-vocabulary-post-trial-queue-and-two-lane-cost-run.md#the-launch-gate-has-lifted)
- [Lane outcome vocabulary handover: the rendering audit, from design to the settled-artifact run](two-lane-outcome-vocabulary-rendering-audit-design-population-and-settled-run.md):
  The rendering audit for the translate lane:
  the four artifacts that form its whole input,
  the design decisions and the review that preceded its rebuild,
  the live arms that exposed the matcher,
  and the run over settled artifacts (#115) with its refusal to compare.
  Sections:
  - [Item 5's whole population is four files, and they were sitting in scratch](two-lane-outcome-vocabulary-rendering-audit-design-population-and-settled-run.md#item-5s-whole-population-is-four-files-and-they-were-sitting-in-scratch)
  - [What the rendering audit is for, and the design decisions inside it (`#85`)](two-lane-outcome-vocabulary-rendering-audit-design-population-and-settled-run.md#what-the-rendering-audit-is-for-and-the-design-decisions-inside-it-85)
  - [The audit is rebuilt, run live, and measured (`#85`)](two-lane-outcome-vocabulary-rendering-audit-design-population-and-settled-run.md#the-audit-is-rebuilt-run-live-and-measured-85)
  - [What the review found before any of that ran, kept because the reasoning still applies](two-lane-outcome-vocabulary-rendering-audit-design-population-and-settled-run.md#what-the-review-found-before-any-of-that-ran-kept-because-the-reasoning-still-applies)
  - [Queue item 5 is under way: the rendering audit over settled artifacts (`#115`)](two-lane-outcome-vocabulary-rendering-audit-design-population-and-settled-run.md#queue-item-5-is-under-way-the-rendering-audit-over-settled-artifacts-115)

## READ THIS FIRST AFTER A COMPACTION (rewritten 2026-08-18 07:05Z)

Filed in:

- [Lane outcome vocabulary handover: flagged-pass findings and fixes, 2026-08-18](two-lane-outcome-vocabulary-flagged-pass-findings-and-fixes-2026-08-18.md)
- [Lane outcome vocabulary handover: what the stream guards saw in production traffic, 2026-08-18](two-lane-outcome-vocabulary-stream-guard-traffic-readings-2026-08-18.md)

### THE PASS IS FINISHED AND THE FREEZE IS LIFTED

`DONE processed=5 of pending=5; artifacts=5/92 elapsed=19109264ms`,
`Finished in 19111.45s`,
exit 0.
All five artifacts are in `~/translation-repair-runs-flagged-20260818/artifacts`.
The producing path
is free:
builds,
tests and `#120`/`#121` fixes can all land.

### `#107` IS ANSWERED, AND THE ANSWER IS A NULL

`doc/audit/flagged-slice-replacement-rate.md`.
Five entries,
42 slices.

  pre-registered ledger reading   flagged 26 of 32  0.8125    unflagged 38 of 52  0.7308
  contested slices only           flagged 14 of 16  0.8750    unflagged 20 of 21  0.9524

THE SIGN FLIPS WITH THE DENOMINATOR,
which is what a difference smaller than the instrument looks
like.
As archive survival:
flagged 2 of 16 at 0.125,
unflagged 1 of 21 at 0.048,
natural six-entry
pool 2 of 39 at 0.051.
The contested cut rests on two survivals against one;
moving one slice changes
the direction.

THE NULL COUNTS,
per `QPC`:
the same reader reports 1.0000 against 0.0000 under `--control shipped`,
so it is shown able to display a difference and does not display one here.

WHAT DID DIFFER is contest availability,
16 of 16 flagged slices held a vote against 21 of 26
unflagged,
which is about whether a candidate arrived rather than how judging went.
Critic attention
runs the OTHER way,
12.7 claims per hundred source characters at flagged slices against 15.2.

### SHEET ONE IS GRADED TO ITEM 7 AND SHEET TWO IS OPEN (2026-08-18 afternoon)

The user graded items 1 to 7 of the first relocation sheet and said it was
"once again,
impossible to grade".
Their comments,
verbatim,
are the requirements:

  1 [N]  2 [N]
  3 [?
  The matching might have been broken.]
  4 [N,
  but 姐妹 obviously is better translated to "girl" or "woman" here,
  depending on the final age of the subject.]
  5 [?
  Maybe there is a matching issue]
  6 [?
  Semantic wrapping is paramount even when original Chinese text didn't,
  for maintainability.
  And I don't see an issue with the original translated text.]
  7 [???
  Once again,
  semantic wrapping,
  and I don't see an issue with the original passage.]

PRESERVED BEFORE ANYTHING WAS REGENERATED:
`relocation-sheet.graded-v1.md` and
`relocation-manifest.v1.json` are copies,
and `relocation-grades-v1.json` holds the seven grades
joined to entry,
lane,
slice and flag.
The generator overwrites `relocation-sheet.md` wholesale,
so
that copy is the only record of their work.

SHEET TWO IS BUILT AND OPEN IN GHOSTTY WITH HX:
`~/translation-repair-runs-flagged-20260818/relocation-sheet.md`,
items 8 to 21,
41 KB,
built by
`~/temp/agent/relocation-sheet-2.mjs`,
zero quota.
Seven flagged runs merged whole,
seven controls,
one rewrap-only item dropped.
Three independent boxes per item,
`damaged` / `unnecessary` /
`mismatch`,
plus free `notes`.
Wrapping normalised away for display,
blank lines kept.
Blinding
deliberately dropped.
Full rationale in `#107`.

STILL WAITING FROM BEFORE:
`~/translation-repair-runs-20260817/damage-sheet.md`,
20 items,
ungraded,
which `#68` needs.
And `#96`'s licence question,
recommendation already written in
`doc/planning/artifact-archive-text.md`.

### THE BIGGEST FINDING OF THE NIGHT IS NOT ABOUT `#107`

`doc/audit/incumbent-almost-never-wins-a-contest.md`.
Over the six NATURALLY accumulated entries,
not
hand picked,
the archive's wording survives a contested vote 2 times in 39.
Mean winning weight 3.31
for a fresh candidate against 0.42 for the incumbent.
Eleven of thirteen retentions are defaults
where nothing was offered against it.

BOTH ARTEFACT EXPLANATIONS ARE RULED OUT:
the incumbent is on the ballot anonymously by design,
and
its slot varies across the 39 judged slices,
13/10/5/3/3/5.
It competes blind,
from a moving
position,
and loses.

That bears on `#84`,
on `#96`,
and on whether this pipeline is described as repairing or translating:
`#83` entered the incumbent into selection on the premise it would sometimes win.

### WHAT TO DO NEXT, in order

  1  `#120` defect 6 LANDED in `d0403c9f0`;
  the remaining five defects are open,
  then `#121`;
  2  `#122`,
  wire `package/cli/markdown-lint`'s add-only `semantic-line-breaks` fix into the ship
     path,
  then re-run `~/temp/agent/wrap-census.mjs` and confirm soft-wrapping loss goes to zero
     with the paragraph counts unchanged;
  3  the `#107` window widening the locality measurement bounds:
  show the judge n-1,
  n,
  n+1.
  Longest
     flagged run anywhere is three and every relocation pair is adjacent,
  so that window is enough
     in every case measured;
  4  optionally `mise run //package/module/translation-repair:rendering-audit-settled -- --archive
     ~/translation-repair-runs-flagged-20260818/artifacts` (moved on 2026-10-18 to `//package/translation-repair/probe:rendering-audit-settled`) which SPENDS QUOTA unless `--cap 0`,
  then
     `rendering-audit-settled-report` for `#107`'s reading A,
  the relocation pairs.

### Sol is closed out: issue `#444` filed, no further effort this session

The user's instruction is launch-and-forget and focus on the pipeline.
`Aquaticat/Monochromatic#444`
carries the full report:
three runs at `--thinking xhigh` exiting 124 with zero bytes and NO session
file written,
against a smoke test on the same model and level that answered in 2.2 seconds and wrote
a five-record session.
Three variants were launched and are forgotten;
if any returns,
add a comment
to `#444` and move on.

`--thinking` must stay `xhigh` or `max` for this use,
per the user,
so lowering effort is not a
workaround.

### The main-worktree `package.json` change is explained

`~/.pi/agent/sessions/--var-home-user-Monochromatic--/2026-08-18T01-36-53-044Z_*.jsonl` is a
concurrent `pi` session,
model `gpt-5.3-codex-spark`,
prompt "commit all",
which ran
`git add package.json pnpm-lock.yaml && git commit -m "chore(root): add open code review..."` at
01:37Z.
That is the `@monochromatic-dev/cli-open-code-review-issue` dependency.
Not this session's
work,
correctly left alone per `EC1`,
and no longer a mystery.

### Sol is closed out: `#444` filed, corrected twice, no further effort

THE CAUSE IS THE ROUTE,
not the prompt.
`openai-codex/gpt-5.6-sol` produced exactly ONE response,
at
02:02:53Z,
and has produced nothing since:
about eleven calls,
every one hanging until killed with
zero bytes on stdout and stderr,
regardless of prompt size,
content,
argv or `@file` delivery,
thinking level,
or context-file discovery.

TWO CONTROLS SETTLE IT.
The exact 51-byte prompt that succeeded at 02:02Z now exits 124.
And the
DEFAULT provider,
same flags and same `--thinking xhigh` and same prompt,
answers `ALIVE` in seconds.
So pi works,
`--print` works,
`xhigh` works,
and the prompt works.

A PROMPT-SIZE STORY WAS BUILT AND RETRACTED.
A ladder at 80,
120,
160,
200,
500,
1000,
2000 and 4000
bytes all hung,
and a 260-byte trivially-answerable prompt hung,
which looked like a size threshold
between 51 and 80 bytes.
It was not:
every one of those runs happened AFTER whatever changed,
and
the baseline was never re-run.
A missing control.
`#444` carries the retraction and was retitled.

DO NOT SPEND MORE ON THIS.
The user's instruction is launch-and-forget and focus on the pipeline,
and
`--thinking` must stay `xhigh` or `max` for this use so lowering effort is not a workaround.

### Two settled pools, which must not be confused

`~/translation-repair-runs-20260817/artifacts`,
SIX entries,
NATURAL ACCUMULATION,
may feed a draw.
`~/translation-repair-runs-flagged-20260818/artifacts`,
FIVE entries,
HAND PICKED for carrying
displacement flags,
may serve `#107`'s comparison and may NOT feed any precision,
damage or
accumulation draw.
`corpus-pass` warns about this itself whenever `--only` is used.

### One thing is waiting on the user

`#60`'s DAMAGE SHEET IS WRITTEN AND UNGRADED:
`~/translation-repair-runs-20260817/damage-sheet.md`,
20 items,
33 KB,
outside the repository.
The manifest beside it keeps the probe's verdicts,
which
the sheet deliberately does not show.
Grading it is what `#68` needs.

`#96` ALSO NEEDS A DECISION and the recommendation is already written in
`doc/planning/artifact-archive-text.md`:
store `targetText` whole,
since the premise that it doubles
artifact size was measured and is false,
at 0.6 to 0.9 percent.
The remaining objection is a licence
question,
which is the user's call and not an agent's.

### Concurrent work in the MAIN worktree, which is not mine and was not touched

`/var/home/user/Monochromatic` carries uncommitted changes to `package.json` and `pnpm-lock.yaml`,
adding `@monochromatic-dev/cli-open-code-review-issue` as a workspace dependency.
Nothing in this
session made them and nothing in this session touched them,
per `EC1`.
They were checked only to
confirm they were not a side effect of building here.
Leave them for whoever is doing that work.

### What landed tonight, newest first

  d39fb47a4  the two pools separated in this document
  (lock fix)  runs directory is created before it is locked,
  so a fresh directory works at all
  4149fb0d2  `describeAbandon`:
  a lost voice now says WHY,
  not just that it was lost
  ada369720  `#118`:
  keep what a cut stream delivered,
  report it on both paths,
  name the model
  0b6d9afb0  tests for which delivery rows a damage draw counts
  b71ea7328  `#60`:
  damage draw reads the version 2 ledger,
  BOTH lanes
  7aa4866fa  `#119`:
  end runaway calls in the drain,
  proven by strip
  fd62d7b37  the runaway watch the drain calls
  8517282c1  both channels scanned,
  so thinking runaways are caught
  bf15afaf1  the verse false positive fixed by raising the bar for speaking
  8eb62c0ac  the degeneration detector itself

### Numbers measured tonight that should not be re-derived

  replacement rate      0.740,
  37 of 50 slices,
  six entries,
  translate lane,
  0 withdrawn
                        held from 0.739 at 23 slices,
  so it survived a doubled denominator
  displacement flags    0 across the six settled entries;
  29 of 92 entries flagged corpus-wide,
                        66 flagged slices against 1260,
  about 5 percent
  damage draw           69 shipped regions in the pool,
  32 repair and 37 translate;
  20 drawn;
                        probe flagged 2,
  silent 18
  degeneration          threshold 0.1 distinct;
  varied prose 0.9909,
  structured output 1.0000,
                        a cycling phrase 0.0015,
  verse with a refrain 0.036
  strip evidence        with the runaway guard inert the drain reads 3,060,000 characters;
                        the healthy long call logs 1,652,574 in BOTH runs,
  which is the control

### Retracted claims, so they are not repeated

  "GLM is slow"                     retracted;
  it is a tail on this workload,
  not a slow model
  "1349 streams,
  mean firstByte"    retracted;
  survivorship bias,
  `#118` was the cause
  "an abandoned call leaves nothing" retracted;
  streaming means partial text always exists
  "storing targetText doubles size"  retracted;
  measured at 0.6 to 0.9 percent
  "the grace re-measure is owed"     retracted;
  it was delivered the same morning in
                                     `doc/audit/straggler-grace-remeasure.md`,
  0.039 against 0.138
  "1,590,838 chars is degeneration"  retracted;
  `chars` counts raw event bytes,
  envelope included

## Purpose of this handover, and the worktree its commands assume

Written 2026-08-16 because the user's machine is stuttering and a hard reboot would otherwise lose the state.
Updated after every step from here rather than at the end.
Companion to `doc/planning/two-lane-corpus-pass.md`,
which holds the design record;
this file holds the WORKING STATE.

Worktree:
`/var/home/user/worktrees/translation-repair`,
branch `translation-repair-rebased`.
All commands below assume it,
not the main worktree.

## The one sentence

Every defect in this stretch is the same shape:
**an absence recorded as a deliberate choice**,
and the fix is always to give the absence its own name rather than let it borrow a decision's.

## Standing instruction: `no-non-null-assertion` names the wrong fix

**From the user,
2026-08-16.**
`oxlint`'s `typescript(no-non-null-assertion)` prints help text suggesting optional chaining
(`x?.y` in place of `x!.y`).
Do not follow it.
The fix in this repo is `nonNullishOrThrow` from `@monochromatic-dev/module-or-throw/ts`,
added as a `workspace:*` dependency where a package does not already carry one;
`translation-repair` already does (`package/module/translation-repair/package.json:23` (moved on 2026-10-18 to
`package/translation-repair/*/package.json`)).

The suggestion is not merely stylistically off,
it changes what the code MEANS:
`x!.y` throws where the value is missing and `x?.y` yields `undefined`,
so following the help text turns a loud failure into a silent one,
which is the defect class this whole session exists to close.
The repo's own configuration already states the intent
(`package/config/oxlint/src/rule/restriction.ts:214`:
"Ban non-null assertion (!):
use nonNullishOrThrow instead").

Per the user,
the message documented the correct fix not long before,
so this reads as a REGRESSION
in wording rather than as a rule anyone re-decided.
What is measured so far:
the help text (`Consider using the optional chain operator ...`) appears
nowhere in this repository outside `node_modules`,
so it is emitted by oxlint's built-in
`typescript/no-non-null-assertion` rather than by any config or plugin here,
while the correct fix IS documented on this side,
in
`package/config/oxlint/src/rule/restriction.ts:214` and
`package/oxlint-plugin/no-restricted-syntax/README.md` (lines 60,
139,
204).
The cause is NOT established beyond that:
no oxlint source has been read for it.
Do not attribute it to the oxlint maintainers and do not file anything upstream on this evidence.
Tracked for the user to fix properly in `#442`.

## Standing instruction: fixtures are cat-themed invention, and adapting is not inventing

**From the user,
2026-08-17**,
on a fixture I wrote for the rendering audit:
"Ensure everything is cat-themed.
Cats don't hand over letters to Li Ming by hand."

The fixture in question was not corpus content,
and that was checked rather than assumed:
`git grep`
against the corpus clone at the pinned commit (`~/one-among-us/data`,
`a41fc607`) reports zero files
for `没有离开`,
`三封信`,
`在窗台上`,
`第二天把三封信`,
`她没有离开`,
`睡了一整天`,
and zero for the
English side (`gave three letters`,
`the windowsill`,
`slept on the windowsill`).
The single hit anywhere was `交给了`,
an ordinary verb phrase,
in an unrelated sentence.
Nothing had been committed either:
the file was untracked when the question was asked.

WHAT WAS ACTUALLY WRONG is worth keeping,
because it is subtler than a licensing slip.
The sentence was ADAPTED from the fixture written into task `#85` by an earlier review,
with the
place and person swapped,
rather than invented here.
An inherited skeleton is weaker than the standard even when it carries no corpus text,
and it is how
a non-cat example survives three rewrites.
Write fixtures;
do not adapt them.

THE CHECK IS CHEAP,
so run it rather than reasoning about it:
`git grep --fixed-strings --count` for
each distinctive span in the corpus clone,
before a fixture is committed and before one is sent to
any model.

WHERE CORPUS TEXT MAY GO,
unchanged:
to the production provider,
which is ZDR and retains no
content.
Not into the repository,
and not into any other external tool.
The sol reviews in this
handover were four source files with no corpus content and no fixtures beyond what those files hold.

## Standing instruction: the accumulation bar is six, and only the user restores fifteen

### THE ACCUMULATION BAR IS NOW SIX, EVERYWHERE, AND IT IS PROVISIONAL

Lowered by the user 2026-08-17 from the "15+" set on 2026-08-14,
in their words low but defensible
at this early stage,
to be flipped back to 15 "when we're more confident".

RESTORING IT TO 15 IS THE USER'S CALL AND NOBODY ELSE'S,
and the user said so explicitly when this
was first written down as "the trigger is confidence":
confidence will need their approval AFTER an
agent already believes it is confident.
Two steps,
in that order.
An agent reaching its own
threshold is the moment to ASK,
never the moment to act,
and "we're more confident now" is not a
reason to raise the bar unasked.
Nobody should read a six-entry result as having met the older bar
either.

It lives in prose only;
there is no constant.
`#60` carries the authoritative statement and the
reason.
Older passages in `doc/handover/translation-repair.md` still reason about fifteen (bands per
draw,
hours to accumulate,
when a draw stops reproducing);
those are correct FOR FIFTEEN and are
kept as the record of what was computed then.
Read them as superseded arithmetic rather than as the
current bar.

WHAT SIX BUYS,
stated plainly so it is not read as more:
it unblocks `#60`'s redraw under the new
bar,
and it is a real step for `#115`,
whose confound was two entries against two.
It does NOT make
claims about particular entries safe,
and it does not separate an effect from an entry,
which is the
confound `#115` measured and could not break at two.
