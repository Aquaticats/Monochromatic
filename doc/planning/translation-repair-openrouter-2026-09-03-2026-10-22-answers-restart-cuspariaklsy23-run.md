# OpenRouter and the corpus passes: the run of CuspariaKLSY23 on build 8864316eb2 (2026-10-22)

What was read on 2026-10-22 of the run of the pass that followed `zheermao23` on the same build:
the log,
the NeuralWatt meter and the plan energy,
the seats and the providers,
the Mercury recheck the owner asked for,
the stage outcomes,
the verdict of every contested slice,
the recurrence of the known classes,
and the figures that could not be sourced.
Entry:
CuspariaKLSY23 (entry `CuspariaKLSY`).
The stage outcomes,
the slice verdicts,
the recurrence of the known classes,
the traces and the open items are in
[the stages of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-stages.md),
and the log line behind every figure,
with the figures that could not be sourced,
is in
[the evidence of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-evidence.md).
The shipped page was not read line by line,
so this record has no departure list (D01 and on) as the record of `CuspariaKLSY22` has.
A second reader checked every figure against the logs on 2026-10-22
(137 confirmed,
6 corrected in place,
none unsourceable beyond the list in the evidence file);
the corrections are the slowest first content by model,
the `reason` keys inside `verdicts` entries,
the decimals of the OpenRouter `SPEND` lines,
the one credential refusal of `CuspariaKLSY22`,
the Kimi rounds that ran as long as its stream,
and the citation of the contest ballot fields.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md);
the figures set in brackets are from
[the run of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-run.md)
and from a script over its log,
and the `zheermao23` figures are from
[the run of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-run.md).

## CuspariaKLSY23 read, 2026-10-22, 14:10 UTC: SETTLED in 17.1 min of pass; NeuralWatt dry from the first reading, Mercury clean, tense left open by the repair lane again

`CuspariaKLSY23` (entry `CuspariaKLSY`) ran on tip `8864316eb29b9848d95ceeaf4c039671c0c430e8`
and pipeline `sha256-tree-v1:4e516f924c6f27eafe29e0d139d0cd04de5494428fc174da007db197d93f1f70`,
and its `START` line (log line 196) carries both values as given;
the artifact's `tip`,
`pipelineDigest` and `corpusSha` (`a41fc607ea5a70d8a7625cc67d5ed8c444f53379`) carry the same strings
(checked by `art3.ts`).
They are the same tip and pipeline as `zheermao23`.
`CuspariaKLSY22` ran on `2a336603bf`
and pipeline `sha256-tree-v1:5d4265dfab83b5a6e61b5b8c23d597519c7ca24293065473c883ec82af54c78c`.

The brief gives the launch as 13:22:07 UTC and the `DONE` as 13:43 UTC.
The log carries neither clock time.
Its first timestamped line is 13:22:21.578 UTC (line 199,
`republish: 0 settled pages judged`),
the page was written at 13:39:29.730 UTC (line 2206),
and its last timestamped line is 13:39:30.494 UTC (line 2210,
a `stream quota` line).
The last line reads `Finished in 1041.98s` (line 2222),
and 13:22:07 UTC plus 1041.98 s is 13:39:29 UTC,
which agrees with the log and not with 13:43 UTC.
The `DONE` line (2209) carries no clock time,
so the brief's 13:43 UTC cannot be the time of that line;
it is listed among the figures that could not be sourced.
The log has 2222 lines (`wc --lines`).
The build printed `Finished in 9.26s` (line 193;
`CuspariaKLSY22`:
8.38 s).

The reader read the log,
the run directory
(the artifact `CuspariaKLSY.json`,
336 stored reply-only payloads,
25 ledger files,
an empty `slice-cache`,
`attempts.json` reading `{"CuspariaKLSY": 1}`
and the shipped `fixed/people/CuspariaKLSY/page.en.md`),
the log of `CuspariaKLSY22`,
the repo record of its run,
and the draft of the `zheermao23` read.
No file in any git checkout was edited,
and no pass,
probe,
gate,
build,
test,
provider call or `git` command was run,
and no `.env*`,
auth or `environ` file was read.
Six source files and two git ref files of the worktree of the branch were read as files for the side checks,
named in "Traces and side checks";
the branch ref file reads the same tip as the `START` line,
but the working tree was not compared with that tip.
Every count names its method:
a script that counts log lines matching a pattern,
the same as `rg --count`,
or a script over the stream,
`SPEND`,
round,
`METERS` or `SEAT` lines,
the artifact or the payloads.
The scripts are `counts.ts`,
`stats.ts`,
`phases.ts`,
`seats.ts`,
`mercury.ts`,
`kimi.ts`,
`warns.ts`,
`misc2.ts`,
`art.ts`,
`art2.ts`,
`art3.ts`,
`art4.ts`,
`art5.ts`,
`polishfind.ts`,
`payloads.ts`,
`payloads2.ts` and `condense.ts` under `temp/agent/tr-fan/w284-read/`,
and,
for the log of `CuspariaKLSY22`,
`counts22.ts`,
`stats22.ts`,
`phases22.ts`,
`seats22.ts`,
`mercury22b.ts` and `payloads22.ts` beside them.
Every figure is traced to its log line in "Evidence for the figures".

