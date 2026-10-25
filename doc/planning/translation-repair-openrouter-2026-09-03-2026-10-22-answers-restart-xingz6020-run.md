# OpenRouter and the corpus passes: the run of XingZ6020 on build 1d46232457 (2026-10-22 to 2026-10-23)

What was read on 2026-10-23 (UTC) of the run `XingZ6020`,
which came after `XingZ6019` on the same frozen build `1d46232457`:
the log,
the two attempts,
the providers and how the Synthetic weekly credit ran out,
the waits that followed,
and the page that shipped.
Entry:
`XingZ6020` (entry `XingZ60`,
the same 118-slice page that `XingZ6018` stopped on).
Hyper,
OpenRouter and Bedrock were out of the run from its first line,
so the run drew on NeuralWatt,
Synthetic,
Xiaomi and DeepSeek only.

The run settled on its second attempt after 10 h 32 min 51 s,
and the page it shipped is missing the last 30 of its 118 slices
(31.5 percent of the source,
including every footnote definition),
with nothing on the page to say so.
It is the run that drove the D rounds and X1,
and the findings of its read are accounted for one by one,
with the commit that fixed each,
in
[the defects of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md).
The two attempts as a timeline,
the page that shipped,
and what the code does with a slice that ends unfilled are in
[the stages of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-stages.md).
The refusals,
errors,
lost voices,
the omissions of the first reader and the points no log settles are in
[the evidence of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-evidence.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).
The passes before this one on the same entry are in
[the XingZ6019 record](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6019.md);
the pass after it,
on the build that carries the fixes of this read,
is in
[the run of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-run.md).

## How this record was made

A first reader read the log
`corpus-pass-XingZ6020.log` (39,957 lines;
11,421,860 bytes)
and the run directory `corpus-pass-XingZ6020/` beside it,
and quoted the source at the run's tip `1d462324574c`
(`git show 1d462324574cdd651ffaf3ba9b7784c3e2ee96d4:<path>`,
read only).
A second reader,
independent of the first,
then checked the read against the log,
the run directory,
the corpus text and the source as built.
This record is the read as corrected by that check:
where the two differ,
the check's reading is the one filed,
and a figure the check did not recompute is marked as read by the first reader only.

- The read is `temp/agent/tr-fan/w312-xingz20/XingZ6020-read.md` (608 lines)
  and the check is `XingZ6020-read-check.md` beside it (335 lines).
- Line numbers in this record are those of the log.
  Times are UTC;
  every clock time carries its zone.
- Counts come from scripts in `temp/agent/tr-fan/w312-xingz20/` or from `rg --count`,
  and each figure names its method.
  The first reader's scripts are `calls.ts`,
  `summary.ts`,
  `rounds.ts`,
  `quality.ts`,
  `tense.ts`,
  `voicelost.ts`,
  `pace.ts`,
  `slow.ts`,
  `rundir.ts`,
  `meters.ts`
  (copied from the scripts of the `XingZ6018` read with their constants replaced),
  and `census.ts`,
  `phases.ts`,
  `gaps.ts`,
  `idle.ts`,
  `holdwaits.ts`,
  `httpcodes.ts`,
  `synthetic.ts`,
  `energy.ts`,
  `models.ts`,
  `stagetime.ts`,
  `slicegrace.ts`,
  `lanes.ts`,
  `seatrefusals.ts`,
  `warns.ts`,
  `claims.ts`,
  `shippedlane.ts`,
  `at.ts`,
  `artkeys.ts` and `artshape.ts` (new).
  `summary.ts` and `slow.ts` list five meter streams and not `balance`,
  so their `balance` row is not a model (`models.ts` replaces them),
  and `refusals.ts` and `dry.ts` carry the constants of `XingZ6018` and were not used for figures.
  The checker wrote its own scripts (`parse.ts`,
  `idle.ts`,
  `synth.ts`,
  `nw.ts`,
  `tail.ts`,
  `waits.ts`,
  `stages.ts`,
  `cons.ts`,
  `grace.ts`,
  `panel.ts`,
  `refine.ts`,
  `est.ts`,
  `quotas.ts`,
  `page.ts` and others under `check/`),
  and used none of the first reader's scripts or outputs for a figure.
