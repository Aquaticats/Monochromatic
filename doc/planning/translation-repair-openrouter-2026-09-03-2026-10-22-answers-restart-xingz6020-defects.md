# OpenRouter and the corpus passes: the defects of XingZ6020 and the commit that fixed each (2026-10-22 to 2026-10-25)

The nine findings of the read of `XingZ6020` (S1,
E2 and D1 to D7) with the second reader's verdict on each,
the ledger entries and commits that fixed them,
whether each fix is on `main` and in the build of `XingZ6021`,
and what `XingZ6021` showed of each.
The log,
the providers and the cost of the Synthetic hold are in
[the run of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md);
the timeline and the page are in
[the stages of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-stages.md);
the refusals and the points no log settles are in
[the evidence of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-evidence.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).
The run after this one,
on the build that carries these fixes,
is read in
[the stages of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-stages.md).

Paths are at the run's tip `1d462324574c` unless a path is of the ledger,
which is read on `main` at `7a61c5e19e`.
Times are UTC and every clock time carries its zone;
line numbers are of the log.
None of the findings was fixed in the run's own build.

## How the fixes are stated here

- Each hash is a commit of this repository.
  "On `main`" means the commit is an ancestor of `origin/main` at `7a61c5e19e`,
  and "in the build of `XingZ6021`" means it is an ancestor of the frozen build `bcff23b8ce`;
  both were measured with `git merge-base --is-ancestor` when this record was filed.
  Every hash named as a fix or a red case here passed both.
- Two ledger sentences are out of date on this point and are not followed.
  Ledger B827 says its shipped tree "has not been landed on `main` at the time of writing";
  `4452f464b4` and `5585eeddd1` are on `main` and in the build of `XingZ6021`.
  The troubleshooting record of the Synthetic weekly block says its fix commits "are not on `main` yet";
  `0204b7c8ba`,
  which merged them,
  is on `main` and in the build of `XingZ6021`.