### Run facts

The `TALLY` line (log line 2207) reads:

```text
TALLY CuspariaKLSY status=SETTLED slices=4 repairStatus=repaired repairIssues=36 repairAccepted=30 repairResolved=26 repairFindings=107 repairChanged=4 translateStatus=complete translateChanged=4 documentsDiffer=4 pageChanged=4 pageSilent=0 alignmentFindings=1 selection=contested ms=1028107
```

The `DONE` line (2209) reads `DONE processed=1 of pending=1; artifacts=1/1 declined=0 ... elapsed=1028162ms`,
which is 17.1 min.
`DESTINATIONS CuspariaKLSY source=0 page=0 dropped=0` (2208),
and `publish: wrote 4 slices into a page of 1886 characters` (2206;
`wc --chars --bytes --lines` of the shipped page gives 1886 characters,
1908 bytes and 52 lines;
`CuspariaKLSY22`:
1842 characters,
1864 bytes).
`FRONT MATTER entry=CuspariaKLSY authority=archive` (line 212,
13:22:23 UTC),
and `CORPUS NAMES ... named=0` (213).
The entry has four slices of 101,
104,
140 and 80 source characters (the `SLICE-START` lines 231 to 234),
and no link,
image or reference,
so `dropped=0` is true but could not have been otherwise.

Beside `CuspariaKLSY22` (its `TALLY`,
line 2300 of its log):

- `repairIssues` 36 against 29.
- `repairAccepted` 30 against 26,
  `repairResolved` 26 against 23.
- `repairFindings` 107 against 126.
- `repairChanged` 4 and `translateChanged` 4 on both;
  `documentsDiffer` 4 on both,
  `pageChanged` 4 on both,
  `pageSilent` 0 on both.
- `alignmentFindings` 1 on both.
- `selection=contested` on both.
- `ms` 1028107 against 2355675,
  so 22.1 min shorter (56 percent).

Where the 22.1 min went,
from the log timestamps (`phases.ts` splits the log at marker lines;
this run,
then `CuspariaKLSY22` by `phases22.ts`):

- Start of log to `both lanes over 4 slices, repair first` (line 229,
  13:22:29.985 UTC):
  0.14 min against 2.15 min.
  The block pairing round took 6,275 ms (line 225);
  in `CuspariaKLSY22` the same round took 125,594 ms,
  of which 120,003 ms were grace on a Mercury stream (its line 227).
  The entry has no reference,
  picture or archive block review (see "Stage outcomes").
- The repair lane,
  to the first `SLICE-START lane=translate` (line 1404,
  13:29:39.168 UTC):
  7.15 min against 21.41 min.
- The translate lane,
  to `lane contest: slices that differ: 4 of 4` (1674,
  13:33:00.569 UTC):
  3.36 min against 2.94 min.
- The lane contest,
  to `consolidation: 4 contested slices to settle` (1752,
  13:35:35.201 UTC):
  2.58 min against 3.01 min.
- Consolidation to `publish` (13:39:29.730 UTC):
  3.91 min against 9.75 min.
- So the repair lane (14.25 min shorter) and consolidation (5.85 min shorter) are 20.10 of the 22.13 min
  (differences of the unrounded times);
  preparation (2.01 min) and the contest (0.44 min) are the rest,
  and the translate lane was 0.42 min longer
  (the parts sum to 22.12 min;
  log start to publish is 17.14 min against 39.26 min).

What shaped the lanes:

- NeuralWatt and Synthetic were dry from the first reading and Hyper was wet all run;
  NeuralWatt and Synthetic took no call (see the NeuralWatt section).
  Hyper served 256 of the 352 settled calls
  (`minimax-m3` 58,
  `deepseek-v4.1-flash-1017` 59,
  `gemma-4-26b-a4b-it` 55,
  `kimi-k3` 39,
  `glm-5.3` 29 and `glm-5.3-flash` 16),
  and OpenRouter the other 96
  (a script over the `SPEND` lines).
- A round waits for a seat that has not answered until the grace after quorum ends:
  120 s for ordinary rounds,
  180 s for writer rounds (`WRITER GRACE built in`,
  line 197).
  The log's one `abandoned ... ms after quorum` line is `hf:moonshotai/Kimi-K3` in a lane contest round
  (line 1743,
  120000 ms),
  against 33 in `CuspariaKLSY22`,
  all `inception/mercury-2.5`.
  The one cut cost 120 s of grace.
