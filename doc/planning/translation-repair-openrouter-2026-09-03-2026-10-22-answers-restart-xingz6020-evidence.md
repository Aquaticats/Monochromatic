# OpenRouter and the corpus passes: the evidence of XingZ6020 on build 1d46232457 (2026-10-22 to 2026-10-23)

The refusals,
errors,
lost voices and findings that are not refusals in the log of `XingZ6020`,
the figures of the brief of the read that were checked,
what the check found that the first reader missed,
and the points no log settles.
Counts are the first reader's,
and the check's recount is named where it differs.
The log,
the providers and the meters are in
[the run of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md);
the timeline and the page are in
[the stages of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-stages.md);
the defects and their fixes are in
[the defects of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).
Times are UTC and every clock time carries its zone;
line numbers are those of the log.

## Control lines

- `TALLY`:
  2 (line 28091 `status=ERROR`,
  line 39940 `status=SETTLED`).
  The `TALLY` lines carry no timestamp.
- `REATTEMPT`:
  1 (line 28092,
  310 more cache records than the attempt began with).
- `OVERLAP`:
  2 (lines 200 and 28093,
  both `value=4 source=fallback`).
- `STALLED`: 0.
  `INCOMPLETE`: 0.
  `[error]` lines: 0.
  No `park`,
  no backoff line and no `window full` line (all `rg --count`,
  zero).
  `DEFECTS XingZ60 checks=destinations` (line 39942).
- The check counted the levels of the log:
  `info` 27,010,
  `warn` 12,708 and 239 lines with no level (the control lines),
  so `[error]` is 0.
  It found no `TALLY`,
  `REATTEMPT`,
  `OVERLAP`,
  `STALLED`,
  `INCOMPLETE` or `[error]` line that the read did not account for.

## Seat refusals

"No provider can take ...":
7,125 lines,
7,119 parsed.
All have the text `every provider serving this model is out of budget, seat unreachable`.

- Structural,
  5,922:
  models served only by Hyper,
  OpenRouter or Bedrock (counts in the run file under the providers that were not reached).
- Synthetic seats after the hold,
  1,197:
  Kimi 666 and Qwen 531,
  by stage in the run file under the cost of the rename.
  The check's `rg --count` agrees,
  and the first is line 19974.
- By stage over all 7,119 (`seatrefusals.ts`):
  select 3,277;
  translate 1,078;
  panel 916;
  produceConsolidations 431;
  lane-contest 378;
  critic 352;
  coverage 303;
  block-pairing 120;
  editor 75;
  refiner 65;
  consolidate-gate 45;
  absolute-naturalness-review 42;
  reference-attest 10;
  section-pairing 10;
  consolidation-polish-gate 6;
  page-consistency 6;
  page-title-lexicon 5.
- Bench-short closes:
  `select: bench short of quorum` 633;
  `panel` 229;
  `translate` 188;
  `critic` 88;
  `produceConsolidations` 63;
  `lane-contest: closed on a short bench` 63;
  `coverage` 60;
  `block-pairing` 24;
  `consolidate-gate` 8;
  `absolute-naturalness-review` 7 (`warns.ts`).

## Voice lost

141 lines (`voicelost.ts`):

- Abandoned after quorum:
  132 (Qwen 131,
  DeepSeek 1).
- `schema-mismatch`:
  9 (DeepSeek 6,
  GLM 2,
  Qwen 1),
  most of them `finish_reason=length` with `raw=""`.

## Wire and transport

- Synthetic HTTP 429:
  1,975 refusal replies (the retry lines and the final warns are in the run file under Synthetic);
  Kimi HTTP 500:
  1 (line 15111).
- NeuralWatt `quota` HTTP 429:
  10 retries.
- Transport failures:
  4 Qwen stream cuts (`refused by TypeError`),
  1 Kimi network refusal,
  1 Kimi stall abort;
  one `synthetic hf:moonshotai/Kimi-K3: no content or reasoning for Nms during body ... aborting for retry`
  (line 16311).

## Pipeline findings that are not refusals

From `warns.ts`:

