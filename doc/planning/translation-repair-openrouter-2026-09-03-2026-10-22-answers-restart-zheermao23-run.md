# OpenRouter and the corpus passes: the run of zheermao23 on build 8864316eb2 (2026-10-22)

What was read on 2026-10-22 of the run of the pass that followed the first-content window and the polish-note fixes:
the log,
the NeuralWatt meter and the plan energy,
the seats and the providers,
the Mercury recheck the owner asked for,
why the repair counts rose,
the verdict of every contested slice,
whether the new rulings fired,
the recurrence of the known classes,
and the figures that could not be sourced.
Entry:
zheermao23 (entry `zheermao101`).
The stage outcomes,
the slice verdicts,
the rulings,
the recurrence of the known classes and the open items are in
[the stages of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-stages.md),
and the log line behind every figure,
with the figures that could not be sourced,
is in
[the evidence of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-evidence.md).
The archive block review of the letter and the email is reported here only as the log shows it,
and the shipped page was not read line by line,
so this record has no departure list (D01 and on) as the record of `zheermao22` has.
A second reader checked every figure against the logs on 2026-10-22
(93 confirmed,
3 corrected in place,
4 that no log can settle,
listed in the evidence file);
the corrections are the round-grace comparison,
the `zheermao22` reason-first count and the irregular-ballot counts of the sibling runs.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md);
the figures set in brackets are from
[the run of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md),
and the sibling figures named by entry are from the provider reads of
`CuspariaKLSY22` and `XingZ6017` in the same directory.

## zheermao23 read, 2026-10-22, 13:38 UTC: SETTLED in 49.4 min of pass; NeuralWatt dry from the first reading, Mercury healthy, Qwen cut six times