- A script over the round lines counts 85 rounds,
  of which 1 had at least 100 s in grace (line 1736,
  120,004 ms)
  against 33 of 89 in `CuspariaKLSY22`.
  Rounds with 10 s or more in grace:
  39 of 85 against 61 of 89.
  Summed over all rounds,
  the grace is 24.6 min against 81.4 min,
  both by the same script on the two logs;
  the sum is over rounds that overlap (up to four slices were in flight in each lane,
  `phases.ts`),
  so it measures the cause and not wall clock.
- Wall clock is better read from `SLICE-COST` (one line per slice per lane),
  this run then `CuspariaKLSY22`:
  repair slice 3 342 s (line 1352),
  slice 1 280 s,
  slice 2 277 s,
  slice 0 260 s
  (1108 s,
  1032 s,
  842 s,
  810 s);
  translate slice 2 201 s,
  slice 1 166 s,
  slice 0 134 s,
  slice 3 62 s
  (175 s,
  135 s,
  109 s,
  101 s);
  consolidation slice 2 234 s (line 2202),
  slice 0 177 s,
  slice 1 153 s,
  slice 3 113 s
  (585 s,
  502 s,
  253 s,
  218 s).
- The slowest round of the run is the lane contest round of slice 2
  (153,218 ms total,
  33,214 ms to quorum,
  120,004 ms in grace,
  line 1736),
  which waited the full grace for the Kimi stream that was reasoning
  (see "Recurrence of the known classes").
- The last 70 s of the run were one re-ask.
  In the last absolute naturalness review of consolidation slice 2
  (round 4/6 heard,
  78,252 ms total and to quorum,
  0 ms in grace,
  line 2200),
  `deepseek-v4.1-flash` returned a `schema-mismatch` on Hyper at 13:38:19 UTC (line 2182)
  and its re-ask on OpenRouter,
  served by the endpoint `Morph`,
  took 70,347 ms (stream line 2197,
  answered usably at line 2199)
  against 7,885 ms for the Hyper stream of the same model (line 2180).
  That re-ask alone ended the slice,
  which is the slowest consolidation slice (234 s).

Counts over the log,
each by a counting script (`counts.ts`,
`warns.ts`,
`seats.ts`;
the `CuspariaKLSY22` figure in brackets is from `counts22.ts` and its read):

- `[warn]` lines:
  82 (103);
  `warns.ts` classes them:
  50 `seat unreachable` on `google.gemma-4-e2b` (45),
  8 `panel ballot ... irregular` (6),
  7 `candidate fails the publication rule` (9),
  3 `schema-mismatch` (2),
  3 `no re-ask elsewhere` (1),
  2 `recovery round` (1),
  2 `translate-archive-disputed` (2),
  2 `json trailing text` or `json false start`,
  2 page assembly lines,
  and one each of `abandoned ... after quorum` (33),
  `standing text lacks contest endorsement`
  and `settled on neither over a standing`.
- `[error]`:
  none (none).
- `DEFECTS`,
  `REATTEMPT`,
  `STALLED`,
  `mdx-downgraded`,
  `StreamOverrunError`:
  none of the five (none;
  the `CuspariaKLSY22` warn `ended a call that exceeded its content bound` appeared twice).
- `Interrupted`,
  `TranslationRepairInterruptedError`,
  `MdxParseError` and `refused by the page grammar`:
  none (none).
- `NoProviderForModelError`:
  none as a string;
  the refusals are worded `no provider can take <model>: ...`,
  53 lines (a script by model):
  50 `google.gemma-4-e2b ... seat unreachable`,
  2 `inception/mercury-2.5 ... no re-ask elsewhere`
  and 1 `glm-5.3 ... no re-ask elsewhere`.
- `seat unreachable`:
  50 lines (45),
  all `google.gemma-4-e2b`.
- `HTTP 401`:
  1 line (2),
  and it is the `PROVIDER OFF bedrock` line (line 198),
  which names the code in its text;
  no call was refused.
  `HTTP 402`,
  `HTTP 429` and `HTTP 5xx`:
  none (none).
- `schema-mismatch`,
  `SyntaxError` or `unusable pairing`:
  14 lines (13),
  of which 11 are `[info]` lines
  (`reaskElsewhereNudged` lines that carry the word)
  and 3 are `[warn]` (lines 954,
  1464 and 2187).
- `threw`:
  16 lines (16),
  of which 6 are `JUDGE SEATS` lines carrying `threw-withheld=none` and 10 are `SEAT` lines;
  every `SEAT` line reads `threw=0`
  (`CuspariaKLSY22`:
  `threw=2` on Kimi).
- `closed on a short bench` and `bench short of quorum`:
  none (none).
- Decision lines reading `keeping the fallback` or `keeping the repaired text`:
  3 (6),
  lines 1172,
  1392 and 1393;
  none is the consolidation run-off rule (see "Slice verdicts and disputes").
