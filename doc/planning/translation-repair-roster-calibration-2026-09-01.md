# Roster calibration under the owner blocklist, 2026-09-01

Evidence record for the seating step of the owner's R3 order
(`doc/decision/translation-repair-roster-blocklist.md`:
"code lands verified,
calibration seats the roster,
four-entry pass,
reading").
Both calibration instruments ran on the nine-model roster at build `48799e6d1`
in the `translation-repair` worktree.
The seating decision itself is recorded separately once the seating-grade editor run lands;
this document carries the numbers that decision cites and the seating rules written before those numbers arrived.

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Roster calibration: the producer and editor standings, reliability and spend](translation-repair-roster-calibration-2026-09-01-calibration-standings-and-reliability.md):
  The nine-model roster measured and how the standings are read;
  the 40-round producer standing and the 6-slice editor and refiner replicate;
  the reliability of the two new admissions,
  the meters and spend,
  and the slice-clustered reading of the replicate.
  Sections:
  - [The roster measured](translation-repair-roster-calibration-2026-09-01-calibration-standings-and-reliability.md#the-roster-measured)
  - [How the numbers are read](translation-repair-roster-calibration-2026-09-01-calibration-standings-and-reliability.md#how-the-numbers-are-read)
  - [Producer calibration, 40 rounds](translation-repair-roster-calibration-2026-09-01-calibration-standings-and-reliability.md#producer-calibration-40-rounds)
  - [Editor calibration, 6-slice default](translation-repair-roster-calibration-2026-09-01-calibration-standings-and-reliability.md#editor-calibration-6-slice-default)
  - [Reliability of the two new admissions](translation-repair-roster-calibration-2026-09-01-calibration-standings-and-reliability.md#reliability-of-the-two-new-admissions)
  - [Meters and spend](translation-repair-roster-calibration-2026-09-01-calibration-standings-and-reliability.md#meters-and-spend)
  - [Slice-clustered reading of the 6-slice replicate](translation-repair-roster-calibration-2026-09-01-calibration-standings-and-reliability.md#slice-clustered-reading-of-the-6-slice-replicate)
- [Roster calibration: the seating rules, the drops made on the owner's authorization and the wall-clock levers](translation-repair-roster-calibration-2026-09-01-seating-rules-drops-and-wall-clock-levers.md):
  The seating rules written before the 40-slice numbers arrived;
  the owner's authorization to drop a model from a role and the drops made on it;
  the wall-clock measurement behind the fast-iteration principle,
  and the stale text the seating edit fixes.
  Sections:
  - [Seating rules, written before the 40-slice numbers arrive](translation-repair-roster-calibration-2026-09-01-seating-rules-drops-and-wall-clock-levers.md#seating-rules-written-before-the-40-slice-numbers-arrive)
  - [Owner authorization of 2026-09-01 and the drops made on it](translation-repair-roster-calibration-2026-09-01-seating-rules-drops-and-wall-clock-levers.md#owner-authorization-of-2026-09-01-and-the-drops-made-on-it)
  - [Wall clock, and the owner's fast-iteration principle](translation-repair-roster-calibration-2026-09-01-seating-rules-drops-and-wall-clock-levers.md#wall-clock-and-the-owners-fast-iteration-principle)
  - [Stale text the seating edit fixes in `run-config.ts`](translation-repair-roster-calibration-2026-09-01-seating-rules-drops-and-wall-clock-levers.md#stale-text-the-seating-edit-fixes-in-run-configts)
- [Roster calibration: the four-entry pass launch and its first failures, 2026-09-01 to 2026-09-02](translation-repair-roster-calibration-2026-09-01-four-entry-pass-launch-and-first-failures-2026-09-01-to-2026-09-02.md):
  The launch form verified before the pass,
  and the first twenty minutes under the 60 second window;
  the pin pass that died of a refusal cooldown and the Carena pass refused at publish time;
  the decision to cut the chain to two entries on a rate measurement.
  Sections:
  - [Before the four-entry pass](translation-repair-roster-calibration-2026-09-01-four-entry-pass-launch-and-first-failures-2026-09-01-to-2026-09-02.md#before-the-four-entry-pass)
  - [The four-entry pass, first twenty minutes](translation-repair-roster-calibration-2026-09-01-four-entry-pass-launch-and-first-failures-2026-09-01-to-2026-09-02.md#the-four-entry-pass-first-twenty-minutes)
  - [The pin pass died of a refusal cooldown, not of budget](translation-repair-roster-calibration-2026-09-01-four-entry-pass-launch-and-first-failures-2026-09-01-to-2026-09-02.md#the-pin-pass-died-of-a-refusal-cooldown-not-of-budget)
  - [The Carena pass finished and was refused at publish time](translation-repair-roster-calibration-2026-09-01-four-entry-pass-launch-and-first-failures-2026-09-01-to-2026-09-02.md#the-carena-pass-finished-and-was-refused-at-publish-time)
  - [The chain was cut to two entries on a rate measurement](translation-repair-roster-calibration-2026-09-01-four-entry-pass-launch-and-first-failures-2026-09-01-to-2026-09-02.md#the-chain-was-cut-to-two-entries-on-a-rate-measurement)
- [Roster calibration: the Toka_ls relaunch and the advisor review, 2026-09-02 from 04:00 to 11:00 UTC](translation-repair-roster-calibration-2026-09-01-toka-ls-relaunch-and-advisor-review-2026-09-02.md):
  The next-action status of that morning,
  with the Toka_ls relaunch killed in consolidation and its rerun that ended incomplete on the naturalness floor;
  the advisor review at 09:30 UTC with its five fixes and six guard-failure proofs.
  Sections:
  - [Next action](translation-repair-roster-calibration-2026-09-01-toka-ls-relaunch-and-advisor-review-2026-09-02.md#next-action)
  - [The Toka_ls relaunch was killed at 77 minutes, in consolidation](translation-repair-roster-calibration-2026-09-01-toka-ls-relaunch-and-advisor-review-2026-09-02.md#the-toka_ls-relaunch-was-killed-at-77-minutes-in-consolidation)
  - [The Toka_ls rerun ended INCOMPLETE at 117 minutes, on a guard the no-loop design had superseded](translation-repair-roster-calibration-2026-09-01-toka-ls-relaunch-and-advisor-review-2026-09-02.md#the-toka_ls-rerun-ended-incomplete-at-117-minutes-on-a-guard-the-no-loop-design-had-superseded)
  - [Advisor review at 09:30 UTC: six proofs and five fixes before the chain](translation-repair-roster-calibration-2026-09-01-toka-ls-relaunch-and-advisor-review-2026-09-02.md#advisor-review-at-0930-utc-six-proofs-and-five-fixes-before-the-chain)
- [Roster calibration: the Carena0442, keyword233 and Toka_ls pages that landed on 2026-09-02](translation-repair-roster-calibration-2026-09-01-entries-landed-2026-09-02.md):
  Carena0442 landing at 13:48 UTC with the escaped-quote defect found in its reading;
  keyword233 landing at 14:13 UTC and Toka_ls landing at 15:52 UTC;
  the cause-of-death decisions and the first XIEPT2 launches refused by the runs-directory guards.
  Sections:
  - [Carena0442 landed at 13:48 UTC, and the two waiters were killed at the same moment](translation-repair-roster-calibration-2026-09-01-entries-landed-2026-09-02.md#carena0442-landed-at-1348-utc-and-the-two-waiters-were-killed-at-the-same-moment)
  - [keyword233 landed at 14:13 UTC in 22 minutes, on the fixed tree](translation-repair-roster-calibration-2026-09-01-entries-landed-2026-09-02.md#keyword233-landed-at-1413-utc-in-22-minutes-on-the-fixed-tree)
  - [Toka_ls landed at 15:52 UTC in 100 minutes at overlap 3](translation-repair-roster-calibration-2026-09-01-entries-landed-2026-09-02.md#toka_ls-landed-at-1552-utc-in-100-minutes-at-overlap-3)
- [Roster calibration: the XIEPT2 launches, 2026-09-02 night to 04:14 UTC on 2026-09-03](translation-repair-roster-calibration-2026-09-01-xiept2-launches-2026-09-02-night-to-2026-09-03.md):
  XIEPT2 refused whole over one unfilled passage;
  the run that exhausted Synthetic's week and the 429 hold that stalled Hyper;
  the request-rate limit that led to paced calls,
  and the landing at 04:14 UTC on the paced client.
  Sections:
  - [XIEPT2 ran 35 minutes at overlap 4 and was refused whole over one unfilled passage](translation-repair-roster-calibration-2026-09-01-xiept2-launches-2026-09-02-night-to-2026-09-03.md#xiept2-ran-35-minutes-at-overlap-4-and-was-refused-whole-over-one-unfilled-passage)
  - [XIEPT2's fourth launch ran Synthetic's week dry, and the 429 hold then stalled Hyper](translation-repair-roster-calibration-2026-09-01-xiept2-launches-2026-09-02-night-to-2026-09-03.md#xiept2s-fourth-launch-ran-synthetics-week-dry-and-the-429-hold-then-stalled-hyper)
  - [XIEPT2's fifth attempt hit Hyper's request-rate limit with no hold at all, so calls are paced now](translation-repair-roster-calibration-2026-09-01-xiept2-launches-2026-09-02-night-to-2026-09-03.md#xiept2s-fifth-attempt-hit-hypers-request-rate-limit-with-no-hold-at-all-so-calls-are-paced-now)
  - [XIEPT2 landed at 04:14 UTC in 129 minutes on the paced client](translation-repair-roster-calibration-2026-09-01-xiept2-launches-2026-09-02-night-to-2026-09-03.md#xiept2-landed-at-0414-utc-in-129-minutes-on-the-paced-client)
- [Roster calibration: the six-seat bench smoke run and seating judges by provider reach, 2026-09-03](translation-repair-roster-calibration-2026-09-01-six-seat-bench-and-judge-seating-by-provider-2026-09-03.md):
  The keyword233 smoke run of the six-seat bench on HEAD,
  with the wrap and damage-claim fixes that followed it;
  the decision to seat the Hyper-slow judge per provider,
  superseding the morning's drop,
  and its live witness;
  the XIEPT2 postscript run,
  where reading the meter once per entry cost two hours,
  and Kimi-K3 measured per role.
  Sections:
  - [keyword233 smoke-tested the six-seat bench on HEAD at 04:44 UTC in 19 minutes](translation-repair-roster-calibration-2026-09-01-six-seat-bench-and-judge-seating-by-provider-2026-09-03.md#keyword233-smoke-tested-the-six-seat-bench-on-head-at-0444-utc-in-19-minutes)
  - [The Hyper-slow judge is seated per provider, superseding the morning's drop](translation-repair-roster-calibration-2026-09-01-six-seat-bench-and-judge-seating-by-provider-2026-09-03.md#the-hyper-slow-judge-is-seated-per-provider-superseding-the-mornings-drop)
  - [XIEPT2 on the postscript rule: the page says it one way, and the seat rule's blind spot cost two hours](translation-repair-roster-calibration-2026-09-01-six-seat-bench-and-judge-seating-by-provider-2026-09-03.md#xiept2-on-the-postscript-rule-the-page-says-it-one-way-and-the-seat-rules-blind-spot-cost-two-hours)
  - [Kimi-K3 measured per role: select-slow on Hyper, fine elsewhere, fine on Synthetic](translation-repair-roster-calibration-2026-09-01-six-seat-bench-and-judge-seating-by-provider-2026-09-03.md#kimi-k3-measured-per-role-select-slow-on-hyper-fine-elsewhere-fine-on-synthetic)
- [Roster calibration: the stub marker strip and the damage probe's precision, 2026-09-03](translation-repair-roster-calibration-2026-09-01-stub-marker-and-damage-probe-2026-09-03.md):
  The owner's three answers and the stub marker traced to archive text no slice covers,
  then stripped;
  the stub-fix XIEPT2 page that landed at 15:28 UTC;
  the damage probe measured at six true claims in ten,
  so the contest is shown them as evidence and nothing obeys them.
  Sections:
  - [The owner's three answers, and the stub marker traced and stripped](translation-repair-roster-calibration-2026-09-01-stub-marker-and-damage-probe-2026-09-03.md#the-owners-three-answers-and-the-stub-marker-traced-and-stripped)
  - [The probe measured: six true claims in ten, so the contest is shown them and nothing obeys them](translation-repair-roster-calibration-2026-09-01-stub-marker-and-damage-probe-2026-09-03.md#the-probe-measured-six-true-claims-in-ten-so-the-contest-is-shown-them-and-nothing-obeys-them)

## Status

-   `producer-calibrate` over 40 rounds:
    complete,
    standings recorded here.
-   `editor-calibrate` at its 6-slice default:
    complete,
    recorded here as a same-day replicate,
    not as seating evidence (the repo's own record calls a standing this small noise).
-   `editor-calibrate` over 40 slices:
    complete,
    21:44Z to 01:02Z (11908 s),
    four slices in flight,
    300000 ms straggler window;
    111 editor rounds from 30 slices,
    25 refiner rounds from 25 slices.
    Log at `~/temp/agent/editor-calibrate-40-20260901.log`.
    Seated on it:
    `doc/decision/translation-repair-roster-seating-2026-09-01.md`
    (editors GLM-5.3-Flash,
    glm-5.3,
    deepseek-v4-pro-0813;
    refiners GLM-5.3-Flash,
    deepseek-v4-pro-0813,
    minimax-m3;
    checkers unchanged).
    Its Synthetic weekly cost:
    15.5 to 10.4 percent remaining;
    Hyper 6967 to 6636 credits.
-   Logs of the two finished runs:
    `~/temp/agent/producer-calibrate-40-20260901.log`
    and `~/temp/agent/editor-calibrate-6-20260901.log`.
-   Landed at `03a58d2ca` on the owner's role-drop authorization:
    `RUN_TRANSLATORS` (seven,
    without
    `gpt-oss-120b` and `deepseek-v4-flash-0731`) and `RUN_JUDGES` (eight,
    without `glm-5.3`) in
    `src/corpus-run/run-config.ts`;
    read back from the built chunk `pass-entry` imports:
    roster 9,
    translators 7,
    judges 8 in both lanes,
    critics and panel 9.
-   The standing report now ends with one counts line per slice (`sliceStandingLines`),
    so the next calibration's log supports a bootstrap over slices without a log parser;
    the 40-slice run in flight predates it and is read through the winner lines instead.

## Decisions waiting on the owner

Collected here so a reader of the last section has the whole list;
each item's evidence lives in the
section or issue it names.

- Issue #473:
  recovery rounds for unreadable answers are served from the prompt-uniqueness cache in
  0 to 1 ms and never recover anything.
  Vary the prompt with the guard's complaint,
  or delete the round.
- Issue #474:
  an HTTP 429 (a concurrency limit on Synthetic) is read as a budget refusal and holds the
  provider dry for five minutes whatever its meter says;
  two holds at once end the pass and fail every
  remaining entry in under a second.
  Re-read the meter on 429 and back off briefly,
  or wait out the
  shorter hold,
  or at least state measured facts in the message.
  The fixed minimum vote weight of 2
  (`MIN_SELECTION_WEIGHT`),
  which a 4-of-8 ballot round reaches only when two full-weight ballots agree,
  belongs to the same decision.
- DECIDED 2026-09-02 09:0x UTC:
  the front-matter guard keeps structural checks only (commit `34e5c7ecd`,
  `doc/decision/translation-repair-front-matter-guard.md`).
  The owner's answer to the veto question was to
  its premise ("why are we caring about metadata being different vs Chinese source at all?"):
  the
  2026-08-28 trigger was a proxy for nothing,
  so the incumbent-fallback refusal and the night's standing
  machinery (daaf0ffa0,
  6f70a2085,
  1160ebb4c) are gone;
  the directory-id refusal now fires on the
  assembled page whether or not it equals the archive,
  narrowed by `6d85b619a` to pages whose source
  names the person differently (the census under "Advisor review at 09:30 UTC").
- DECIDED at the same time:
  unseat GLM-5.3-Flash from every judge role,
  keep its editor seat (commit
  `9a7d48354`,
  addendum in the seating decision doc);
  #473 is fixed by varying the recovery prompt with
  the complaint (commit `6323f05d8`,
  `RECOVERY_NUDGE` appended to the recovery round's messages);
  the
  slice 9 pronoun is a judge defect,
  not a policy (the source uses 她 sixteen times;
  commit `21757b86c`
  adds a `pronoun` line to the identity context and reads the house rule off the whole original);
  #474 options 1 and 2 are both implemented (commit `83e8dfa90`:
  a refusal
  re-reads the meter and holds a wet provider for 30 s rather than 300 s;
  a both-dry reading with a hold
  behind it waits out the shorter hold before ending the run).
- keyword233 is in the chain again (378 source characters in 9 blocks,
  the smallest of the four).
  XIEPT2
  is not:
  7365 source characters in 191 blocks against Toka_ls's 1532 in 62,
  about five times the
  source,
  and the publish test records its earlier attempt at four hours forty-eight minutes with no page
  kept.
  Under `FIT` (size runs to end within about an hour) that is the owner's call:
  a run of several
  hours,
  a different overlap,
  or a fresh week.
  Not a design decision,
  so no question is pending;
  the
  measurement is here for the veto.
- LANDED 2026-09-03 04:14 UTC:
  XIEPT2 at overlap 4 on Hyper alone in 129 minutes,
  paced at 1,000
  requests an hour,
  outside `FIT`;
  the pacer is the cost of one provider,
  and a run within the hour on
  Hyper alone needs either Synthetic's week back or a smaller shape (owner's call,
  veto invited).
- Reading findings on XIEPT2 (`doc/planning/translation-repair-xiept2-reading-2026-09-03.md`),
  each a
  policy or placement reading for the owner:
  the postscript's "took medication and breathed in gas"
  where the obituary block dropped both;
  the archive stub's `(To-Do)` placeholder kept as the page's
  first paragraph (fix the pass to drop placeholder-only archive paragraphs,
  or edit the stub);
  glosses
  from the archive's comments inserted inline (大证,
  WER) rather than as footnotes;
  semantic wrap applied
  per slice so the page alternates shapes.
- Provider-aware judge seat (task #33):
  Qwen3.8-27B is unseated from every judge round on Hyper's cut
  record but answered 25 of 28 select rounds on Synthetic;
  Kimi-K3 is the next candidate under the same
  rule (majority of select and contest rounds cut on Hyper,
  under half in critic and panel).