- Every command of both readers was run as `env --ignore-environment PATH=... HOME=... <cmd>`.
  The scripts only read local files:
  the log,
  the run directory,
  the public corpus at the pinned commit (`git show a41fc607ea:<path>`)
  and the source at the run's tip.
  No provider was called,
  no built runner,
  probe or corpus pass was run,
  no `mise` task was run,
  and no `.env*`,
  auth file or `/proc/<pid>/environ` was opened.
- The corpus copies the readers used are public data:
  `source.page.md` and `archive.page.en.md` at the pin,
  and `shipped.page.en.md` from the run directory.

## Headlines

- The run ended `SETTLED` on its second attempt (line 39940),
  after the first attempt ended `ERROR` on the hard deadline of 25,200,000 ms (line 28091).
  `DONE` (line 39943) reads `processed=1 of pending=1; artifacts=1/1`
  (the check placed `artifacts=1/1` on line 39943,
  three lines after the `TALLY`,
  not on line 39940 as the read had it).
  One artifact (`artifacts/XingZ60.json`,
  11,245,948 bytes)
  and one page (`fixed/people/XingZ60/page.en.md`,
  34,637 bytes,
  538 lines)
  were written.
- The page ends part way through the ninth friend's section.
  Thirty of the 118 slices (88 to 117) were never filled.
  They hold 5,277 of the 16,733 source code points
  (31.54 percent;
  the check counted from the start of slice 88 at source line 363,
  and the read's 5,278 counts the blank line before it):
  the rest of that section,
  the tenth section,
  the closing dedication and every footnote definition.
  The run recorded it (`source-passage-unfilled` 30 times,
  lines 32774 to 32803;
  `pageSilent=30` in line 39940) and still closed `SETTLED`.
- NeuralWatt read wet on all 518 meter readings.
  It served 1,779 calls and drew 451.711 Wh of the 550 Wh day;
  98.288 Wh were left at the end.
  No NeuralWatt hold was ever set,
  so E2 did not fire and this run says nothing about its fix.
- Synthetic was the provider that ran out.
  Its weekly block was renamed between 22:20:54 UTC and 22:21:58 UTC on 2026-10-22,
  and the build read it as unreadable from then on (S1).
  Synthetic served 1,024 calls before the rename and 633 after it (`SPEND` lines;
  the split is one call different at a boundary of 22:21:58.000 UTC),
  but from 00:28:48 UTC on 2026-10-23,
  when its weekly credit reached zero,
  it served 48 calls in three short windows and refused the rest (1,975 refusal replies).
- The cost of S1,
  measured by both readers:
  from 00:28:48 UTC to the end,
  249.0 of the 336.8 minutes of wall clock (74 percent) had no model stream in flight,
  and 246.4 of those minutes lay inside the intervals of 228 logged waits for a 300,000 ms Synthetic hold.
- Seven new defect candidates were filed by the read (D1 to D7),
  and the check classed them:
  D1 corrected,
  D2 confirmed but not new in kind,
  D3 confirmed but overlapping D1,
  D4 confirmed with the rollback known and the silence new,
  D5 corrected on a figure and known in kind,
  D6 confirmed and known,
  D7 corrected.
  Each was fixed or accounted for in the build that ran `XingZ6021`;
  the commits are in the defects file.

## Run facts

- The build finished in 9.10 s (line 193).
  `ONLY XingZ60 ...` (line 195) says the entry was chosen by hand for a throwaway runs directory.
- `START` (line 196):
  tip `1d462324574cdd651ffaf3ba9b7784c3e2ee96d4`,
  pipeline `sha256-tree-v1:44ce3051663ba4db06dc78c8b3e6717f3569be8d5fb4ab69af92eff99b24e80c`,
  `files=3 pending=1 done=0 soft=259200000ms hard=25200000ms`.
  The artifact's `corpusSha` is `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`.
- `PROVIDER OFF bedrock` (line 198);
  the writer grace of 180,000 ms is built in (line 197).
- `OVERLAP XingZ60 value=4 source=fallback` (lines 200 and 28093).
  `source=fallback` is the pass default of four slices in flight per driver when no environment value is set
  (`corpus-pass/src/corpus-run/pass-overlap.ts`,
  `PASS_OVERLAP` at line 22 and the print at lines 71 to 72;
  the read cited line 19 for the constant),
  not a fault.
- The first timestamped line is 19:32:42.829 UTC on 2026-10-22 (line 199);
  `DONE` (line 39943) says `elapsed=37971042ms` (10 h 32 min 51 s);
  `Finished in 37984.15s` (line 39957) is 10 h 33 min 4 s.
  The unit's launch at 19:32:29 UTC fits.
- Run directory `corpus-pass-XingZ6020/` (`rundir.ts`;
  the check recounted the files):
  `attempts.json` reads `{"XingZ60": 2}`;
  `artifacts/` 1 file;
  `fixed/` 1 file;
  `ledger/` 633 files;
  `prompt-payloads/` 3,694 files;
  `slice-cache/` empty.
  The cache is empty by design:
  a settled entry discards its slice cache after the artifact write
  (`corpus-pass/src/corpus-run/pass-entry-caches.ts:116` to `:130`,
  which calls `discardSliceCache` at `slice-cache/src/corpus-run/slice-cache-store.ts:415`).
- Two attempts:
  - Attempt 1:
    lines 199 to 28091,
    19:32:42.8 UTC on 2026-10-22 to 02:32:42.8 UTC on 2026-10-23,
    7 h 0 min.
    `TALLY XingZ60 status=ERROR ms=25200743 aborted=true error=Timeout: XingZ60 exceeded its 25200000ms deadline`
    (line 28091),
    then `REATTEMPT XingZ60 queued: 310 more cache records than it had` (line 28092).
  - Attempt 2:
    lines 28093 to 39943,
    02:32:43.8 UTC to 06:05:33.8 UTC on 2026-10-23,
    3 h 33 min.
    `TALLY XingZ60 status=SETTLED slices=118 repairStatus=repaired repairIssues=389 repairAccepted=254
    repairResolved=250 repairFindings=4761 repairChanged=80 translateStatus=unfilled translateChanged=41
    documentsDiffer=63 pageChanged=61 pageSilent=30 alignmentFindings=45 selection=contested ms=12768973`
    (line 39940),
    `DESTINATIONS XingZ60 source=2 page=1 dropped=1` (line 39941),
    `DEFECTS XingZ60 checks=destinations` (line 39942).
- The entry:
  15 aligned units,
  118 slices,
  45 alignment findings (`TALLY`),
  source 16,733 code points,
  archive 33,185 characters.
  The archive page stops at the ninth friend's section and ends with the stub marker `Under Construction`
  (archive line 358),
  which the pass strips as a placeholder (lines 214 to 217,
  again 28106 to 28109).
- The control lines are counted in the evidence file.

## Meters

518 `METERS` lines (`meters.ts`),
from line 211 (19:32:44.942 UTC) to line 39869 (06:05:32.557 UTC).
The check recounted the NeuralWatt,
Synthetic `wet`,
`unreadable` and `dry` totals with `rg --count` and agrees.

- NeuralWatt `wet` on all 518.
- Hyper `dry` (`hyperBalance=0`) and OpenRouter `dry` (`openrouterUsd=-0.10`) on all 518;
  Bedrock `dry` on all 518 (turned off by the owner,
  line 198).
- Synthetic:
  `wet` 132 times (lines 211 to 12568),
  `unreadable` 209 times and `dry` 177 times from line 12628 on.
  The `dry` readings are the `unreadable` meter with the 300,000 ms hold folded in.
- DeepSeek `wet` throughout (`deepseekUsd` 9.93 to 9.70,
  a drop of 0.23 against 0.278706 booked).
  The booked figure is a peak-rate upper bound,
  because the build priced DeepSeek at peak rates always.
  Ledger B920 (the fix is in progress and is not on `main`) books a call at half when it started outside the weekday
  peak windows of 01:00 to 04:00 UTC and 06:00 to 10:00 UTC,
  so this figure over-reads the calls that started off peak by up to twice.
  The run lay on a Thursday evening and a Friday morning in UTC,
  so some of its 21 booked DeepSeek calls (22 streams) may have started inside the Friday windows and some outside;
  their start times were not examined for this record,
  and how much the figure over-reads is not computed.
  Note of 2026-10-25 (UTC), added with the third round of the fix, which merges `main` into its branch:
  the rule as built books a call at half only when both its start and its reply's completion lie outside every window,
  and at peak when either lies inside one, so "when it started outside" above is the first round's reading.
  The entry is in `package/translation-repair/doc/audit-ledger-deepseek-off-peak-pricing.md`.
- Xiaomi `wet` throughout (`xiaomiUsdSpent` 0.0000 to 0.0975,
  booked 0.097514).

## NeuralWatt against the 550 Wh day

- Start `neuralwattWh=550.000`,
  `neuralwattAccountWh=0.000` (line 211);
  end `98.288` and `451.712` (line 39869).
  The summed `NeuralWatt energy` lines give 451.711 Wh over 1,779 calls (`energy.ts`),
  within a thousandth of the account meter.
  The check recounted the 1,779 lines,
  the 451.711 Wh and the 1,779 `SPEND provider=neuralwatt` lines (cost sum 2.497789),
  and the minimum remaining of 98.288.
  `neuralwattReckonedWh=0.000` on every reading.
- By model:
  `deepseek-v4.1-flash` 1,367 calls,
  423.705 Wh;
  `glm-5.3-flash` 412 calls,
  28.006 Wh.
- By hour (UTC,
  Wh):
  19h 18.0 (110 calls),
  20h 53.5,
  21h 51.9,
  22h 46.7,
  23h 32.6,
  00h 41.9,
  01h 43.6,
  02h 62.3,
  03h 38.5,
  04h 21.2,
  05h 40.7,
  06h 0.8.
- Remaining at selected readings:
  412.762 (line 12628,
  22:22:00 UTC),
  326.242 (line 19933,
  00:28:48 UTC),
  258.219 (line 25560,
  02:03:43 UTC),
  200.301 (line 28078,
  02:31:04 UTC),
  171.199 (line 32813,
  03:20:56 UTC),
  142.896 (line 36029,
  04:56 UTC),
  98.519 (line 39797,
  06:00:14 UTC).
- `NeuralWatt plan energy: no plan energy was spent between the start and the end of the run` (line 39945);
  the baseline was `6.5731 kWh charged this period` (line 204).
  So the 451.7 Wh were all free-day energy and the plan figure did not move.
- The booked `cost=` fields total 2.497789 (`deepseek-v4.1-flash` 2.328561,
  `glm-5.3-flash` 0.169228).
  The log does not say whether these are billed;
  the plan energy line says no plan energy was spent.
- The estimate beside each measurement was far off in both directions (`energy.ts`;
  the check recounted it in `est.ts`):
  summed estimate 728.779 Wh against 451.711 Wh measured;
  `glm-5.3-flash` estimated 395.852 Wh and measured 28.006 Wh (14.1 times,
  below the estimate on all 412 calls);
  `deepseek-v4.1-flash` measured above the estimate on 552 of 1,367 calls,
  by up to 21.27 times.
  That is D6 in the defects file.
- 87.0 Wh (19 percent of the day's draw) went to the 48 tail-slice attempts that produced nothing
  (D2;
  the check recounted 249 calls and 87.020 Wh,
  58.414 Wh in attempt 1 and 28.606 Wh in attempt 2).
- Ten NeuralWatt `quota: HTTP 429; retrying` lines (the first at line 20287) belong to the quota meter endpoint
  and not to chat calls,
  and set no hold.

## Synthetic

Models:
`hf:moonshotai/Kimi-K3` and `hf:Qwen/Qwen3.8-27B`.
Synthetic has no `cost=` field.
Tokens from `SPEND` (`synthetic.ts`):
Kimi 899 calls,
prompt 17,577,267,
completion 695,319;
Qwen 758 calls,
prompt 15,452,278,
completion 2,588,936.
That is 1,657 `SPEND` lines in all.
A stream whose reply body is under 400 characters and carries no content is counted as a refusal reply;
every one of the 1,976 was 144 raw characters except one that was empty (the Kimi HTTP 500,
line 15111).

- Kimi:
  2,021 streams,
  899 served,
  1,121 refusals (1,120 HTTP 429 and one 500),
  1 cut.
  First content (900 streams with content):
  median 2,728 ms,
  p90 3,746 ms,
  p99 6,013 ms,
  maximum 73,116 ms (line 835,
  in preparation).
  Longest elapsed 177,222 ms,
  the `cut` stream (line 16312,
  `Stalled ... emitted no content or reasoning for 120000ms`).
- Qwen:
  1,748 streams,
  757 served with content,
  855 refusals,
  135 `cut`,
  1 other zero-content.
  First content (893):
  median 3,600 ms,
  p90 4,641 ms,
  p99 6,344 ms,
  maximum 17,603 ms (line 15178).
  Longest elapsed 350,268 ms (line 20013).
  The 135 cut streams ran 21,392 s in all (5.9 h summed),
  wrote 5,043,928 reasoning characters and 3,287 content characters,
  and have no `SPEND` line,
  so their tokens are not in the ledger.
  Of the cuts 78 came before the rename window and 57 after it.
- By UTC hour (served,
  refused and cut),
  Kimi then Qwen:
  19h 63/0/0 and 54/0/7;
  20h 195/0/0 and 168/0/33;
  21h 218/0/0 and 191/0/26;
  22h 191/1/0 and 167/0/23;
  23h 136/0/1 and 107/0/28;
  00h 64/105/0 and 56/105/11;
  01h 14/185/0 and 12/185/2;
  02h 0/215/0 and 0/230/0;
  03h 0/205/0 and 0/50/0;
  04h 0/210/0 and 0/100/0;
  05h 18/189/0 and 2/179/5;
  06h 0/11/0 and 0/6/0.
- After 00:28:48 UTC Synthetic served 48 calls in three windows
  (the check's line ranges):
  2 from 00:29:07 UTC to 00:29:29 UTC (lines 19988 to 20014),
  26 from 01:43:05 UTC to 01:50:47 UTC (lines 24495 to 24852),
  and 20 from 05:06:59 UTC to 05:12:10 UTC (lines 36675 to 37002).
  The last `SPEND` was line 37002 at 05:12:10 UTC.
  The `/quotas` reply measured 284 to 286 characters again in two stretches only
  (lines 24522 to 24684 and lines 36614 to 36890),
  and 269 characters otherwise;
  the first reader's wall-clock bounds for those stretches were
  01:45:38 UTC to 01:50:20 UTC and 05:06:50 UTC to 05:12:11 UTC.
- Retry ladders (`httpcodes.ts`):
  1,591 retry lines,
  1,580 of them Synthetic 429 (Kimi 896,
  Qwen 684),
  summing 4,450 s of retry delay across concurrent calls (median delay 1,982 ms,
  maximum 7,993 ms),
  and 395 ladders ending in the final warn `<- model: HTTP 429` (Kimi 224,
  Qwen 171);
  1 Kimi HTTP 500 (a 36.5 s call with no byte,
  line 15110);
  10 NeuralWatt `quota` HTTP 429 retries (7.0 s);
  4 Qwen `stream cut ... (refused by TypeError)` retries between 20:27 UTC and 20:43 UTC
  (lines 3583,
  3820,
  3822 and 4848);
  1 Kimi network refusal (line 3988);
  1 Kimi stall abort (lines 16311 to 16313).
  The check counted the lines that contain `HTTP 429` at 1,985,
  which is the 1,590 retry lines (1,580 Synthetic and 10 NeuralWatt) and the 395 final warns,
  with no `HTTP 429` before line 19939 (00:28:48.330 UTC) and the last at line 39860.

## NeuralWatt models

- `deepseek-v4.1-flash`:
  1,367 streams,
  1,347 with content,
  20 with reasoning only and no content.
  First content median 1,403 ms,
  p90 2,261 ms,
  p99 3,531 ms,
  maximum 10,454 ms (line 21752).
  Longest elapsed 83,644 ms (line 27047).
  30 `schema-mismatch on neuralwatt; re-asking another provider` lines:
  23 answered usably on the re-ask (`deepseek-flash`,
  the DeepSeek API),
  6 failed again,
  1 other;
  the failure lines that record a cause name `finish_reason=length` with `raw=""`
  (for example lines 26558 and 29899).
- `glm-5.3-flash`:
  412 streams,
  all with content.
  First content median 1,968 ms,
  p90 2,987 ms,
  p99 10,272 ms,
  maximum 11,059 ms (line 20954).
  Longest elapsed 17,853 ms.

## Xiaomi and DeepSeek

- `mimo-v2.6-flash` (Xiaomi):
  237 calls,
  0.097514 USD.
  First content median 5,290 ms,
  p90 10,943 ms,
  p99 22,746 ms,
  maximum 33,699 ms (line 5870);
  longest elapsed 225,272 ms (line 16768).
- `deepseek-flash` (the DeepSeek API):
  22 streams,
  21 `SPEND` lines,
  0.278706 USD,
  used for the re-asks of the NeuralWatt DeepSeek replies that failed the schema.
  First content median 1,687 ms,
  maximum 2,745 ms;
  1 cut (line 26590,
  41,227 ms);
  longest elapsed 80,537 ms.

## Providers that were not reached

- Hyper,
  OpenRouter and Bedrock:
  no request,
  no `SPEND`,
  no stream.
  The models they serve were seat-unreachable for the whole run:
  `minimax-m3` 1,435 refusals,
  `google.gemma-4-e2b` 1,370,
  `gemma-4-26b-a4b-it` 1,370,
  `inception/mercury-2.5` 1,182,
  `glm-5.3` 565 (`seatrefusals.ts`,
  5,922 in all),
  which are the `SEAT ... unreachable` counts of lines 39946 to 39956
  (the `SEAT` lines differ from the parsed counts by one on `minimax-m3` and `gemma-4-26b-a4b-it`,
  and add `google.gemma-4-31b` once).
  So there is no Mercury recheck in this run:
  `inception/mercury-2.5` had zero streams.
- Seat tally from the `SEAT` lines (39946 to 39956):
  `hf:moonshotai/Kimi-K3` asked 942,
  usable 942,
  unreachable 666;
  `hf:Qwen/Qwen3.8-27B` asked 797,
  usable 796,
  abandoned 131,
  unreachable 531;
  `deepseek-v4.1-flash` asked 1,523,
  usable 1,517,
  unusable 6,
  abandoned 1;
  `hf:zai-org/GLM-5.3-Flash` (served by NeuralWatt `glm-5.3-flash`) asked 496,
  usable 494,
  unusable 2;
  `xiaomi/mimo-v2.6-flash` asked 237,
  usable 237;
  `threw=0` on every seat.
  Asked counts are above the stream counts because they include 331 `PROMPT-REUSE` answers replayed from memory
  (`rg --count`).
  The check did not recount the per-model `SEAT` census beyond the 1,197 Synthetic refusals
  (first reader only for the rest).

## The Synthetic rename and what it cost

### The rename

- The last readable `/quotas` reading was line 12568 (22:20:54.193 UTC):
  `synthetic=wet`,
  weekly credit 34.315 percent.
  The reply then was 420 or 421 characters.
- The first unreadable reading was line 12620 (22:21:58.963 UTC,
  285 characters),
  followed by line 12621:
  `synthetic: budget unreadable, treating as spendable (Synthetic /quotas body violated expectations:
  weeklyTokenLimit is not an object; received subscription:object, search:object, planWeeklyLimit:object)`.
  That fits the brief's window (after 22:20:53 UTC and before 22:21:58 UTC)
  and the source comment that the later fix wrote
  (`provider-stream/src/synthetic-quota.ts:34` at `0204b7c8ba`).
  The troubleshooting record is
  [Synthetic quotas renamed its weekly block](../troubleshooting/synthetic-quota-weekly-block-renamed.md).
- 386 such warns follow (`rg --count`;
  216 in attempt 1,
  170 in attempt 2),
  one per fresh reading.
  The weekly credit fell from 95.0003 percent (line 211) to 34.3154 percent at line 12568,
  which is 60.7 points in 168 min (21.6 points an hour);
  the log shows no value after that.

### Synthetic calls after the rename

- From 22:21:58.964 UTC to the end (`synthetic.ts`):
  2,667 Synthetic streams;
  632 served with content (633 `SPEND` lines);
  1,976 refusal replies (1,975 of them HTTP 429 and one the Kimi 500 of 22:55:01 UTC,
  line 15110);
  58 cut.
  The check counted 1,975 refusal streams with a 144-character body after 00:28:48 UTC and none before it.
- Before 00:28:48 UTC there was no 429 at all:
  from 22:21:58 UTC to 00:28:48 UTC (2 h 7 min) the unreadable meter did no harm,
  because the credit was still above zero and the build treated the provider as spendable.
  The brief's sentence "routed Synthetic as spendable while its weekly credit was spent" therefore holds only
  from 00:28:47 UTC on 2026-10-23 and not from the rename.
- The credit ran out at 00:28:47 UTC.
  Evidence:
  - The first 269-character `/quotas` reply came at 00:28:47.016 UTC (line 19925)
    and the first HTTP 429 at 00:28:48.330 UTC (lines 19938 and 19939).
  - The reply sizes the check measured (`quotas.ts`):
    413 to 422 characters through line 12561,
    277 to 287 from line 12620,
    then 269 characters from line 19925,
    except in the two stretches of the last section.
    A 269-character reply is the size of a body whose weekly value is shorter.
    The source comment at `0204b7c8ba` records a keyed read at 03:28:16 UTC on 2026-10-23
    that returned 269 characters with `percentRemaining` 0 and `nextRegenPercent` 2
    (`provider-stream/src/synthetic-quota.ts:34` to `:40`),
    a read made outside this log;
    it falls inside this log's 269-character stretch,
    which supports the reading.
  - The only calls served after 00:28:48 UTC were in the windows where the reply measured about 286 characters again,
    which fits a credit that came back by a few percent twice.
  - The log never prints a 429 body (the reply is 144 raw characters and is not decoded),
    and shows no percentage after line 12568,
    so "weekly credit spent" is an inference from the reply size and the refusals,
    which the check kept as an inference.
- The 300,000 ms hold:
  395 `synthetic: refused us while its meter reads unreadable ... held out for 300000ms` lines
  (`holdwaits.ts`;
  lines 19970 to 39870),
  all 300,000 ms and all on `unreadable`.
  The check confirmed that `markRefused` has no other provider and no other duration in the log.
  The hold is `REFUSAL_COOLDOWN_MS` (300,000,
  `provider/src/provider-budget.ts:77`),
  taken in `markRefused` whenever the provider's own meter is not `wet` (`:827` to `:829`);
  with the meter unreadable the 429 was treated as exhaustion.
  With a wet meter the same 429 would have held Synthetic for 60,000 ms (`RATE_LIMIT_BACKOFF_MS`,
  `:96`).
- The seat reader then waited for that hold.
  `readDrynessPastShortBench` waits "the shortest running hold" whenever a bench is short and any hold is running
  (`run/src/corpus-run/run-seats-read.ts:380` to `:398`);
  when no hold is running it prints "no provider has named its return" and goes on at once
  (`:322` to `:379`).
  So the unreadable meter turned every refusal into a five minute wait for the whole entry.
  228 waits were logged (`holdwaits.ts`):
  translate lane 112,
  lane contest 52,
  consolidation 59,
  lanes 4,
  page consistency 1;
  every one named a non-zero Synthetic hold and a zero hold for NeuralWatt,
  Bedrock,
  DeepSeek,
  Xiaomi,
  Hyper and OpenRouter.
  The first is line 20031 (00:29:33 UTC),
  the last line 39832 (06:00:25 UTC).
  Their summed wait is 1,030.7 min by the check (the read's 1,030.8 is rounding;
  attempt 1:
  86 waits,
  363.3 min;
  attempt 2:
  142 waits,
  667.5 min),
  which adds concurrent slices;
  the wall-clock figure is the idle time.
- The cost,
  confirmed by the check on every figure:
  the window from 00:28:48.000 UTC to the last timestamped line (39944,
  06:05:34.105 UTC) is 336.77 min;
  idle (no model stream in flight,
  each stream taken as the interval from its line time minus its elapsed time to its line time,
  the six meter streams excluded) is 249.04 min in 59 gaps of 30 s or more
  (also 249.04 at 10 s;
  250.9 counting gaps of 1 s;
  251.8 counting every gap).
  Before 00:28:48 UTC the idle time is 0.00 min at 30 s and at 10 s.
  Of the 249.04 min,
  246.39 min (98.9 percent) lay inside the intervals of the logged waits
  (each wait line's time plus its `waiting Nms`).
  By stage the idle time was:
  attempt 1 translate lane 76.0 min;
  the first wait of attempt 2 5.0 min;
  translate lane 26.8 min;
  lane contest 71.4 min;
  consolidation 64.2 min;
  page consistency and assembly 4.8 min
  (the read had 4.9;
  the stages sum to 248.2 against 249.04,
  the difference being short gaps between stages).
- Seats lost:
  `hf:moonshotai/Kimi-K3` was seat-unreachable 666 times and `hf:Qwen/Qwen3.8-27B` 531 times,
  1,197 in all,
  all after 00:28:50 UTC (`seatrefusals.ts`;
  the check's `rg --count` agrees,
  and the first is line 19974).
  By stage:
  select 675,
  translate 326,
  produceConsolidations 116,
  lane-contest 63,
  absolute-naturalness-review 7,
  consolidate-gate 5,
  coverage 3,
  page-consistency 1,
  consolidation-polish-gate 1.
  The repair lane (critic,
  panel,
  editor,
  refiner,
  checker) closed at 00:11:11 UTC and lost no Synthetic seat.
- What a readable meter would have changed,
  from the code and not from a run:
  the read said Synthetic would have read `dry` (percent 0) with no hold,
  so `shortest` would have been zero and the stages would have gone on at once on the benches they could reach.
  The check could not confirm "with no hold".
  A readable dry meter is not `wet`,
  so `provider-budget.ts:827` to `:829` would still give a refusal received inside the 60 s freshness window
  (`BUDGET_FRESH_MS`,
  `:69`) the full 300,000 ms hold,
  and `readDrynessPastShortBench` would wait for it.
  But `routesAsDry` is true for `dry`,
  so once the meter read dry the router would stop sending to Synthetic,
  no further refusal would arrive,
  and `markRefused` would not run again.
  Holds would come only from refusals received before the next meter read and from each regeneration window
  when the meter briefly read wet again:
  a handful of holds and not 395,
  but not zero.
  This is code reading and not a run.
- The benches would not have been better either way.
  The first reading of the run,
  with every meter wet,
  already printed `wide 3 of 7 reachable, quorum 4` (line 212)
  and `select 3 of 7 reachable, quorum 4` (line 19172),
  so Synthetic's two seats lifted the bench to three of seven and no more.
  The missing time is recoverable;
  the missing voices are not.

### E2

- E2 did not fire.
  Every `METERS` line has `neuralwatt=wet`;
  every one of the 228 wait lines prints `holds neuralwatt 0ms`;
  there is no `markRefused neuralwatt` line.
  NeuralWatt's known return was never needed.
  This run says nothing about E2's fix (ledger B826).
  The NeuralWatt day started at 19:15:08 UTC on 2026-10-22 (line 205) and would have ended at 19:15:08 UTC on
  2026-10-23,
  13 h 10 min after the run,
  so no day reset was in reach.

## Where the 10.5 hours went

Times are UTC.
Each wall-clock figure is from the log's timestamps (`phases.ts`,
`gaps.ts`,
`idle.ts`).
"Idle" means no model stream was in flight,
counting gaps of 30 s or more;
it was 0.0 min before 00:28 UTC on 2026-10-23.

- Busy time:
  19:32 UTC to 00:28 UTC (4 h 56 min) had no idle gap.
  The first 4 h 39 min of it were preparation,
  the repair pass and the refine pass.
  In the repair pass the post-quorum grace on a straggler was the largest single cost;
  the check corrected its size to 647.6 of 792.8 summed slice minutes (81.7 percent),
  and the figure is explained in the defects file under D5.
- Waiting on the Synthetic hold:
  249.0 min of idle wall clock after 00:28 UTC
  (attempt 1 translate lane 76.0 min;
  attempt 2 5.0 + 26.8 + 71.4 + 64.2 + 4.8 = 172.2 min by the check's stage windows),
  against 336.8 min of wall clock after 00:28:48 UTC.
- Work that could not produce text:
  the 30 tail slices tried in both attempts (about 25 min and 38.7 min),
  the lane contest (73 min) and the consolidation (85 min).
  The check pointed out that the read added the contest and consolidation windows as "158 min"
  where they are 77.7 min and 85.4 min (163.1 min),
  and that 135.6 min of them are already inside the 249.0 idle minutes,
  so they are not a separate cost;
  only about 6.3 min of the contest window and 21.2 min of the consolidation window are not idle.
- NeuralWatt was wet the whole time,
  so no wait was on NeuralWatt.
- No `park` and no backoff line;
  the only backoffs were the transport retry ladders (1,591 retry lines,
  counted under "Synthetic").
- The time of each attempt by stage is in
  [the stages of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-stages.md).