- `introduced repetition`,
  `archive-revision-refused`,
  `archive review dropped`,
  `refine-rolled-back-by-probe`:
  none of the four (none).
- The build output carries no `UNRESOLVED_IMPORT` line (none).
- `katex`:
  no line.
- `voice lost`:
  4 lines (37;
  lines 954,
  1464,
  1743 and 2187).

Counting effects:

- `alignmentFindings` 1 is the block pairing line itself
  (the artifact's `alignmentFindings` holds that one string).
- `stage-voice-lost` 15 and `stage-roster-incomplete` 15
  (`art2.ts` over the issue lists of the artifact):
  all 15 voice-lost findings name `google.gemma-4-e2b`.
  `glm-5.3` was asked 29 times (31 in `CuspariaKLSY22`).
- `seat unreachable` 50 against 45 is the same Bedrock effect:
  the provider was off from the first line,
  so no credential was refused (see "Seats,
  units and meters").
- `repairFindings` 107 is the sum of 14 kinds (`art.ts`):
  `panel-packet` 31,
  `refine-skip` 17,
  `stage-voice-lost` 15,
  `stage-roster-incomplete` 15,
  `duplicate-issue-merged` 6,
  `editor-candidates`,
  `editor-envelope-select` and `editor-chunk-select` 4 each,
  `select-self-vote` 4,
  `refine-skipped` 3,
  and one each of `block-pairing`,
  `refine-candidates`,
  `refine-declined` and `content-survival`.

### NeuralWatt: baseline, stop and energy

NeuralWatt took no model call in this run.
That is the first fact,
and the rest of this section follows from it.
The 14 `stream quota` lines (`rg --count`;
all `completed`,
none `cut` or `failed`;
`CuspariaKLSY22`:
28)
are meter reads and not model calls.

Baseline.

- One `takeBaseline` line,
  at 13:22:22.332 UTC (line 205):
  `NeuralWatt quota at the start of the run: 5.9624 kWh charged this period`
  (`zheermao23`:
  5.6318 kWh;
  `CuspariaKLSY22`:
  3.712 kWh).
- `decideNeuralWattDayAnchor` (line 206):
  the day starts at 19:15:08.000 UTC,
  and the current day began 2026-10-21T19:15:08.000Z.
  This run began 18 h 7 min into that day (computed from the line 206 anchor and the line 205 time),
  and the day ends at 19:15:08 UTC on 2026-10-22,
  5 h 36 min after the run ended.
  It is the same day as `XingZ6017` and `zheermao23`,
  which found the free energy overdrawn.

Stop.

- No stop line fired,
  and none could:
  every one of the 13 `METERS` lines (lines 210 to 2030) reads `neuralwatt=dry`,
  `neuralwattWh=-5.188`,
  `neuralwattReckonedWh=0.000`,
  `neuralwattAccountWh=555.188` and `neuralwattKeyAllowance=none`.
  The meter and the account sum to 550.000 on all 13 lines,
  so nothing was in flight on NeuralWatt at any reading.
  The pair of 5.188 Wh overdrawn and 555.188 Wh used is the last reading of `XingZ6017`
  and of `zheermao23`,
  so the account did not move between that reading and the 13 readings here.
- `rg` for `markRefused` finds no line.
  The router did not need to refuse NeuralWatt,
  because the meter already read dry at the first reading (13:22:23.061 UTC).
- `JUDGE SEATS` reads `neuralwatt=dry` from its first line (line 211).

Energy.

- A script that counts `SPEND` lines by provider finds Hyper 256 and OpenRouter 96,
  352 in all,
  and no `provider=neuralwatt` and no `provider=synthetic` line
  (`CuspariaKLSY22`:
  NeuralWatt 74 calls,
  Hyper 190,
  OpenRouter 109,
  373 in all).
  A case-insensitive search for `neuralwatt` finds 22 lines:
  the 13 `METERS` lines,
  the 6 `JUDGE SEATS` lines,
  the baseline,
  the day anchor and the end line.
  So the measured energy of this run is 0 Wh and no estimate-to-measure ratio exists.
- The two NeuralWatt models of `CuspariaKLSY22` ran on Hyper:
  `deepseek-v4.1-flash-1017` 59 calls and `glm-5.3-flash` 16.
  Hyper bills in credits and its `SPEND` lines carry no cost.
- Plan energy.
  The end line (2211) reads:

  ```text
  NeuralWatt PLAN ENERGY SPENT: 255.7 Wh of the owner's plan were charged between the start and the end of the run; anything else done on the account in that time is in this figure too
  ```

  (`zheermao23`:
  326.9 Wh;
  `XingZ6017`:
  54.3 Wh;
  `CuspariaKLSY22`:
  37.3 Wh).
  It is the fourth nonzero figure in a row,
  and 255.7 Wh in 17.1 min is 14.9 Wh per min,
  against 6.6 Wh per min for `zheermao23` (326.9 Wh in 49.4 min)
  and 0.95 Wh per min for `CuspariaKLSY22` (37.3 Wh in 39.3 min).
  The end baseline would stand near 6.2181 kWh (computed,
  no line prints it).

The contradiction,
as three numbers:

- the run made no NeuralWatt request,
  so none of the 255.7 Wh is this run's use of the covered models
  (inferred from the absence of any `SPEND` line and of any NeuralWatt stream;
  the log holds no NeuralWatt request line of any kind);
- the account reading stood at 555.188 Wh used on all 13 lines,
  so the covered-model draw on the free energy that the by-model route sees was zero in the window;
- the baseline of 5.9624 kWh is 3.7 Wh higher than the end baseline that `zheermao23` implies
  (5.6318 kWh plus 0.3269 kWh is 5.9587 kWh,
  computed;
  the figure carries a rounding of about 0.2 Wh),
  and the two passes ran back to back
  (the last timestamp of `zheermao23` is 13:20:58 UTC,
  this baseline 13:22:22 UTC,
  about 84 s apart),
  so the plan figure moved by 3.7 Wh in a gap with no pass in it,
  which is 2.6 Wh per min.

Whatever drew the 255.7 Wh drew it on the plan and not on the free energy that the by-model route reads,
and it was drawing faster than during `zheermao23`.
`neuralwatt-quota.ts:9` to `26` says the plan figure is `kwh_used`,
the energy charged against the plan,
that it "should not move" while the free energy lasts,
and that it cannot separate the owner's other account use from the run's
(read in the worktree this time).
The log cannot name the source of the 255.7 Wh.
The owner settled it on 2026-10-22:
the NeuralWatt key of the owner's dsh harness
(`package/dsh-plugin/neuralwatt`,
which reads `NEURALWATT_API_KEY`)
is on the same account,
so the plan figure counts that harness's use as well as the run's,
and the run made no NeuralWatt call.
It is not a code defect.
The free energy resets at 19:15:08 UTC.

### Seats, units and meters

The `SEAT` lines (10) print after the `DONE` line (2212 to 2221);
there is no `hf:Qwen/Qwen3.8-27B` line and no `google.gemma-4-31b` line this time
(`CuspariaKLSY22` printed 10 lines and neither of those two either):

- `google.gemma-4-e2b`:
  asked 0,
  unreachable 50
  (45).
- `deepseek-v4.1-flash`:
  asked 59,
  usable 59
  (55,
  55).
- `hf:zai-org/GLM-5.3-Flash`:
  asked 16,
  usable 16
  (asked 19,
  usable 18,
  unusable 1).
- `hf:moonshotai/Kimi-K3`:
  asked 39,
  usable 39,
  abandoned 1
  (asked 45,
  usable 43,
  threw 2).
- `glm-5.3`:
  asked 29,
  usable 28,
  unusable 1
  (31,
  30,
  1).
- `inception/mercury-2.5`:
  asked 64,
  usable 62,
  unusable 2,
  abandoned 0
  (asked 36,
  usable 36,
  abandoned 33).
- `gemma-4-26b-a4b-it`:
  asked 55,
  usable 55
  (55,
  55).
- `minimax-m3`:
  asked 58,
  usable 58
  (58,
  58).
- `typesafe/jev-1.13`:
  asked 16,
  usable 16
  (24,
  24).
- `xiaomi/mimo-v2.6-flash`:
  asked 8,
  usable 8
  (10,
  10).

Totals:
344 asked,
341 usable,
3 unusable,
0 threw,
1 abandoned,
50 unreachable (`seats.ts` over the `SEAT` lines;
`CuspariaKLSY22`:
333,
329,
2,
2,
33,
45).

Reading them:

- `threw=0` on all 10 lines;
  `SEATS DARK` does not print (`rg --count` finds none).
  The 50 `seat unreachable` warn lines are all `google.gemma-4-e2b`.
  There was no picture step,
  so no `google.gemma-4-31b` refusal.
- The three `unusable` answers are three `[warn]` lines:
  `glm-5.3` in translate slice 2 (line 1464),
  whose reply was a plain numbered translation and not JSON (stop reason `end_turn`),
  and `inception/mercury-2.5` in the repair select of slice 3 (line 954,
  a reply that opens as an array but carries an object key,
  stop reason `stop`)
  and in the last absolute naturalness review (line 2187,
  parsed as JSON but failed the caller schema guard).
  Each was followed by `no re-ask elsewhere` (lines 1463,
  953 and 2186),
  because the only provider that serves the model with budget had already answered.
  The `glm-5.3` reply and the first Mercury reply were each followed by a recovery round that heard 1 of 1
  (lines 1499 to 1510 and 1014 to 1033).
- The abandoned seat is Kimi alone,
  once,
  cut with 40 answer characters and 25,012 reasoning characters after 153,209 ms
  (stream line 1737,
  warn line 1743,
  round line 1736;
  see "Recurrence of the known classes").
  The `SEAT` tally does not count an abandoned call as asked
  (`seat-tally.ts:370` to `375`),
  so Kimi reads `asked=39` against 40 streams.
- The asks and the `SPEND` lines close exactly this time (`seats.ts`):
  344 asked,
  plus 8 re-asks that answered usably on OpenRouter
  (the `reaskElsewhereNudged` lines 337,
  433,
  742,
  757,
  1517,
  1630,
  1869 and 2199;
  `gemma-4-26b-a4b-it` 4,
  `minimax-m3` 3,
  `deepseek-v4.1-flash` 1),
  is 352,
  the `SPEND` lines;
  plus the 1 cut Kimi stream is 353,
  the model stream lines
  (432 stream lines less 79 meter reads:
  `quota`,
  `credits`,
  `quotas`,
  `models` and `usage-by-model`).
  That a re-ask is not counted as an ask is read from this arithmetic,
  and the code that counts asks was not read for it.
  The read of `CuspariaKLSY22` left two `SPEND` lines unaccounted (373 against 375),
  and the `zheermao23` draft left its seats unreconciled.
- The stored payloads close the same way:
  336 files (Hyper 256,
  OpenRouter 80),
  and the 80 are the 96 OpenRouter `SPEND` lines less the 16 `typesafe/jev-1.13` typed decisions,
  which are not stored as replies
  (inferred from the counts).
- `JUDGE SEATS` prints 6 lines (6 in `CuspariaKLSY22`;
  lines 211,
  227,
  228,
  1400,
  1673 and 1751),
  and the fields are identical on all six:
  `neuralwatt=dry synthetic=dry bedrock=dry hyper=wet openrouter=wet`,
  `wide=6 select=6 late=7 slate=6 checkers=3 translators=7 readers=5 seeing=4 writers=8 roster=8`,
  `withheld=hf:Qwen/Qwen3.8-27B,hf:moonshotai/Kimi-K3`,
  `unserved=google.gemma-4-e2b,google.gemma-4-31b`
  and `threw-withheld=none`.
  Kimi is named in `withheld=` and still served 39 calls on Hyper:
  `withheld=` names a model that lost at least one seat,
  and `select=6 slate=6` against `late=7` fits that,
  as the `CuspariaKLSY22` providers record reads it at its lines 236 to 242
  (it cites `run-seats.ts:386`,
  which was not re-read).
- Qwen got no call because Synthetic was dry from the first reading,
  so the model was withheld from every phase.
  In `zheermao23` the translate lane roster still seated it
  (its line 2681,
  with `synthetic=wet` on the line before),
  and it then ran 11 calls on Hyper;
  here the translate lane re-seat line (1401) names seven writers and not Qwen,
  with `synthetic=dry` on line 1400.
- The re-seat lines:
  `chunk re-seated under a hold` prints 0 times (0),
  `slice re-seated under a hold` 0 times,
  and `translate lane re-seated before starting` once (line 1401;
  1).
  Line 1401 lists `google.gemma-4-e2b` among the seven writers although its only provider was off,
  as the same line did in the sibling reads.
- Did any quorum close short?
  No.
  All 4 introduced-defect probe rounds read `3/3 heard` (lines 1199,
  1252,
  1258 and 1349),
  with no `bench short of quorum` line,
  and none of the 6 absolute naturalness reviews read `quorum-not-met`
  (4 acceptable,
  2 unacceptable;
  `5/7`,
  `5/6`,
  `5/7`,
  `5/6`,
  `5/7` and `4/7 usable` against `quorumNeeded=4`).
  50 of the 85 rounds heard fewer voices than their bench.
  Three rounds closed on a short count with 0 ms in grace:
  the repair select of slice 3 (3/5,
  22,700 ms,
  line 1013),
  the translate round of slice 2 (4/6,
  86,348 ms,
  line 1498)
  and the last naturalness review (4/6,
  78,252 ms,
  line 2200).
  Each sits beside an unusable answer or a slow re-ask that the neighbouring lines name
  (Mercury at line 954,
  `glm-5.3` at line 1464,
  the `deepseek-v4.1-flash` re-ask at line 2197),
  so the read takes the quorum to have been met by the last live voice and not by a hung stream.
  That is the shape `XingZ6017` named N1 (no grace runs when the quorum equals the live bench),
  but the round code was not read,
  so these are candidates and not recurrences.

Units.

- NeuralWatt is in watt-hours on the meter and the account lines,
  and in kilowatt-hours on the plan baseline.
- Synthetic is a weekly meter in percent;
  it reads 0 percent on all 13 `METERS` lines,
  with `syntheticFiveHour=none` and `syntheticThrottled=none`.
- Hyper is a balance in credits (an integer on the `METERS` lines),
  and its `SPEND` lines carry no cost.
- OpenRouter is in United States dollars:
  two decimals on the `METERS` lines,
  7 to 9 on the `SPEND` lines
  (8 on 51 lines,
  9 on 40,
  7 on 5).
- Times are in milliseconds in the log;
  minutes in this record are the millisecond figure divided by 60,000.
- The result of the systemd unit (`Result=success`) is the brief's;
  it is not in the log (see the figures that could not be sourced).

Bedrock,
Hyper,
Synthetic,
OpenRouter:

- Bedrock:
  the provider was off.
  The launch printed `PROVIDER OFF bedrock: the owner ruled on 2026-10-20 to leave Bedrock off rather than issue
  a new key ...` (line 198),
  so no credential was refused and no `SEAT CREDENTIAL REFUSED` line printed
  (`CuspariaKLSY22`:
  one refusal,
  `SEAT CREDENTIAL REFUSED` at its line 215 after one `HTTP 401` call at its line 214).
  `METERS` reads `bedrock=dry` on all 13 lines and `JUDGE SEATS` reads `bedrock=dry` on all 6,
  so the two agree;
  the `METERS` lines no longer carry the `bedrockUsd` field.
- Hyper:
  wet all run (`hyper=wet` on 13 of 13 `METERS` lines);
  `hyperBalance` 193 at the first line and 157 at the last (36 credits by 13:37:36 UTC;
  `zheermao23`:
  250 to 194 in 56 credits).
  The `SPEND provider=hyper` lines count 256 (555 in `zheermao23`).
  At this run's rate of 36 credits in 15.2 min,
  157 credits is about 66 min of pass time (computed).
- Synthetic:
  dry from the first `METERS` line (line 210),
  weekly meter 0 percent on all 13 lines,
  no `HTTP 429`,
  no `markRefused` and no `HTTP 402`.
  Synthetic took 0 calls and 0 completion tokens
  (`zheermao23`:
  178 calls).
  The weekly meter has not moved from the closing reading of `zheermao23`.
- OpenRouter:
  3.98 USD at the first `METERS` line to 3.93 USD at the last;
  the 96 `SPEND provider=openrouter` lines sum to 0.06023 USD
  (`inception/mercury-2.5` 64 calls,
  0.03881 USD;
  `typesafe/jev-1.13` 16,
  0.004751;
  `xiaomi/mimo-v2.6-flash` 8,
  0.004781;
  `minimax/minimax-m3` 3,
  0.007147;
  `google/gemma-4-26b-a4b-it` 4,
  0.002345;
  `deepseek/deepseek-v4.1-flash-1017` 1,
  0.002396;
  `CuspariaKLSY22`:
  0.0730 USD over 109 calls;
  `zheermao23`:
  0.1178 USD over 191).
  `XingZ6017` spent 9.218 USD of the balance in its long run,
  and 3.93 USD is left.

The condition against `CuspariaKLSY22`:
two of the three metered providers were dry from the start
(NeuralWatt and Synthetic),
where in `CuspariaKLSY22` Synthetic was wet and NeuralWatt took 74 calls,
and the run was 22.1 min shorter,
in a run where the Mercury stalls of `CuspariaKLSY22` did not recur
(see "The Mercury recheck");
the log shows both,
not that one caused the other.

### The Mercury recheck

The owner asked for a recheck of `inception/mercury-2.5`.
The question is whether its streams stalled,
and whether any reached the 90000 ms first-content window.

- The `SEAT` line (log line 2217) reads
  `SEAT inception/mercury-2.5 asked=64 usable=62 unusable=2 threw=0 silent=0 abandoned=0 unreachable=0`
  (`CuspariaKLSY22`:
  `asked=36 usable=36 abandoned=33`;
  `zheermao23`:
  `asked=121 usable=118 unusable=3 abandoned=0`).
- Streams:
  64 `stream inception/mercury-2.5` lines (`mercury.ts` over the stream lines),
  all 64 `completed`,
  none `cut`
  (`CuspariaKLSY22`:
  69 streams,
  33 cut after 123,140 to 179,481 ms,
  all 33 with 0 content characters;
  `mercury22b.ts`),
  and 64 `SPEND` lines
  (`provider=openrouter`,
  endpoint `Inception` on all 64,
  0.03881 USD,
  168,151 completion tokens).
  The asks (64) equal the streams (64) and the `SPEND` lines (64).
- First content,
  over the 64 completed streams (every one carries a first-content reading):
  maximum 21,327 ms (line 1530),
  median 6,656 ms,
  minimum 4,701 ms (line 1193),
  mean 7,229 ms,
  90th percentile 8,885 ms,
  95th percentile 9,599 ms
  (nearest rank,
  zero-based index of the floor of the count times the share).
  The next four are 10,610 ms (line 1524),
  10,242 ms (line 1494),
  9,599 ms (line 438)
  and 9,150 ms (line 440).
  The first half of the run (32 streams) has a median of 6,656 ms and the second half 6,700 ms.
- Streams that hit the 90000 ms window:
  none.
  No Mercury stream had a first content,
  or an elapsed time,
  at or over 90,000 ms,
  or at or over 30,000 ms;
  the longest elapsed time was 21,556 ms (line 1530),
  and the median elapsed time 6,738.5 ms.
  The window is 4.2 times the largest first content and 4.2 times the longest elapsed time.
  The window itself is not printed anywhere in the log,
  and the artifact's `callConfig` records only the shared windows:
  `perCallTimeoutMs: 360000`,
  `streamFirstContentMs: 180000`,
  `streamIdleMs: 120000`.
  A hit would be a stream that carries no content until the window ends,
  and the log holds no such stream.
- First byte:
  maximum 1,300 ms (line 1494),
  median 815.5 ms.
- Empty streams:
  none.
  No Mercury stream carried 0 content characters,
  none read `firstContent -1ms`,
  and there is no `provider_unavailable`,
  `code 502` or `transport failure` line in the log
  (`zheermao23` had two empty streams that ended in a provider 502 and a retry).
- The two `unusable` answers are a `SyntaxError` reply and a schema-guard reply
  (lines 954 and 2187;
  see "Seats,
  units and meters"),
  and `readJsonOutcome` read one reply with 4 trailing characters (line 244)
  and one with an abandoned opening of 28 characters (line 1295).
  None is a stall.
- Cost to the run:
  no round waited on a Mercury stream,
  because the single `abandoned` line is Kimi's.
- The slowest Mercury streams are the translate lane's.
  All four Mercury streams of the translate lane fall in 13:30:06 to 13:31:52 UTC,
  at 6,616 ms (line 1458),
  10,242 ms (line 1494),
  10,610 ms (line 1524)
  and 21,327 ms (line 1530);
  the three slowest of the run are three of those four.
  The 21,327 ms stream is 1.7 times the largest first content of `zheermao23` (12,820 ms).
  The largest completed elapsed time of `CuspariaKLSY22` was 22,356 ms,
  and the card puts the 99th percentile at 20.3 s over 4,636 streams
  (`model-cards.ts:424` to `425`),
  so one stream of 64 just above that percentile is not out of line.
  Why the translate lane's Mercury calls ran longer was not traced;
  reply length does not explain it
  (the 10,610 ms stream at line 1524 carried 6,379 content characters and the 10,242 ms stream at line 1494 carried 318).
- The series for this seat,
  from the sibling reads:
  `zheermao22` 11 abandoned of 107 asked (21:59 to 22:24 UTC on 2026-10-21);
  `CuspariaKLSY22` 33 cut out of 69 streams (22:25 to 23:04 UTC);
  `XingZ6017` 53 cut out of 1535 streams (23:05 to 03:34 UTC);
  `zheermao23` 0 stalled out of 121 (12:31 to 13:21 UTC);
  this run 0 stalled out of 64 (13:22 to 13:39 UTC).
- What the recheck shows,
  and what it does not.
  It shows Mercury answering in 21.6 s or less on every one of 64 streams in a 17-minute window,
  with the median at 6.7 s,
  the same as the card's 6.7 s over 4,636 streams.
  It does not exercise the 90000 ms window,
  because no stream came near it,
  and so it neither confirms that the window works nor shows that it is needed.
  Two clean runs in a row (185 streams) show no stall between 12:31 and 13:39 UTC;
  they do not say whether the earlier stalls were a matter of time of day,
  load on the provider or something else.
- Side check on the window itself,
  from the worktree files (not from the log):
  the Mercury card names `firstContentMs: 90_000` (`model-cards.ts:431`,
  with the ledger B809 comment at 418),
  `openrouter-client.ts:462` passes the card's own window to the request,
  and the shared windows are `STREAM_FIRST_CONTENT_MS = 180_000` and `STREAM_IDLE_MS = 120_000`
  (`stream-idle-guard.ts:89` and `:106`),
  which agree with the artifact's `callConfig`.
  The branch ref file of the worktree reads `8864316eb29b9848d95ceeaf4c039671c0c430e8`,
  the `START` tip;
  the working tree was not compared with that tip,
  so that the run used the card's window is read from the match of the tip and not proved.
- For scale,
  the slowest first content of any model in the run is `gemma-4-26b-a4b-it` on Hyper at 28,953 ms,
  then Mercury at 21,327 ms,
  Kimi at 13,688 ms
  (its cut stream,
  line 1737;
  its slowest completed stream is 8,980 ms at line 1701),
  `glm-5.3` at 13,317 ms
  and `minimax-m3` at 9,304 ms (`stats.ts`);
  no stream of any model reached 60,000 ms to first content.