`zheermao23` (entry `zheermao101`) ran on tip `8864316eb29b9848d95ceeaf4c039671c0c430e8`
and pipeline `sha256-tree-v1:4e516f924c6f27eafe29e0d139d0cd04de5494428fc174da007db197d93f1f70`,
and its `START` line (log line 196) carries both values as given;
the artifact's `tip` and `pipelineDigest` carry the same two.
`zheermao22` ran on `2a336603bf`
and pipeline `sha256-tree-v1:5d4265dfab83b5a6e61b5b8c23d597519c7ca24293065473c883ec82af54c78c`.
The brief gives the launch as 12:30:44 UTC;
the `START` line carries no time.
The log's first timestamped line is 12:31:31.474 UTC (line 199,
`republish: 0 settled pages judged`)
and its last is 13:20:58.428 UTC (line 4210,
a `stream quota` line);
the log has 4224 lines (`wc --lines`),
and its last line reads `Finished in 2989.20s`.
The corpus input is the same pin as in `zheermao22`
(the artifact's `corpusSha` reads `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`).

The reader read the log,
the run directory
(the artifact `zheermao101.json`,
892 stored reply-only payloads,
40 ledger files,
an empty `slice-cache`
and the shipped `fixed/people/zheermao101/page.en.md`)
and the sibling reads named in the introduction.
No file in any git checkout was edited,
and no pass,
probe,
gate,
build,
test,
provider call or `git` command was run,
and no `.env*`,
auth or `environ` file was read.
The worktree of the branch was read as files for four side checks,
named in "Traces and side checks";
its branch ref file reads the same tip as the `START` line,
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
`rounds.ts`,
`graces.ts`,
`misc.ts`,
`misc2.ts`,
`warns.ts`,
`art.ts`,
`art2.ts`,
`payloads.ts`,
`payloads2.ts`,
`consol.ts` and `slice0.ts` under `temp/agent/tr-fan/w281-read/`.
Every figure is traced to its log line in "Evidence for the figures".

### Run facts

The `TALLY` line (log line 4207) reads:

```text
TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired repairIssues=42 repairAccepted=21 repairResolved=21 repairFindings=270 repairChanged=9 translateStatus=complete translateChanged=9 documentsDiffer=10 pageChanged=11 pageSilent=0 alignmentFindings=63 selection=contested ms=2966120
```

The `DONE` line (4209) reads `DONE processed=1 of pending=1; artifacts=1/1 declined=0 ... elapsed=2966245ms`,
which is 49.4 min.
`DESTINATIONS zheermao101 source=2 page=3 dropped=0` (4208),
and `publish: wrote 15 slices into a page of 5972 characters` (4206,
13:20:57.710 UTC;
`wc --chars --bytes --lines` gives 5972 characters and 6102 bytes,
151 lines;
`zheermao22`:
6117 characters,
6255 bytes,
148 lines).
`FRONT MATTER entry=zheermao101 authority=archive` (line 211,
12:31:33 UTC).

Beside `zheermao22` (figures from its read):

- `repairIssues` 42 against 31.
- `repairAccepted` 21 against 22,
  `repairResolved` 21 against 21.
- `repairFindings` 270 against 258.
- `repairChanged` 9 against 10;
  `translateChanged` 9 against 10.
- `documentsDiffer` 10 against 11,
  `pageChanged` 11 against 12,
  `pageSilent` 0 on both.
- `alignmentFindings` 63 against 61.
- `selection=contested` on both.
- `ms` 2966120 against 4741311,
  so 29.6 min shorter (37 percent).

Where the 29.6 min went,
from the log timestamps (a script that splits the log at marker lines;
this run,
then `zheermao22`):

- Start of log to `both lanes over 15 slices, repair first` (line 634,
  12:50:35 UTC):
  19.1 min against 20.8 min.
  The local photo read took 1.35 s,
  0.78 s and 0.64 s
  (from the phase start or the previous photo's `corroborated` line to its `read N characters without a model` line,
  log lines 271,
  289 and 307;
  `zheermao22`:
  1.6 s,
  0.9 s and 0.7 s),
  and the three read the same 515,
  422 and 388 characters.
  The picture phase,
  with five model readers,
  ran 12:32:36 to 12:34:41 UTC (2.1 min;
  `zheermao22`:
  1.8 min).
  The archive block reviews ran 12:35:14 to 12:49:48 UTC,
  14.6 min (`zheermao22`:
  16.7 min).
- The repair lane,
  to the first `SLICE-START lane=translate` (line 2685,
  13:09:32 UTC):
  18.9 min against 25.4 min.
  The chunk pass ended with the last repair `SLICE-COST` at 13:05:45 UTC (15.2 min);
  the refine pass,
  with its select,
  checker and probe rounds,
  ran on to 13:09:32 UTC.
- The translate lane,
  to `lane contest: slices that differ: 10 of 15` (3154,
  13:13:53 UTC):
  4.4 min against 5.8 min.
- The lane contest,
  to `consolidation: 10 contested slices to settle` (3351,
  13:15:06 UTC):
  1.2 min against 5.8 min.
- Consolidation to `publish` (13:20:57 UTC):
  5.9 min against 21.2 min.
- So consolidation (15.3 min shorter) and the repair lane (6.5 min shorter) are 21.8 of the 29.6 min;
  the contest (4.6 min),
  the translate lane (1.4 min) and preparation (1.7 min) are the rest
  (the parts sum to 29.5 min,
  the 0.1 min is rounding).

What shaped the lanes:

- Synthetic stayed wet to 13:11:26.103 UTC (39.9 min into the 49.4 min run;
  `zheermao22`:
  69.1 of 79.0 min) and Hyper was wet all run;
  NeuralWatt was dry all run and took no call
  (see the NeuralWatt section).
  Hyper served the two NeuralWatt models (`deepseek-v4.1-flash` 126 calls,
  `glm-5.3-flash` 56 calls),
  and from the Synthetic dry reading it served `hf:moonshotai/Kimi-K3` (35 calls from line 2850) and
  `hf:Qwen/Qwen3.8-27B` (11 calls from line 2957).
- A round waits for a seat that has not answered until the grace after quorum ends:
  120 s for ordinary rounds,
  180 s for writer rounds (`WRITER GRACE built in`,
  log line 197).
  The log's 6 `abandoned ... ms after quorum` lines are all `hf:Qwen/Qwen3.8-27B`
  (`zheermao22`:
  23,
  of which `inception/mercury-2.5` 11,
  Qwen 10,
  Kimi 1 and `hf:zai-org/GLM-5.3-Flash` 1).
  The six cost 780 s of grace in all
  (one writer round at 180 s,
  line 385,
  and five rounds at 120 s,
  lines 536,
  896,
  1147,
  1393 and 1692).
- A script over the round lines counts 205 rounds,
  of which 7 had at least 100 s in grace,
  against 30 in `zheermao22`.
  The seven are the six Qwen rounds and the slice 9 editor round of the repair lane
  (117.6 s in grace,
  line 2136),
  which waited for a `glm-5.3-flash` stream on Hyper that sent its first content after 122,849 ms (line 2134).
  Only one round waited 140 s or more (the archive block review round at 180 s,
  line 385),
  against four in `zheermao22`.
  Summed over all rounds,
  the grace fell from 132.74 min over 218 rounds in `zheermao22` to 68.59 min over 205 rounds;
  over the rounds with 100 s or more in grace it fell from 62.43 min (30 rounds) to 14.96 min (7 rounds),
  the `zheermao22` read's 62.4 min being that second sum
  (both recounted from the two logs by the independent check,
  `zheermao23-read-check.md`).
  These sums are over rounds that overlap (up to four slices were in flight in each lane,
  a script over the `SLICE-START` and `SLICE-COST` lines),
  and 97 of the 205 rounds had 10 s or more in grace,
  so they measure the cause and not wall clock;
  both fell with the wall clock.
- Wall clock is better read from `SLICE-COST` (one line per slice per lane):
  repair slice 0 846 s (line 2357),
  slice 9 534 s,
  slice 8 295 s,
  slice 4 246 s,
  slice 5 236 s
  (`zheermao22`:
  largest 746 s,
  slice 8);
  consolidation slice 0 204 s,
  slice 8 149 s,
  slice 4 148 s,
  slice 14 129 s,
  slice 12 123 s,
  slice 5 108 s,
  slice 11 99 s
  (`zheermao22`:
  largest 612 s,
  slice 4).
- Repair slice 0 is the single slowest piece,
  and slower than the slowest of `zheermao22`.
  The 12 round lines tagged `repair slice 0` sum to 846 s total and 644 s in grace
  (a script over the round lines;
  rounds inside the panel stage overlap,
  so this is not a wall-clock claim).
  Two panel rounds waited the full 120 s grace for a Qwen stream that was reasoning with no answer
  (lines 1393 and 1692),
  the critic round had 90.3 s in grace (line 738),
  three select rounds had 55.0 s,
  65.0 s and 44.0 s,
  and the introduced-defect probe round 63.5 s.
  Slice 9 is the second:
  447 s of its 603 s of round time were grace.
- In consolidation,
  `inception/mercury-2.5` accounts for 282 s of stream time over 43 calls
  (`zheermao22`:
  1701 s over 48 calls),
  and Qwen for none (it was withheld);
  `hf:moonshotai/Kimi-K3` on Hyper accounts for 336 s over 21 calls
  (a script over the stream lines tagged `consolidation slice`).
  No mercury stream carried an empty body for the grace to cut
  (see "The Mercury recheck").

Counts over the log,
each by a counting script (the `zheermao22` figure in brackets is from its read):

- `[warn]` lines:
  184 (210);
  a script that classes them finds 112 `seat unreachable` on `google.gemma-4-e2b`,
  30 `panel ballot ... irregular`,
  9 `archive review dropped`,
  6 `abandoned ... after quorum`,
  3 `unusable pairing`,
  3 picture refusals for `google.gemma-4-31b`,
  3 `translate-archive-disputed`,
  3 `no re-ask elsewhere`,
  3 lane-contest or select `schema-mismatch`,
  4 `candidate fails the publication rule` or `revision still fails`,
  2 `introduced repetition`,
  2 `502 transport failure`,
  and one each of `recovery round`,
  `standing text lacks contest endorsement`,
  `settled on neither over a standing` and `json trailing text`.
- `[error]`:
  none (none).
- `DEFECTS`,
  `REATTEMPT`,
  `STALLED`,
  `mdx-downgraded`,
  `StreamOverrunError`:
  none of the five (none).
- `Interrupted` and `TranslationRepairInterruptedError`:
  none (none).
- `NoProviderForModelError`:
  none as a string;
  the refusals are worded `no provider can take <model>: ...`,
  118 lines (a script by model):
  112 `google.gemma-4-e2b ... seat unreachable`,
  3 `google.gemma-4-31b failed outright` and 3 `inception/mercury-2.5 ... no re-ask elsewhere`.
- `seat unreachable`:
  112 lines (110),
  all `google.gemma-4-e2b`.
- `HTTP 401`:
  1 line (3),
  and it is the `PROVIDER OFF bedrock` line (log line 198),
  which names the code in its text;
  no call was refused.
  `HTTP 402`:
  none (none).
  `HTTP 429`:
  none (10).
  The two Mercury `code 502` lines are in the stream failure text and not an `HTTP` line.
- `schema-mismatch`,
  `SyntaxError` or `unusable pairing`:
  18 lines (27),
  of which 12 are `[info]` lines
  (`reaskElsewhereNudged` lines that carry the word)
  and 6 are `[warn]` (3 `unusable pairing` from `minimax-m3` and 3 `schema-mismatch` from `inception/mercury-2.5`).
- `threw`:
  19 lines (21),
  of which 7 are `JUDGE SEATS` lines carrying `threw-withheld=none` and 12 are `SEAT` lines;
  every `SEAT` line reads `threw=0`.
- `closed on a short bench` and `bench short of quorum`:
  none (none).
- Decision lines reading `keeping the fallback` or `keeping the repaired text`:
  none (8).
- `abandoned ... ms after quorum`:
  6 (23).
- `refused by the page grammar`,
  `MdxParseError`:
  none (none).
- `archive-revision-refused`:
  none (2);
  `introduced repetition`:
  2 lines (2),
  the same warn printed by the repair and the translate lane (log lines 2676 and 3144),
  both for "One Among Us members" in slices 9 and 11 (archive 0,
  shipped 2).
- `archive review dropped the editorial-context reply`:
  8 lines (6),
  on the two label blocks (5 on the email's label,
  3 on the letter's label);
  and one line of a kind the `zheermao22` log did not have,
  `archive review dropped the source-supported reply of inception/mercury-2.5` (line 387).
- `refine-rolled-back-by-probe`:
  none (1).
- The build output carries no `UNRESOLVED_IMPORT` line in this log (none).
- `katex`:
  no line.

Counting effects,
as `zheermao22` explained its own:

- `alignmentFindings` 63 against 61 is the kept review findings:
  the four block reviews recorded 12 (the email),
  12 (its label),
  10 (the letter) and 11 (its label) findings
  (the `review recorded N findings` lines 425,
  483,
  544 and 599),
  45 of the 63
  (`zheermao22`:
  10,
  10,
  12 and 13,
  also 45 of 61).
- `stage-voice-lost` 37 against 34 and `stage-roster-incomplete` 35 against 32
  (a script over the issue lists of the artifact):
  34 of the 37 voice-lost findings name `google.gemma-4-e2b` and 3 name `hf:Qwen/Qwen3.8-27B`.
  `glm-5.3` was asked 70 times (75 in `zheermao22`).
- `seat unreachable` 112 against 110 is the same Bedrock effect:
  the provider was off from the first line,
  so no credential was refused this time (see Bedrock).

### NeuralWatt: baseline, stop and energy

NeuralWatt took no call in this run.
That is the first fact,
and the rest of this section follows from it.

Baseline.

- One `takeBaseline` line,
  at 12:31:32.217 UTC (line 202):
  `NeuralWatt quota at the start of the run: 5.6318 kWh charged this period`
  (`zheermao22`:
  3.712 kWh).
  It is the plain wording.
  All 39 `stream quota:` lines of the run read `completed` (`rg --count`;
  no `cut` or `failed` line;
  `zheermao22`:
  58).
- `decideNeuralWattDayAnchor` (line 203):
  the day starts at 19:15:08.000 UTC,
  current day began 2026-10-21T19:15:08.000Z.
  This run began 17 h 16 min into that day (computed from the line 203 anchor and the line 202 time),
  and the day ends at 19:15:08 UTC on 2026-10-22,
  after the run.
  It is the same day as `XingZ6017`,
  which ended it with the free energy overdrawn.

Stop.

- No stop line fired,
  and none could:
  every one of the 38 `METERS` lines (lines 209 to 4163) reads `neuralwatt=dry`,
  `neuralwattWh=-5.188`,
  `neuralwattReckonedWh=0.000`,
  `neuralwattAccountWh=555.188` and `neuralwattKeyAllowance=none`.
  The meter and the account sum to 550.000 on all 38 lines
  (`zheermao22`:
  13 of 57 lines differed by a call in flight),
  so nothing was in flight on NeuralWatt at any reading.
  The pair of 5.188 Wh overdrawn and 555.188 Wh used is the last reading of the `XingZ6017` read
  (its providers file,
  lines 292 and 293),
  so the account did not move between that reading and the 38 readings here.
- `rg` for `markRefused` finds no line.
  The router did not need to refuse NeuralWatt,
  because the meter already read dry at the first reading (12:31:33.329 UTC).
- `JUDGE SEATS` reads `neuralwatt=dry` from its first line (line 210).

Energy.

- A script that counts `SPEND` lines by provider finds Hyper 555,
  OpenRouter 191 and Synthetic 178,
  924 in all,
  and no `provider=neuralwatt` line
  (`zheermao22`:
  NeuralWatt 190 calls and 40.9099 Wh,
  Hyper 368,
  OpenRouter 192,
  Synthetic 254,
  1004 in all).
  A case-insensitive search for `neuralwatt` finds 48 lines:
  the 38 `METERS` lines,
  the 7 `JUDGE SEATS` lines,
  the baseline,
  the day anchor and the end line.
  So the measured energy of this run is 0 Wh and no estimate-to-measure ratio exists
  (`zheermao22`:
  1.39;
  `XingZ6017`:
  2.84).
- The two models that ran on NeuralWatt in `zheermao22` ran on Hyper:
  `deepseek-v4.1-flash-1017` 126 calls (132 on NeuralWatt) and `glm-5.3-flash` 56 calls (58),
  by the `SPEND` lines.
  Hyper bills in credits and its `SPEND` lines carry no cost.
- Plan energy.
  The end line (4211) reads:

  ```text
  NeuralWatt PLAN ENERGY SPENT: 326.9 Wh of the owner's plan were charged between the start and the end of the run; anything else done on the account in that time is in this figure too
  ```

  (`zheermao22`:
  `no plan energy was spent`).
  This is the largest figure of the series:
  `CuspariaKLSY22` read 37.3 Wh and `XingZ6017` 54.3 Wh,
  and 326.9 Wh is 6.0 times the larger of the two.
  The end baseline would stand near 5.9587 kWh (computed,
  no line prints it).

The contradiction,
as three numbers:

- the run made no NeuralWatt request,
  so none of the 326.9 Wh is this run's use of the covered models
  (inferred from the absence of a `SPEND` line;
  a request that failed before it settled would also leave none,
  but the log holds no NeuralWatt request line of any kind);
- the free-energy account stood at 555.188 Wh used at the first and the last reading,
  so the covered-model draw on the free energy in the window was zero;
- the baseline of 5.6318 kWh is 1.828 kWh higher than the end baseline of `XingZ6017`
  (about 3.8036 kWh,
  computed in its providers file,
  line 355),
  so the plan figure moved by that much with no pass in this record between them.

Whatever drew the 326.9 Wh drew it on the plan and not on the free energy,
and it began after 12:31:32 UTC.
`neuralwatt-quota.ts:9` to `26` says that figure cannot separate the owner's other account use
(read in the `CuspariaKLSY22` provider read,
which cites it);
this read did not re-read that file.
The log cannot name the source of the 326.9 Wh.
The owner settled it on 2026-10-22,
after this read:
the NeuralWatt key of the owner's dsh harness
(`package/dsh-plugin/neuralwatt`,
which reads `NEURALWATT_API_KEY`)
is on the same account,
so the plan figure counts that harness's use as well as the run's.
It is not a code defect,
and it is the third run in a row with a nonzero plan figure.

### Seats, units and meters

The `SEAT` lines (12) print after the `DONE` line (4212 to 4223):

- `google.gemma-4-e2b`:
  asked 0,
  unreachable 112
  (`zheermao22`:
  110).
- `deepseek-v4.1-flash`:
  asked 126,
  usable 126
  (133).
- `gemma-4-26b-a4b-it`:
  asked 123,
  usable 123
  (129).
- `hf:zai-org/GLM-5.3-Flash`:
  asked 58,
  usable 58
  (asked 59,
  usable 56,
  unusable 3,
  abandoned 1).
- `glm-5.3`:
  asked 70,
  usable 70
  (75).
- `minimax-m3`:
  asked 141,
  usable 141
  (142).
- `inception/mercury-2.5`:
  asked 121,
  usable 118,
  unusable 3,
  abandoned 0
  (asked 107,
  usable 106,
  unusable 1,
  abandoned 11).
- `hf:Qwen/Qwen3.8-27B`:
  asked 104,
  usable 104,
  abandoned 6
  (asked 138,
  usable 138,
  abandoned 10).
- `hf:moonshotai/Kimi-K3`:
  asked 124,
  usable 124
  (asked 147,
  usable 147,
  abandoned 1).
- `google.gemma-4-31b`:
  asked 0,
  unreachable 3
  (3).
- `xiaomi/mimo-v2.6-flash`:
  asked 28,
  usable 28
  (28).
- `typesafe/jev-1.13`:
  asked 30,
  usable 30
  (asked 28,
  usable 28).

Totals:
925 asked,
922 usable,
3 unusable,
0 threw,
6 abandoned,
115 unreachable (a script over the `SEAT` lines).

Reading them:

- `threw=0` on all 12 lines;
  `SEATS DARK` does not print (`rg --count` finds none).
  The `unreachable=` figures sum to 115.
  The 112 `seat unreachable` warn lines are all `google.gemma-4-e2b`;
  the 3 on `google.gemma-4-31b` are the picture-read refusals
  (`no provider can take google.gemma-4-31b: every provider serving this model is out of budget`,
  lines 287,
  305 and 330).
- The three `unusable` for `inception/mercury-2.5` are three `schema-mismatch` warns
  (content parsed as JSON but failed the caller schema guard)
  with the raw reply `[]` twice (lane contest slices 3 and 14,
  lines 3176 and 3313) and `["text"]` once (consolidation slice 4 select,
  line 3483);
  each was followed by `no re-ask elsewhere`,
  because OpenRouter is the only provider that serves the model with budget.
- The abandoned seat is Qwen alone,
  six times,
  each cut with `0 answer chars` and 31,990 to 62,107 reasoning chars after 129,943 to 273,389 ms
  (stream lines 386,
  537,
  897,
  1148,
  1394 and 1693;
  warn lines 390,
  545,
  920,
  1158,
  1425 and 1711).
  This is runaway reasoning,
  not the comment-only stream of the Mercury class,
  and it is the same shape `XingZ6017` named N2.
  The `SEAT` tally does not count an abandoned call as asked,
  as in the sibling reads.
- The asks and the `SPEND` lines do not close one to one.
  The `SEAT` lines count 925 asked;
  the log has 930 model stream lines
  (1159 stream lines less 229 meter reads:
  `quota`,
  `credits`,
  `quotas`,
  `models` and `usage-by-model`),
  of which the 6 cut Qwen streams have no `SPEND` line,
  leaving 924 `SPEND` lines.
  The 15 picture reads and 12 re-asks sit inside the 930,
  and whether the 925 asks count them was not established;
  seat by seat the difference does not close
  (`deepseek-v4.1-flash`,
  `glm-5.3` and Mercury close exactly;
  `minimax-m3` asked 141 against 140 streams,
  `hf:zai-org/GLM-5.3-Flash` asked 58 against 59,
  and Kimi asked 124 against 122).
  The remainder is unreconciled,
  and a reply that ended without a usage record is a guess.
- `JUDGE SEATS` prints 7 lines (11 in `zheermao22`;
  lines 210,
  268,
  632,
  633,
  2680,
  3153 and 3350).
  The first (12:31:33 UTC) reads `neuralwatt=dry synthetic=wet bedrock=dry hyper=wet openrouter=wet`
  and `unserved=google.gemma-4-e2b,google.gemma-4-31b`,
  with `readers=6 seeing=5 writers=9 roster=9`.
  The lines at 3153 and 3350 (13:13:53 and 13:15:06 UTC,
  after Synthetic went dry) read `synthetic=dry` and
  `withheld=hf:Qwen/Qwen3.8-27B,hf:moonshotai/Kimi-K3`
  (the two classed alike,
  as in `zheermao22`),
  `wide=6 select=6 slate=6 readers=5 seeing=4 writers=8 roster=8`.
  All 7 carry `threw-withheld=none`.
  Kimi is named in `withheld=` and still served 35 calls on Hyper after the dry reading:
  `withheld=` names a model that lost at least one seat (`CuspariaKLSY22` providers read,
  lines 240 to 249),
  and the `select=6 slate=6` against `late=7` of line 3153 matches that read.
  The `bedrock=dry` of this run is on every line,
  and the `METERS` lines now read `bedrock=dry` too
  (see Bedrock).
- The re-seat lines:
  `chunk re-seated under a hold` prints 0 times (0),
  `slice re-seated under a hold` 0 times (2),
  and `translate lane re-seated before starting` once (line 2681;
  1).
  In all,
  `re-seated` is on 1 line against 3.
  Line 2681 lists `google.gemma-4-e2b` among the eight writers although its only provider was off,
  as the same line did in the sibling reads.
  Synthetic went dry during the translate lane (line 2842),
  before the lane contest began,
  and the two consolidation re-seat lines of `zheermao22` did not print;
  the log does not say why.
- Did any quorum close short?
  No.
  All 14 introduced-defect probe rounds read `3/3 heard` (a script over the round lines),
  with no `bench short of quorum` line,
  and none of the 15 absolute naturalness reviews read `quorum-not-met`
  (10 acceptable,
  5 unacceptable;
  `5/5` to `5/7 usable` against `quorumNeeded=4`;
  `zheermao22`:
  16 reviews,
  `4/5` to `5/8`).
  116 of the 205 rounds heard fewer voices than their bench.
  Two lane-contest rounds closed on `4/6` with `0ms in grace`:
  slice 3 after 62,423 ms (line 3329) and slice 14 after 24,569 ms (line 3337).
  That is the shape `XingZ6017` named N1 (no grace runs when the quorum equals the live bench),
  but the log does not show which voices were missing or whether any stream was hung,
  so it is only a candidate recurrence.

Bedrock,
Hyper,
Synthetic,
OpenRouter:

- Bedrock:
  the provider was off.
  The launch printed `PROVIDER OFF bedrock: the owner ruled on 2026-10-20 to leave Bedrock off rather than issue
  a new key ...` (line 198),
  so no credential was refused and no `SEAT CREDENTIAL REFUSED` line printed
  (`zheermao22`:
  the first refusal was at 21:05:47 UTC).
  `METERS` reads `bedrock=dry` on all 38 lines and `JUDGE SEATS` reads `bedrock=dry` on all 7,
  so the two now agree
  (`zheermao22` flagged that `METERS` read `bedrock=wet` while `JUDGE SEATS` read `bedrock=dry`);
  the `METERS` lines no longer carry the `bedrockUsd` field.
- Hyper:
  wet all run (`hyper=wet` on 38 of 38 `METERS` lines);
  `hyperBalance` 250 at the first line and 194 at the last (56 credits;
  `zheermao22`:
  250 to 204).
  The `SPEND provider=hyper` lines count 555 (368):
  `minimax-m3` 139,
  `deepseek-v4.1-flash-1017` 126,
  `gemma-4-26b-a4b-it` 118,
  `glm-5.3` 70,
  `glm-5.3-flash` 56,
  `kimi-k3` 35 and
  `qwen3.8-27b` 11
  (a script over the `SPEND` lines;
  `zheermao22`:
  `minimax-m3` 140,
  `gemma-4-26b-a4b-it` 123,
  `glm-5.3` 75,
  `qwen3.8-27b` 14,
  `kimi-k3` 12,
  `glm-5.3-flash` 3 and `deepseek-v4.1-flash-1017` 1).
- Synthetic:
  weekly meter 3.97 percent at the first reading,
  `synthetic=dry` from the `METERS` line at 13:11:26.103 UTC (line 2842),
  0 percent at the last.
  There was no `HTTP 429`,
  no `markRefused` and no `HTTP 402`:
  the dry verdict came from the meter alone,
  where `zheermao22` had Qwen and Kimi answering `HTTP 429` first.
  One Qwen call settled after the dry reading (line 2862).
  Synthetic took 178 calls (Qwen 91,
  Kimi 87) and 220,287 completion tokens (a script over the `SPEND` lines;
  `zheermao22`:
  254 calls,
  414,163 tokens).
- OpenRouter:
  4.10 USD at the first `METERS` line to 3.99 USD at the last;
  the 191 `SPEND provider=openrouter` lines sum to 0.1178 USD
  (`inception/mercury-2.5` 121 calls,
  0.0747 USD;
  `zheermao22`:
  0.1344 USD over 192 calls,
  mercury 0.0816 USD over 119).
  The balance is the thing to watch:
  `XingZ6017` spent 9.218 USD of it (13.42 to 4.15 USD),
  and 3.99 USD is left.

The condition against `zheermao22`:
one of the three paid or metered providers (NeuralWatt) was dry from the start and one (Synthetic) went dry at
the 40th minute,
and the run was nonetheless 29.6 min shorter,
because the Mercury stalls and the other long rounds of `zheermao22` did not recur
(see "The Mercury recheck" and "Run facts").

### The Mercury recheck

The owner asked for a recheck of `inception/mercury-2.5`.
The question is whether its streams stalled,
and whether any reached the 90000 ms first-content window.

- The `SEAT` line (log line 4218) reads
  `SEAT inception/mercury-2.5 asked=121 usable=118 unusable=3 threw=0 silent=0 abandoned=0 unreachable=0`
  (`zheermao22`:
  `asked 107,
  usable 106,
  unusable 1,
  abandoned 11`).
- Streams:
  121 `stream inception/mercury-2.5` lines (a script over the stream lines),
  all 121 `completed`,
  none `cut` (`zheermao22`:
  11 abandoned at the grace),
  and 121 `SPEND` lines
  (`provider=openrouter`,
  endpoint `Inception`,
  0.074681 USD,
  295,463 completion tokens).
- First content,
  over the 119 completed streams that carry a first-content reading:
  maximum 12,820 ms (line 2112),
  median 6,504 ms,
  90th percentile 8,683 ms,
  95th percentile 10,556 ms.
  The next four are 12,406 ms (line 227),
  11,834 ms (line 3502),
  11,277 ms (line 2081) and 10,650 ms (line 3457).
- Streams that hit the 90000 ms window:
  none.
  No Mercury stream had a first content,
  or an elapsed time,
  at or over 90,000 ms;
  the longest elapsed time was 13,252 ms (line 2112),
  and the median elapsed time 6,522 ms.
  The largest first content (12,820 ms) is 7.0 times shorter than the window,
  and the longest elapsed time 6.8 times.
  The window itself is not printed anywhere in the log,
  and the artifact's `callConfig` records only the shared windows:
  `perCallTimeoutMs: 360000`,
  `streamFirstContentMs: 180000`,
  `streamIdleMs: 120000`
  (`zheermao22`:
  `perCallTimeoutMs: 360000, streamFirstByteMs: 600000, streamIdleMs: 600000`).
  A hit would be a stream that carries no content until the window ends,
  and the log holds no such stream.
- First byte:
  maximum 1,629 ms,
  median 994 ms.
- Two streams carried no content at all (lines 3731 and 3790,
  consolidation slice 6,
  13:17:44 and 13:17:53 UTC):
  442 and 492 raw chars,
  2,162 and 3,146 ms,
  `0 content chars`.
  Each was followed by a warn
  `stream carried a provider failure instead of a completion: code 502, type provider_unavailable ...`
  (lines 3733 and 3792) and a retry in 745 ms and 685 ms (`attempt 1 of 5`),
  and each `SPEND` line carries `estimated=abandoned` (lines 3732 and 3791).
  I read that label as a cost estimate for a stream that ended with no usage record,
  and the `SEAT` tally's `abandoned=0` as consistent with it,
  because both streams were retried inside the same ask;
  that is an inference,
  since the tally code was not read.
  The raw counts are not multiples of 25,
  so they are not the comment-only signature of the Mercury class of `zheermao22`;
  they are fast failures.
  `zheermao22` had four such 502 requests besides its 11 comment-only streams.
- The three unusable answers are `schema-mismatch` replies with an empty or wrong-shape JSON value
  (see "Seats,
  units and meters"),
  and `readJsonOutcome` read one reply with 380 trailing characters (line 4065).
  None is a stall.
- Cost to the run:
  no round waited on a Mercury stream (all six `abandoned` lines are Qwen),
  and the two empty streams cost 2 to 3 s each before the retry.
- The series for this seat,
  from the sibling reads:
  `zheermao22` 11 abandoned of 107 asked (21:59 to 22:24 UTC);
  `CuspariaKLSY22` 33 abandoned out of 69 streams (22:25 to 23:04 UTC);
  `XingZ6017` 53 cut out of 1535 streams,
  105 with no content and 21 that ran 290 s or more (23:05 to 03:34 UTC);
  this run 0 stalled out of 121 (12:31 to 13:21 UTC).
- What the recheck shows,
  and what it does not.
  It shows Mercury answering in 13 s or less on every one of 121 streams in a 49-minute window.
  It does not exercise the 90000 ms window,
  because no stream came near it,
  and so it neither confirms that the window works nor shows that it is needed.
  The stalls of `zheermao22` began after 21:59 UTC;
  this run ran in a different part of the day,
  so a provider-side time effect and the window cannot be told apart from one run.
- Side check on the window itself,
  from the worktree files (not from the log):
  the Mercury card names `firstContentMs: 90_000` (`model-cards.ts:431`,
  with the ledger B809 comment),
  `openrouter-client.ts:461` to `462` passes the card's own window to the request,
  and the shared windows are `STREAM_FIRST_CONTENT_MS = 180_000` and `STREAM_IDLE_MS = 120_000`
  (`stream-idle-guard.ts:89` and `:106`),
  which agree with the artifact's `callConfig`.
  The branch ref file of the worktree reads `8864316eb29b9848d95ceeaf4c039671c0c430e8`,
  the `START` tip;
  the working tree was not compared with that tip,
  so that the run used the card's window is read from the match of the tip and not proved.
- For scale,
  the only stream of any model that would have crossed 90000 ms on first content is `glm-5.3-flash` on Hyper
  (122,849 ms,
  line 2134,
  repair slice 9 editor).
  The next four are Hyper `gemma-4-26b-a4b-it` at 82,446 ms (line 2623),
  Kimi at 66,710 ms (line 3881),
  Kimi on Synthetic at 62,308 ms (line 2617) and 61,922 ms (line 2613).
  None reached the shared 180000 ms window.

### Why repairIssues rose from 31 to 42 and repairFindings from 258 to 270

The artifact shows why (a script over the issue lists and findings of the artifact):

- Issues by slice:
  slice 0 holds 8 (`zheermao22`:
  7),
  slice 4 holds 7 (2),
  slice 9 holds 7 (4),
  slice 5 holds 5 (6),
  slices 1,
  6 and 2 hold 3,
  3 and 2,
  and slices 3,
  8,
  11,
  12,
  13 and 14 hold 1 or 2.
  The status split is 21 accepted (22),
  18 rejected (7) and 3 `needs-human` (2).
  The rise is the rejected claims:
  11 more rejected issues,
  against one fewer accepted.
- Rejected issues by slice:
  slice 5 holds 4,
  slices 1 and 6 hold 3 each,
  slice 4 holds 2,
  and slices 0,
  2,
  9,
  11,
  12 and 13 hold 1 each.
- The 21 accepted issues are 15 minor and 6 major.
  The six major ones are one on slice 0 (the "I can't wait that long" reading as impatience),
  four on slice 9 (the plural "One Among Us members" and its name,
  the same subject as the two major claims of `zheermao22` on slice 9) and one on slice 11
  (the dropped name 那些秋叶).
- The three `needs-human` issues are all on slice 0 and all major:
  the question addressed to Zheermao by name,
  the "In addition" clause that distorts the favourite-flavour question,
  and the generalized handle 折耳猫 rendered as "your".
- `repairResolved` 21 of 21 accepted
  (`zheermao22`:
  21 of 22);
  no accepted issue is unresolved.
- `repairFindings` is 12 higher,
  by kind:
  `panel-packet` 68 (59),
  `stage-voice-lost` 37 (34),
  `stage-roster-incomplete` 35 (32),
  `select-self-vote` 10 (10),
  `duplicate-issue-merged` 5 (8),
  and one-offs:
  `empty-quote`,
  `introduced-repetition` and `content-survival` (148 of 163 distinctive archive words kept).
  There is no `refine-rolled-back-by-probe` and no `refine-declined` finding.
  The `refine-skip` findings (16) and `refine-skipped` (8) record blocks the refine pass skipped
  (`not-a-paragraph`,
  `0 eligible paragraphs`).
- Beside the findings,
  the log carries 30 `panel ballot ... irregular` warns
  (whether they are counted in the 68 `panel-packet` findings was not checked):
  29 from `gemma-4-26b-a4b-it` (13 `unknown-regrade-severity`,
  16 `group-index-out-of-range`) and 1 `group-index-out-of-range` from `inception/mercury-2.5`
  (log lines 804 to 2152).
  Counted from the logs by the independent check:
  `zheermao22` 28,
  `XingZ6017` 96,
  `CuspariaKLSY22` 6,
  `CuspariaKLSY21` 7,
  `zheermao21` 2.
  So 30 is 2 more than `zheermao22` and well under `XingZ6017`.