- Publication-rule failures on candidates:
  22 `candidate fails the publication rule`,
  3 `revision still fails`,
  3 `withheld ... from the judges`,
  5 `ineligible-standing-withheld`,
  5 `translate incumbent excluded by deterministic source floor`.
- `translate-archive-disputed` 64 lines (the same 32 slices in each attempt).
- `introduced repetition` 16;
  `straightenProseQuotes ... refused by the page grammar` 17;
  `challenge round declined` 8.
- `content lost at slice N` 118 info lines (59 per attempt):
  distinctive archive words the repaired document no longer carries,
  one line per slice.
- Page assembly:
  36 findings,
  32 slices trimmed,
  1 withdrawn (slice 44).
- `refine-rolled-back` 3 and `refine-rolled-back-by-probe` 3 (D4 in the defects file).
- Deprecated markers:
  8 `stripped stub marker` lines (4 per attempt).

## What the check found that the read missed

All minor.
The check's warn census (228 distinct messages) was compared with the read's list.

- 58 warns `slice N: consolidation standing text lacks contest endorsement and remains retryable`
  (the first at line 34798) and 23 `final-selection-unendorsed` lines.
  The read counts the consolidation outcomes but not that 58 slices are left retryable rather than final.
  Ledger B831 made the `TALLY` line count them (`standingUnendorsed=`).
- `the aligner refused sections on this document` (line 267,
  twice) and the artifact's `section-pairing paired 13 of 15`;
  the read covers only block pairing.
  The structure and section-pairing findings reached the artifact and not the log (ledger B830).
- `chunk N: N non-translation votes stand; proceeding, votes carried as evidence` (line 11899,
  twice).
- The 3.6 min wait between 03:17:18 UTC (line 32811) and the first contest-tagged line 32824 (03:20:57 UTC),
  and the 4.7 min wait between 04:30:16 UTC (line 34785) and 04:34:58 UTC:
  both inside the read's stage windows and counted as idle,
  but not named as the first `JUDGE SEATS` waits of those phases.
- Outside the six items the check was asked about,
  it did not check the critic claim tallies,
  the tense and pronoun readings,
  the first-content timings per model,
  or the per-model `SEAT` census beyond the 1,197 Synthetic refusals.
  Those figures are first reader only.

## Figures from the brief of the read that were checked

- Confirmed:
  `START` at line 196 with build `1d46232457` (tip `1d462324574cdd651ffaf3ba9b7784c3e2ee96d4`),
  frozen pipeline `sha256-tree-v1:44ce3051...`,
  entry `XingZ60`,
  one file pending;
  39,957 lines;
  11.4 MB;
  10 h 33 min of wall clock;
  the handover's attempt 2 (`attempts.json` and line 28092);
  NeuralWatt's 550 Wh day started at 19:15:08 UTC on 2026-10-22;
  the Synthetic rename window (last readable 22:20:54 UTC,
  first unreadable 22:21:58 UTC);
  and that this build still refused every quota body after it.
- Corrected:
  "routed Synthetic as spendable while its weekly credit was spent" holds only from 00:28:47 UTC on 2026-10-23,
  not from the rename.
  For the 2 h 7 min between them the credit was not spent (34.3 percent at the last reading).

## Could not be determined

- The Synthetic weekly percentage after 22:20:54 UTC,
  and the 429 body:
  neither is in the log.
  "Credit spent from 00:28:47 UTC" is read from the 269-character reply,
  the refusals and the three service windows.
- Whether the 135 Qwen cut streams drew on the weekly credit,
  and by how much.
- Whether slices 26 and 32 were rolled back on a split ballot like slice 4.
  The check did not open `tally.resolved` and so did not state the minimum ballots a "resolved" tally needs.
- Whether `SETTLED` with `translateStatus=unfilled` is meant to be publishable.
  The check answered that the owner's rule ships an unfilled passage as a recorded gap and that the premise was not
  examined for a tail of this size.
- What the translate lane could have written for slices 88 to 117 on a healthy bench:
  nothing in this run reached one.
- The quality of the 63 contested slices beyond the examples read:
  no page-wide line-by-line comparison was made.