- The lead's briefs named the rounds of this work D1 to D4,
  X1 and E2.
  Those labels are the briefs' and are not the read's:
  the handover calls the brief's D4 model-level dryness (ledger B822),
  while the read's D4 is the rollback with no log line.
  This record uses the read's labels throughout and cites ledger ids and hashes for the rest.
  Of the fix commits named here only the merge `9eea0009d1` (the fix rounds 1 to 9 of the briefs' D4 and E2) is an
  ancestor of `a0b38f5a45`,
  the merge the briefs name for the D4 and E2 round;
  the others are not,
  and are ancestors of `bcff23b8ce`,
  the merge the briefs name for the D1 to D3 and X1 round.
- The ledger numbers moved once.
  The entries now numbered B833 to B836 were first written as B827 to B830 and were renumbered by `2a8514d7fa`
  on 2026-10-23,
  so a commit message from before it uses the old numbers
  (`5b64a0ad3b` files the short-bench ceiling,
  the unasked rounds and the NeuralWatt means as B827 to B830),
  while B827 to B831 now name the entries of the page-assembly ledger that `f3193b1acb` filed.
  This record uses the ledger's present numbers.

## The findings and their fixes

### S1: the renamed Synthetic weekly block read as unreadable

Check:
confirmed,
known.

- The read:
  the build read the renamed Synthetic weekly block as unreadable and routed Synthetic as spendable.
  Code:
  `provider-stream/src/synthetic-quota.ts:549` to `:558` requires `weeklyTokenLimit`;
  `provider/src/provider-meters.ts:199` to `:208` returns `unreadable` and logs `treating as spendable`;
  `provider/src/provider-budget.ts:827` to `:829` gives a non-wet provider the full 300,000 ms cooldown.
  The cost is in the run file:
  nothing before 00:28:48 UTC;
  after it 395 holds of 300,000 ms,
  228 seat waits,
  249.0 min of idle wall clock and 1,197 Kimi and Qwen seat refusals.
- The check:
  every code reference confirmed.
  The unreadable-as-spendable policy is ledger B671 and B686
  (`package/translation-repair/doc/audit-ledger-follow-ups-emphasis-definitions-cache-accounts-and-synthetic-quota.md`).
  The read's counterfactual,
  that a readable meter would have read dry "with no hold",
  is unconfirmed.
  A readable dry meter is not `wet`,
  so any refusal received inside the 60 s freshness window (`BUDGET_FRESH_MS`) would still have been held for
  300,000 ms and waited for;
  but a dry meter stops the router from sending to Synthetic,
  so only a handful of holds would have followed,
  not 395,
  and not zero.
  This is code reading and not a run.
- The fix:
  the parser reads the weekly block under both names and states a spent budget's return.
  `a4a53c010d` (the first fix),
  `c388130106` (instant validation,
  `nextRegenPercent` ignored,
  a null weekly key read as absent),
  `90a480dcea` (a spent window kept when its return is no instant;
  red `61528e1180`),
  `bc367f792f` (two blocks that agree on the percent and differ in the return;
  red `4e1db8a35a`) and `9371655ffd` (one warning for such a pair;
  red `218443fc90`),
  merged into `main` by `0204b7c8ba` on 2026-10-23 at 05:51 UTC.
  The record is
  [Synthetic quotas renamed its weekly block](../troubleshooting/synthetic-quota-weekly-block-renamed.md).
  The build of `XingZ6020` predates all of it (`0204b7c8ba` is not an ancestor of `1d46232457`).
- Status:
  fixed,
  on `main`,
  in the build of `XingZ6021`.
  `XingZ6021` read Synthetic's credit throughout:
  102 of 102 `METERS` lines carry `syntheticWeekly=<percent>`,
  there is no `budget unreadable` line,
  and the build read 0 percent as dry at 00:01:55 UTC on 2026-10-25.

### E2: NeuralWatt's known return was not counted as a named return

Check:
E2 was not exercised by this run,
and the check confirmed it:
NeuralWatt read `wet` on 518 of 518 readings,
no hold named it,
and there is no `markRefused neuralwatt` line.

- The finding is older than the run:
  the lead's read of `XingZ6019` found that the seat reader stopped an entry on a writing bench for want of a named
  return although NeuralWatt's day restarted 48 s after the stop.
  It is ledger B826
  (`package/translation-repair/doc/audit-ledger-providers-and-seats-a-run-holds-out.md`).
- The fix:
  the merge `9eea0009d1` of the fix rounds 1 to 9 on 2026-10-23 at 08:05 UTC,
  with rounds 10 and 11 after it (round 10 ends at `e9d79e6f0d`).
  The design is in `doc/decision/translation-repair-direct-providers.md` and the ledger entry.
- Status:
  fixed,
  on `main`,
  in the build of `XingZ6021`.
  `XingZ6021` did not exercise it either:
  NeuralWatt read `wet` on 102 of 102 readings.
  No run has yet exercised E2's fix.

### D1: the seat reader waits for a hold whose end cannot lift the bench

Check:
corrected.
The mechanism is confirmed;
the sweeping claim is not.

- The read:
  `run/src/corpus-run/run-seats-read.ts:380` to `:398` waits `shortest` whenever a bench is short and a hold is
  running,
  and never asks whether the bench would reach its quorum when that hold ended.
  In this run 228 waits ended in a bench of at most three against a quorum of four,
  costing 249.0 min of idle wall clock,
  and "cost a second attempt" when the deadline fell inside a wait (line 28090).
  The read added that a 60 s hold from a wet meter's concurrency limit would be waited for the same way.
- The check:
  the code reading is confirmed
  (`:315` to `:321` returns at once when no bench is short;
  `:322` to `:379` is the no-hold branch;
  `:380` to `:398` waits;
  `:394` to `:431` proceeds after one wait whatever the bench then reads),
  and so are the 228 waits.
  Three corrections:
  - "No wait could have lifted any bench to its quorum" holds for the judge benches only.
    The all-wet readings are `wide 3 of 7, quorum 4` (lines 212 and 727) and `select 3 of 7, quorum 4` (line 19172).
    But 117 of the 228 wait lines (translate lane 112,
    lanes 4,
    page consistency 1) also read `translators 2 of 8 reachable, quorum 4`,
    and the all-wet reading at line 19172 names no translators clause,
    so a returned Synthetic would have lifted the translators from 2 to the quorum.
    The lane contest waits (52,
    `wide 1 of 7`) and the consolidation waits (59,
    `slate 1 of 7; wide 1 of 7`) have no all-wet reading for `slate` in this log,
    so whether they could lift is not shown for `slate`.
    In practice Synthetic refused again after every hold,
    so no wait lifted anything.
    Lifting the translators to quorum would not have changed whether `select` could back a tail candidate
    (a floor of two ballots against one voice),
    which is what held the 30 tail slices unfilled.
  - "Cost a second attempt" is unconfirmed.
    The deadline fell inside a wait at 02:32:42.829 UTC,
    but the lane was then closing tail slices that could not fill,
    and the 7 h budget was already spent on 4 h 39 min of preparation,
    repair and refine plus the translate lane.
    The wait is part of the cause and is not shown to be the cause.
  - The first wait of attempt 2 (lines 28563 to 28566,
    299.1 s,
    from 02:33:16 UTC to 02:38:17 UTC) precedes a repair-lane replay that asked nobody.
  The ledger had nothing for it:
  new in kind,
  with adjacent entries H5
  (`package/translation-repair/doc/audit-ledger-defect-class-history.md`,
  lanes that never re-seat under a hold),
  B826 and B817.
- The fix:
  - `a019032c6d` (2026-10-23 at 01:50 UTC):
    wait only for a hold that would cure a short bench,
    and warn when a bench is seated short.
    It postdates the build of `XingZ6020`.
  - B833,
    the line that states the ceiling a running hold could lift a bench to:
    red `e28fd62214`,
    fix `dc62056a10`.
  - B837,
    a held provider whose meter keeps it dry no longer counts as coming back:
    red `4cf385e4b3`,
    fix `39863c9691` (cases `cc3b7dd3e7`).
  - B838,
    a wait is judged against the floor as well as the quorum:
    red `228ddd405b` and `2a7d699bb2`,
    fix `39863c9691`,
    wording `fe0639334b`,
    and the case that ends the hold with the first wait `0fbf3cd3e9`.
    All are in
    `package/translation-repair/doc/audit-ledger-short-benches-and-the-last-live-seat.md` and
    `package/translation-repair/doc/audit-ledger-short-bench-floors-and-the-waits-that-cross-them.md`.
- Status:
  fixed,
  on `main`,
  in the build of `XingZ6021`.
  `XingZ6021` exercised it only on its branch with no hold:
  every one of 161 `JUDGE SEATS` info lines reads `waited=0ms`,
  and a short bench with a hold running whose end would not lift it never arose.

### D2: tail slices that cannot be filled were tried twice and shipped as a gap

Check:
confirmed in its figures.
The behaviour is known design;
the scale and the repeat purchase are new.

- The read:
  slices 88 to 117 have no archive text,
  so a candidate must be backed by two select ballots
  (`winner was named by 1 ballot against a floor of 2; keeping the fallback`,
  367 lines;
  `no-candidate-backed`,
  130 lines:
  40 before line 28091,
  90 after it,
  of which 30 are the final settle at lines 32774 to 32803).
  The selection bench was one voice (`select round: 1/7 heard`):
  190 select rounds in the tail windows heard one voice (71 in attempt 1,
  119 in attempt 2) and four heard none.
  So 18 tail slices in attempt 1 and 30 in attempt 2 could not close any other way:
  48 slice attempts,
  249 NeuralWatt calls,
  87.0 Wh (58.41 Wh and 28.61 Wh),
  about 64 min of lane time,
  for nothing.
  Unfilled slices are not cached by design,
  so attempt 2 paid again.
  The pass ended `SETTLED` with 31.5 percent of the document missing and no marker on the page.
- The check:
  every figure recounted
  (`SLICE-COST ... exit=unfilled` 18 and 30;
  NeuralWatt calls tagged `[translate slice N]` with N of 88 or more:
  153 calls and 58.414 Wh in attempt 1,
  96 calls and 28.606 Wh in attempt 2;
  249 calls and 87.020 Wh in all;
  25.05 min plus 38.75 min of wall clock,
  which is the read's 64 min).
  One range corrected:
  the first unfilled close of attempt 2 is slice 91 at line 29309,
  so the range is 29309 to 32763.
  Known design:
  `publish-completeness.ts:3` to `:16`,
  ledger A8,
  the "not cached" rule in `translate-slice-buy.ts:169` to `:173` and
  `package/translation-repair/doc/audit-ledger-outcome-rules-and-stage-records.md:150` to `:160`.
  The check's answer to the read's question whether `SETTLED` with `translateStatus=unfilled` is meant to be
  shippable:
  the owner's rule ships an unfilled passage as a recorded gap,
  with the premise unexamined for a tail of this size.
- The fix has four parts:
  - What a shipped page carries for such a slice.
    The owner ruled on 2026-10-23,
    "Source text + note".
    Ledger B827:
    red `5585eeddd1`,
    fix `4452f464b4` (with `3827743a7e`),
    and ledger B832 for the follow-ups that close the holes between the page grammar and the plain parse
    (for example `449bfb369d`:
    a page that carries the original of an untranslated passage parses,
    or says why it ships without it).
    The page prints one note,
    "This passage has not been translated yet.
    The original text follows.",
    before the first slice of each contiguous run,
    and the source text and its footnote definitions after it.
  - A slice that ended unfilled for want of a second ballot is not bought again by a later attempt:
    ledger B834,
    red `60f141495e`,
    fix `d59562881c`.
  - A translate slice on a select bench under its floor buys nothing the first time either:
    ledger B839,
    red `7a091d9428`,
    fix `11227fc801`,
    a repeat keeps what the slice was told `ba9654f88b`,
    and the translate hook reads its bench again while the select reach it hands is under the floor
    (red `9991ee7cd1`,
    fix `2e76c32d0f`).
    The ledger measured the saving on this log at 249 calls and 87.020 Wh,
    and at most about 9 min of wall clock.
  - The artifact says a slice was kept for want of a bench:
    ledger B840,
    generation 21 declared in `93e9e77a82`,
    red `2fb9ef9245`,
    fix `b505b55f4e`.
- Status:
  fixed,
  on `main`,
  in the build of `XingZ6021`.
  `XingZ6021` shows D2 did not recur:
  118 `SLICE-START lane=translate`,
  `attempts.json` reads 1,
  the translate lane closed 118 of 118 `computed`,
  and `pageSilent=0`.
  The note path (`pageSourceShipped`) was not exercised,
  and seven tail slices came within one ballot of it.

### D3: the lane contest and the consolidation ran in full on a bench that could not decide

Check:
confirmed in figures,
but it overlaps D1 and S1 and is not a separate cost.

- The read:
  lane contest 63 rounds,
  63 times `1/7 usable, settled on neither` (lines 32824 to 34784),
  73 min of wall clock,
  63 NeuralWatt calls and 17.5 Wh;
  consolidation 63 slices,
  85.4 min,
  232 NeuralWatt calls,
  55.4 Wh and 20 Synthetic calls,
  `slate 1 of 7 reachable, quorum 4`,
  56 slates that ended `winner named by one judge alone; keeping the incumbent`,
  7 slices changed.
  Together "158 min" for at most 7 changed slices.
- The check:
  counted the same (63 lines,
  first 32912 and last 34784;
  63 calls and 17.50 Wh;
  232 calls and 55.39 Wh and 20 Synthetic calls in lines 34785 to 39831;
  56 `[consolidateDocument]` lines,
  the same phrase appearing 95 more times under `[translateDocument]`;
  the artifact has `laneSelection` 63 of 63 `quorum-not-met` and consolidation `unchanged` 50,
  `incumbent` 6,
  `consolidated` 7).
  Corrections:
  the windows are 77.7 and 85.4 min,
  163.1 min in all and not 158;
  and the read counted 71.4 + 64.2 = 135.6 of those minutes inside S1's 249.0 as well,
  so only about 6.3 min of the contest window and 21.2 min of the consolidation window are not idle.
  The short-bench behaviour does what it says
  (`stage/src/stage-windowed-rounds.ts:110` to `:139`,
  `neededWith` at `:266` to `:272`).
  The ledger had no entry for it
  (B816,
  a different symptom,
  was the only hit):
  new as a finding.
- The fix:
  ledger B835.
  The lane contest opens no round for a slice whose reachable bench is under its floor
  (red `a425c3f0ec`,
  fix `6bc22e4160`);
  the consolidation buys no slate for a slice whose reachable slate bench is under its floor
  (red `dc635a867b`,
  fix `51ee45924b`),
  and only for a slice with an eligible standing and no lane text;
  a follow-up makes a reading under the floor with no hold run quietly and never reads a settlement taken under the
  floor from disk (red `a5063fb88e`,
  fix `7e16bcb1bf`);
  the slate-judge count in the log line is `130d025b24`;
  the artifact's record of the skip is B840.
  The ledger measured the rule on this log:
  it skips 54 consolidation slices (164 NeuralWatt calls and 39.746 Wh) and,
  with the contest's 63 rounds,
  227 calls and 57.247 Wh.
- Status:
  fixed for the lane contest and in part for the consolidation,
  on `main`,
  in the build of `XingZ6021`.
  `XingZ6021` was the first measurement:
  the lane contest took 0 rounds and 0.06 s against 73 min and 63 rounds,
  and the consolidation slate skip fired for 10 slices only (88 to 97),
  because it cannot fire when lane texts exist;
  the stage still ran for the rest and took 21.1 min and 383 NeuralWatt calls.

### D4: a winning refinement was rolled back with no log line

Check:
confirmed.
The rollback is known design;
the silence is new.

- The read:
  `repair/src/refine-slice-settle.ts:318` to `:339` returns the pre-rewrite text when `retainsResolvedIssues` says no;
  `repair/src/refine-recheck.ts:334` to `:345` and `:390` to `:399` add only a finding
  `refine-rolled-back (adjudicated/...)`,
  while every other rollback path warns.
  The log says `refinement from X won weight N` (`refine-stage.ts:526`) and then nothing:
  48 such lines,
  42 chunks `refined: true`,
  3 rolled back by the probe (lines 16772,
  17836,
  18066) and 3 by the recheck with no line (slices 4,
  26 and 32;
  win lines 16251,
  17014 and 17262).
- The check:
  confirmed every count and line.
  For slice 4 the ballots on issue `ea4de6864f...` were `not-fixed` (Xiaomi,
  line 16390) and `fixed` (Kimi,
  line 16391).
  Not traced by the check:
  the minimum ballots a "resolved" tally needs.
  The rollback is designed (ledger B141,
  `package/translation-repair/doc/audit-ledger-which-seats-a-quorum-counts.md:180`;
  B161,
  `:321` to `:372`;
  `package/translation-repair/doc/audit-ledger-repair-lane.md:587`);
  no ledger entry records a silent rollback.
- A later ledger finding corrected the read on one point.
  Ledger B829 found that the page's slice 4 sentence is not the rolled-back refinement:
  the artifact's slice 4 has six `refine-rolled-back` findings,
  `chunk.refined` is false,
  `comparison.repairText` equals `chunk.repairedText`,
  and the consolidation ships that text,
  so the page carries the editors' text from before the refinement,
  which is what the rollback restores.
  The rollback took effect,
  and the read's account of the page does not stand:
  the read said slice 4 shipped the wording that two judges had named as added content because the rollback
  discarded a better text,
  while the sentence on the page is the editors' text and not a rolled-back refinement.
- The fix:
  ledger B829,
  red `ed8e3b3131`,
  fix `ee54c874bf`:
  the recheck now writes one warn line naming the slice,
  the rewriting contributors,
  the issue ids and the ballots on each issue.
- Status:
  fixed,
  on `main`,
  in the build of `XingZ6021`.
  Not recurring there:
  all 18 refinements that won each have a warn
  (15 `refine-probe-unheard`,
  2 `refine-recheck-unheard`,
  1 `refine-rolled-back-by-probe`).

### D5: the post-quorum wait on Qwen

Check:
corrected on one figure;
known in kind.
It is a measurement and not a code fault.

- The read:
  the wait after quorum on Qwen was the largest single cost of the repair lane,
  57 percent of the summed slice time (455 of 793 min),
  131 abandoned Qwen calls,
  135 cut streams of 892 calls (15 percent) with 5.04 million reasoning characters and 3,287 content characters,
  none in the `SPEND` ledger,
  and 188 rounds with 100 s or more in grace in attempt 1 (`XingZ6018`:
  2 of 717).
  The ordinary grace of 120 s and the writer grace of 180 s are the owner's decisions.
- The check:
  confirmed the 132 abandoned lines (Qwen 131,
  DeepSeek 1;
  grace 120,000 ms 126 times and 180,000 ms 6 times),
  1,748 Qwen streams of which 135 `cut`,
  and the 188 rounds.
  Corrected the share:
  the summed round `in grace` time over the 88 computed repair slices is 647.6 of 792.8 min (81.7 percent),
  with a median of 409 s per slice against a median slice of 530 s.
  The read's `slicegrace.ts` matched only lines shaped `[repair slice N] <stage> round:` and missed the 229 panel
  rounds (191.5 min of grace);
  the read's own 188-round count includes the panel rounds,
  so its two figures disagreed.
- The ledger:
  B815
  (`package/translation-repair/doc/audit-ledger-providers-and-seats-a-run-holds-out.md`) records Qwen on Synthetic
  being cut after the grace (40 of 95 streams in `XingZ6017`),
  removes it from the lane-contest and consolidation benches
  (`9639e19d85`,
  2026-10-22 at 07:58 UTC),
  and leaves the wide seats with Qwen open to the owner's veto.
  That fix is in the build of `XingZ6020` (`9639e19d85` is an ancestor of `1d46232457`),
  and the run's line 213 `late=7 slate=7` matches it.
  D5 is that open question measured on the wide seats,
  not a new defect.
- Status:
  open to the owner.
  No commit changes the wide seats or the grace.
  `XingZ6021` still saw it:
  34 of 196 Qwen streams cut (17 percent),
  none in `SPEND`,
  five of them in the 2 percent regen window.

### D6: the NeuralWatt energy estimate does not predict the measurement

Check:
confirmed;
known family.

- The read:
  summed estimate 728.779 Wh against 451.711 Wh measured;
  `glm-5.3-flash` 14 times too high;
  `deepseek-v4.1-flash` up to 21 times too low on single calls.
  No effect in this run (`neuralwattReckonedWh` was 0.000 on every reading and 98 Wh were left),
  but near the day limit an estimate that reserves 14 times a call's energy would hold GLM out early.
  Not traced:
  where the estimate is used.
- The check:
  `glm-5.3-flash` 412 calls,
  measured below the estimate on all 412 (395.852 against 28.006 Wh,
  14.1 times);
  `deepseek-v4.1-flash` 1,367 calls,
  above the estimate on 552,
  maximum ratio 21.27;
  all 1,779 estimated 728.779 against 451.711 Wh.
  The "where it is used" question is answered by ledger B808
  (`package/translation-repair/doc/audit-ledger-neuralwatt-replies-cut-at-the-default-effort-and-the-energy-estimate.md:375`
  to `:420`,
  `:560` to `:575`):
  the estimate "feeds the in-flight hold and the ledger line only".
- The fix:
  ledger B836,
  red `f5fb8027be`,
  fix `898445b025`:
  the four buckets with enough calls hold the means of `XingZ6020`
  (DeepSeek 0.3044 and 0.3237 Wh,
  GLM-5.3-Flash 0.0714 and 0.0556 Wh);
  replayed on `XingZ6020` the table holds 451.697 Wh against 451.711 measured.
  The GLM means rest on a build that sends text calls at the `low` effort (B807),
  which `XingZ6020` was the first settled pass to do.
- Status:
  fixed,
  on `main`,
  in the build of `XingZ6021`.
  `XingZ6021` shows a smaller spread (25.416 against 35.363 Wh,
  0.72 times);
  the check of that run did not trace why.

### D7: the page ships with footnote references that have no definitions

Check:
corrected.

- The read:
  five references (page lines 128,
  230,
  516 three times) point at definitions that were in unfilled slices,
  the shipping check names only destinations,
  and "a reference check would have caught it independently".
- The check:
  the five references,
  the absent definitions and `DEFECTS XingZ60 checks=destinations` are confirmed.
  Corrected:
  the page did not introduce these references.
  The archive carries them without definitions
  (`page.en.md` lines 98,
  160 and 334 three times),
  because it stops at line 358 with the stub marker.
  The existing check,
  `assertPageFootnotesIntact` (`page-check/src/corpus-run/page-footnote-integrity.ts:9` to `:17`,
  `:37` to `:74`),
  is relative to the archive by design and could not fire,
  so "a reference check would have caught it" is true only of a check not measured against the archive.
  The ledger had one line,
  `package/translation-repair/doc/audit-ledger-footnote-markers-and-link-destinations.md:462`,
  saying a reference with no definition keeps its markup;
  nothing about shipping dangling references:
  new as a finding.
- The fix:
  ledger B828,
  red `ef0a8407f1`,
  fix `7354f10f1a`.
  `assertPageFootnotesResolve` reads the page alone and names every reference without a definition,
  every definition without a reference and every identifier defined twice.
  It joins the publish checks as `page-footnotes` and reports on the `DEFECTS` line,
  and the page ships (the owner's ruling of 2026-09-27,
  "Ship with the defect").
  Shipping the original with its definitions (D2,
  B827) resolves the five references themselves.
- Status:
  fixed,
  on `main`,
  in the build of `XingZ6021`.
  `XingZ6021` shipped 9 references and 9 definitions,
  none missing or unused,
  and the check did not fire.

## Ledger entries that came from this read and are not a numbered finding

- B830:
  the preparation's alignment findings reached the artifact and not the log
  (the check's omission about `structure-mismatch` and `roster-unpaired`).
  Red `1a494c2086`,
  fix `1fb20af076`.
- B831:
  the `TALLY` line did not count the 58 standings kept without contest endorsement.
  Red `0b54b22597`,
  fix `155a69cb4f`;
  `standingUnendorsed=` counts 56 on the real artifact.
- B832:
  the follow-ups of the source-text decision
  (`package/translation-repair/doc/audit-ledger-source-text-under-a-note-and-the-page-grammar.md`).
- B841:
  the cache-account audit read no record without a word.
  It was found on 2026-10-23 and is recorded in the same round;
  it is not a finding of this read.
  Fix `c49da916d4`.

## What the second reader changed

All applied in this record:

- Item 1:
  `artifacts=1/1` is on line 39943 and not on line 39940;
  `PASS_OVERLAP` is at `pass-overlap.ts:22` and not `:19`.
- Item 2:
  the tail is 5,277 of 16,733 code points from source line 363 and not 5,278 from line 362;
  five dangling references are inherited from the archive;
  the "Three bilibili destinations" sentence is replaced by `source=2 page=1 dropped=1`.
- Item 3:
  the summed wait is 1,030.7 min and not 1,030.8;
  page consistency idle is 4.8 min and not 4.9;
  the Synthetic split of 1,024 and 633 depends on the boundary by one call;
  "with no hold" in the counterfactual is too strong.
- Item 5:
  D1 narrowed and "cost a second attempt" unconfirmed;
  D2's range starts at line 29309;
  D3's windows are 77.7 and 85.4 min and overlap S1;
  D5's grace share is 81.7 percent;
  D7's references are inherited.
- Item 6:
  four omissions,
  listed in the evidence file.

## Points no log settles

- Whether slices 26 and 32 were rolled back on a split ballot like slice 4 (D4).
- The minimum ballots a "resolved" tally needs (D4).
- Whether the 135 Qwen cut streams drew on the Synthetic weekly credit (D5).
- Whether the owner keeps Qwen on the wide seats and the grace values as they are (D5).
- Whether a one-voice stage that cannot reach its floor should be skipped or run once with the result kept as
  evidence,
  for the stages that the fixes do not reach (the consolidation of slices that have lane texts).
