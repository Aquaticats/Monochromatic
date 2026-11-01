# OpenRouter and the corpus passes: the run of zheermao22 on build 2a336603bf (2026-10-21)

What was read on 2026-10-21 of the run of the first pass after the Gemma readers were lifted:
the log,
the NeuralWatt meter,
the seats and the providers,
why the repair counts rose,
whether the new rulings fired,
the traces of the findings to code,
and the class verdict.
Entry:
zheermao22 (entry `zheermao101`).
The archive block review of the letter and the email is in
[the letter of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-letter.md),
and the shipped page,
line by line,
is in
[the page of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-page.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of the restart share are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).

## zheermao22 read, 2026-10-21, 23:16 UTC: SETTLED in 79.0 min of pass; the email and the quote recovered, the letter regressed; three code classes and one design class

`zheermao22` (entry `zheermao101`) ran on tip `2a336603bf38a6a53aeb4f3e70f0a3447acf5487`
and pipeline `sha256-tree-v1:5d4265dfab83b5a6e61b5b8c23d597519c7ca24293065473c883ec82af54c78c`,
and its `START` line (log line 196) carries both values as given;
the artifact's `tip` and `pipelineDigest` carry the same two.
`zheermao21` ran on `3637592bd4`
and pipeline `sha256-tree-v1:06c97a69446dc452fa2c602da834c36febb7f0d7b4b1010b5dfa9fab81097eec`.
[The handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-the-landings-the-zheermao22-restart-and-the-state-after-k4.md) records the launch at 21:05:30 UTC;
the `START` line carries no time.
The log's first timestamped line is 21:05:44.369 UTC (`republish: 0 settled pages judged`)
and its last is 22:24:46.483 UTC (a `stream quota` line);
the log has 4737 lines (`wc --lines`).
The corpus input is the pin `a41fc607ea` (the artifact's `corpusSha` reads `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`).

The reader read the log,
the run directory
(the artifact `zheermao101.json`,
961 stored reply-only prompt payloads
and the shipped `fixed/people/zheermao101/page.en.md`)
and the source of the entry in the one-among-us data tree.
The three photos were read as images.
No file in any git checkout was edited,
no pass,
probe,
gate,
build,
test or provider call was run,
and no `.env*`,
auth or `environ` file was read.
Code was read with `git show` and `git grep` at `2a336603bf`,
not from a working tree.
Every count names its method:
a script that counts log lines matching a pattern,
the same as `rg --count`
(a second script splits the counts that mixed `[info]` and `[warn]` lines).

The data tree is not a git checkout,
so equality of its `page.en.md` with the pin is not verified.
What could be checked:
the artifact's `preparation.archiveText` is the archive after the review round,
and a `diff` of it against the working-tree `page.en.md` differs only at working-tree lines 49 to 126,
which are the label of the letter,
the letter,
the label of the email and the email.
Everything the review did not touch agrees with what the lanes used.
Where this record says "the archive",
it means the working-tree `page.en.md` (before the review);
where it says "the archive after the review",
it means the artifact's text.

### Run facts

The `TALLY` line (log line 4720) reads:

```text
TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired repairIssues=31 repairAccepted=22 repairResolved=21 repairFindings=258 repairChanged=10 translateStatus=complete translateChanged=10 documentsDiffer=11 pageChanged=12 pageSilent=0 alignmentFindings=61 selection=contested ms=4741311
```

The `DONE` line (4722) reads `DONE processed=1 of pending=1; artifacts=1/1 declined=0 ... elapsed=4741466ms`,
which is 79.0 min.
`DESTINATIONS zheermao101 source=2 page=3 dropped=0` (4721),
and `publish: wrote 15 slices into a page of 6117 characters` (4719,
22:24:45.822 UTC;
`wc --chars --bytes` gives 6117 characters and 6255 bytes,
148 lines;
`zheermao21`:
5869 and 6005).
`FRONT MATTER entry=zheermao101 authority=archive` (21:05:46 UTC).

Beside `zheermao21` (figures from its read):

- `repairIssues` 31 against 28.
- `repairAccepted` 22 against 18,
  `repairResolved` 21 against 16.
- `repairFindings` 258 against 321.
- `repairChanged` 10 on both;
  `translateChanged` 10 on both.
- `documentsDiffer` 11 on both,
  `pageChanged` 12 against 10,
  `pageSilent` 0 on both.
- `alignmentFindings` 61 against 13.
- `selection=contested` on both.
- `ms` 4741311 against 2443411,
  so 38.3 min longer.

Where the extra 38.3 min went,
from the log timestamps (a script that splits the log at marker lines;
this run,
then `zheermao21`):

- Start of log to `both lanes over 15 slices, repair first` (line 662,
  21:26:30 UTC):
  20.8 min against 27.3 min.
  The local photo read that took 98 s,
  120 s and 47 s in `zheermao21` took 1.6 s,
  0.9 s and 0.7 s here
  (from the phase start or the previous photo's `corroborated` line to its `read N characters without a model` line,
  log lines 277,
  296 and 315),
  and the three read the same 515,
  422 and 388 characters.
  The whole picture phase,
  with five model readers,
  ran 21:06:56 to 21:08:43 UTC.
  The archive block reviews ran 21:09:04 to 21:25:49 UTC,
  16.7 min (`zheermao21`:
  18.5 min).
- The repair lane,
  to the first `SLICE-START lane=translate` (line 2641,
  21:51:56 UTC):
  25.4 min against 7.4 min.
  The chunk pass ends with the last repair `SLICE-COST` at 21:46:11 UTC (19.7 min);
  the refine pass,
  with its select,
  checker and probe rounds,
  runs on to 21:51:56 UTC.
- The translate lane,
  to `lane contest: slices that differ: 11 of 15` (3184,
  21:57:45 UTC):
  5.8 min against 1.6 min.
- The lane contest,
  to `consolidation: 11 contested slices to settle` (3427,
  22:03:31 UTC):
  5.8 min against 0.4 min.
- Consolidation to `publish` (22:24:45 UTC):
  21.2 min against 4.0 min.
- So the repair lane (+18.0 min) and consolidation (+17.2 min) are 35.2 of the 38.3 min;
  the translate lane (+4.2) and the contest (+5.4) add 9.6;
  preparation is 6.5 min shorter.

What lengthened the lanes:

- Synthetic stayed wet to 22:14:48.707 UTC (69.1 min into a 79.0 min run;
  `zheermao21`:
  31.4 of 40.7 min) and Hyper was wet all run.
  `hf:Qwen/Qwen3.8-27B` and `hf:moonshotai/Kimi-K3` were therefore heard in nearly every round:
  `SEAT` lines count 138 and 147 asked,
  against 22 and 24 in `zheermao21`,
  where Synthetic went dry 4.0 min into the lanes.
- A round waits for a seat that has not answered until the grace after quorum ends:
  120 s for ordinary rounds,
  180 s for writer rounds (`WRITER GRACE built in`,
  log line 197).
  The log's 23 `abandoned ... ms after quorum` lines are
  `inception/mercury-2.5` 11,
  `hf:Qwen/Qwen3.8-27B` 10,
  `hf:moonshotai/Kimi-K3` 1 and `hf:zai-org/GLM-5.3-Flash` 1,
  against 6 in `zheermao21`,
  all Qwen.
- A script over the round lines counts 30 rounds with at least 100 s in grace,
  against 7 in `zheermao21`.
  Summed over rounds,
  the grace is 62.4 min against 14.0 min;
  that sum is over rounds that overlap (up to four slices were in flight in each lane,
  a script over the `SLICE-START` and `SLICE-COST` lines),
  so it is a measure of the cause and not of wall clock.
  Four rounds waited 140 s or more:
  the letter's recovery round (180 s,
  21:21:58 UTC),
  the slice 8 editor recovery round (175 s,
  21:41:22 UTC),
  slice 6 `produceConsolidations` (175 s,
  22:15:09 UTC) and the translate round of slice 0 (140 s,
  21:54:34 UTC).
- Wall clock is better read from `SLICE-COST` (one line per slice per lane):
  repair slice 8 746 s,
  slice 0 515 s,
  slice 4 387 s,
  slice 5 364 s,
  slice 9 323 s
  (`zheermao21`:
  largest 368 s,
  slice 0);
  consolidation slice 4 612 s,
  slice 3 596 s,
  slice 6 548 s,
  slice 0 476 s,
  slice 1 458 s,
  slice 5 444 s,
  slice 8 407 s
  (`zheermao21`:
  largest 112 s,
  slice 5).
- Slice 8's repair path is the single slowest piece.
  Its `glm-5.3-flash` editor call on NeuralWatt ran to the 18,316-token cap twice (21:37:15 and 21:40:06 UTC,
  0 content characters each),
  the Hyper re-ask of the same voice also ran to its cap (21:41:22 UTC),
  and the editor recovery round heard 0 of 1 voices after 174,954 ms
  (log lines 1984 to 2225;
  the class is traced as K1 in the section "Traces to code").
- In consolidation,
  `inception/mercury-2.5` accounts for 1701 s of stream time over 48 calls
  (`zheermao21`:
  462 s over 65),
  and `hf:Qwen/Qwen3.8-27B` for 1948 s over 26 calls (from the phase script’s output).
  Eleven of mercury's streams carried no content at all
  (see "Traces to code").

Counts over the log,
each by a counting script (the `zheermao21` figure in brackets is from its read):

- `[warn]` lines:
  210 (491);
  `warn` appears on 210 lines,
  so no summary line quotes one.
- `[error]`:
  none.
- `DEFECTS`,
  `REATTEMPT`,
  `STALLED`,
  `mdx-downgraded`,
  `StreamOverrunError`:
  none of the five.
- `Interrupted` and `TranslationRepairInterruptedError`:
  none.
- `NoProviderForModelError`:
  none as a string;
  the refusals are worded `no provider can take <model>: ... seat unreachable`.
- `seat unreachable`:
  110 lines (420),
  all `google.gemma-4-e2b`.
- `HTTP 401`:
  3 lines (3),
  all Bedrock;
  `HTTP 402`:
  none (none);
  `HTTP 429`:
  10 lines (10),
  all from 22:14:42 to 22:14:55 UTC.
- `schema-mismatch`,
  `SyntaxError` or `unusable pairing`:
  27 lines,
  of which 23 are `[info]` lines (`reaskElsewhereNudged` lines that carry the word;
  38 such lines in all)
  and 4 are `[warn]` (`zheermao21` counted 16 without that split).
- `threw`:
  21 lines,
  of which 9 are `JUDGE SEATS` lines carrying `threw-withheld=none` and 12 are the `SEAT` and other lines;
  every `SEAT` line reads `threw=0`.
- `closed on a short bench` and `bench short of quorum`:
  none (none).
- Decision lines reading `keeping the fallback` or `keeping the repaired text`:
  8 (12).
- `abandoned ... ms after quorum`:
  23 (6).
- `refused by the page grammar`,
  `MdxParseError`:
  none (none).
- `archive-revision-refused`:
  2 lines (2):
  one `[warn]` (log line 548,
  `gemma-4-26b-a4b-it` on the letter) and the finding that line 576 quotes;
  `introduced repetition`:
  2 lines (1),
  the same warn printed by the repair and the translate lane (log lines 2635 and 3174).
- `archive review dropped the editorial-context reply`:
  6 lines (0),
  all on the two label blocks.
- `refine-rolled-back-by-probe`:
  1 line (0),
  repair slice 11 at 21:50:06 UTC.
- The build output carries no `UNRESOLVED_IMPORT` line in this log (3 in `zheermao21`).

Counting effects,
as `zheermao21` explained its own:

- `alignmentFindings` 61 against 13 is the kept review findings (ledger B714):
  the four block reviews recorded 10 (the email),
  10 (its label),
  12 (the letter) and 13 (its label) findings
  (the `review recorded N findings` lines 427,
  487,
  576 and 628),
  45 of the 61.
- `stage-voice-lost` 34 against 97 and `stage-roster-incomplete` 32 against 66
  (a script over the issue lists of each artifact):
  Hyper and Synthetic were wet,
  so `glm-5.3` (asked 75 times,
  `SEAT` line;
  unreachable 93 times in `zheermao21`),
  Qwen and Kimi were seated and heard instead of recorded as lost.
- `seat unreachable` 110 against 420 is the same effect.

### NeuralWatt: baseline, stop and energy

Baseline.

- One `takeBaseline` line,
  at 21:05:45.428 UTC:
  `NeuralWatt quota at the start of the run: 3.712 kWh charged this period`.
  It is the plain wording.
  All 58 `stream quota:` lines of the run read `completed` (`rg --count`;
  no `cut` or `failed` line),
  so the 60 s retry rule never had a failure to act on.
- `decideNeuralWattDayAnchor` (line 204):
  the day starts at 19:15:08.000 UTC,
  current day began 2026-10-21T19:15:08.000Z.
  This run began 1 h 50 min into that day,
  so the first `METERS` line reads `neuralwattWh=550` and the first call settled at 21:09:05 UTC
  (`neuralwattAccountWh=0.083`).
  `zheermao21` ran on the previous day's allowance (its first reading was 468.773 Wh).
- End line (4724):
  `NeuralWatt plan energy: no plan energy was spent between the start and the end of the run; the free daily energy is not in this figure`.

Stop.

- No stop line fired.
  `rg` for `markRefused` finds two lines,
  both `synthetic` (22:14:53 and 22:14:55 UTC);
  none names NeuralWatt.
  All 57 `METERS` lines read `neuralwatt=wet`.
- The lowest remaining reading was 509.607 Wh,
  so the meter's dry rule (less left than `NEURALWATT_SMALLEST_ESTIMATE_WH`) was never near.

Energy and the meter:

- A script that reconciles the meter against the log found
  190 settled calls,
  40.9099 Wh measured,
  and 190 `SPEND provider=neuralwatt` lines (`rg --count`).
  Estimated for the same calls:
  29.4515 Wh,
  so measured is 1.39 times the estimate
  (`zheermao21`:
  247 calls,
  44.7156 Wh,
  1.23 times).
- `deepseek-v4.1-flash`:
  132 calls,
  12.1126 Wh measured (9.8481 Wh estimated).
  `glm-5.3-flash`:
  58 calls,
  28.7973 Wh measured (19.6034 Wh estimated).
- Energy per call:
  0.215 Wh against 0.181 Wh in `zheermao21`.
- Fifteen calls of 0.5 Wh or more total 22.8912 Wh,
  56 percent of the run's NeuralWatt energy (a script that ranks the NeuralWatt calls by energy).
  The four calls that ran to 18,316 completion tokens total 12.0174 Wh,
  29 percent:
  the letter at 2.7184 Wh (21:17:32 UTC) and 5.0034 Wh (21:20:36 UTC),
  slice 8 at 1.8466 Wh (21:37:15 UTC) and 2.4490 Wh (21:40:06 UTC).
  The 5.0034 Wh call is the largest of the run
  (`zheermao21`:
  3.2377 Wh;
  `zheermao20`:
  4.1361 Wh,
  the same model on the same letter block).
  The largest `deepseek-v4.1-flash` call is 1.0122 Wh (21:09:36 UTC,
  8,892 tokens),
  and its capped call is 0.6835 Wh (13,082 tokens).
- `neuralwattWh`:
  550 at the first `METERS` line to 509.607 Wh at the last (line 4652);
  a fall of 40.393 Wh.
  The 185 calls settled between those lines sum to 40.3929 Wh,
  so the meter's fall equals the sum of the settled calls to the third decimal.
  Account side:
  the last line's `neuralwattAccountWh` is the same 40.393 Wh (550 less 509.607).
  The 5 calls after the last `METERS` line sum to 0.517 Wh,
  so the meter would stand near 509.09 Wh at the end (computed,
  no line prints it).
- The meter plus the account does not equal 550 on 13 of the 57 lines (a script that sums the meter and the account):
  gap 0.372 on six lines,
  0.373 on two,
  0.076 on two and 0.001 on three.
  0.372 is the `glm-5.3-flash` estimate (0.3725 Wh) and 0.076 the `deepseek-v4.1-flash` estimate (0.0758 Wh),
  so each is one call in flight at the moment of reading:
  the meter subtracts the hold of a call still running.
  This confirms Finding 9 of [the causes document](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md),
  and closes the open item on this gap in the `zheermao21` read.
  `neuralwattReckonedWh` reads 0.000 on every line.
- Plan energy:
  the end difference is 0 kWh,
  so no plan energy was spent.

### Seats, units and meters

The `SEAT` lines (12) print after the `DONE` line:

- `google.gemma-4-e2b`:
  asked 0,
  unreachable 110.
- `gemma-4-26b-a4b-it`:
  asked 129,
  usable 129.
- `deepseek-v4.1-flash`:
  asked 133,
  usable 133.
- `glm-5.3`:
  asked 75,
  usable 75
  (`zheermao21`:
  asked 0,
  unreachable 93).
- `inception/mercury-2.5`:
  asked 107,
  usable 106,
  unusable 1,
  abandoned 11.
- `hf:zai-org/GLM-5.3-Flash`:
  asked 59,
  usable 56,
  unusable 3,
  abandoned 1.
- `minimax-m3`:
  asked 142,
  usable 142.
- `hf:Qwen/Qwen3.8-27B`:
  asked 138,
  usable 138,
  abandoned 10
  (`zheermao21`:
  asked 22,
  abandoned 6,
  unreachable 160).
- `hf:moonshotai/Kimi-K3`:
  asked 147,
  usable 147,
  abandoned 1
  (`zheermao21`:
  asked 24,
  unreachable 19).
- `google.gemma-4-31b`:
  asked 0,
  unreachable 3.
- `xiaomi/mimo-v2.6-flash`:
  asked 28,
  usable 28.
- `typesafe/jev-1.13`:
  asked 28,
  usable 28.

Reading them:

- `threw=0` on all 12 lines;
  `SEATS DARK` does not print (`rg --count` finds none).
  The `unreachable=` figures sum to 113.
  The 110 `seat unreachable` warn lines are all `google.gemma-4-e2b`;
  the 3 on `google.gemma-4-31b` are the picture-read refusals (`credential refused this run (bedrock)`).
- The three `unusable` for `hf:zai-org/GLM-5.3-Flash` are the three `schema-mismatch (provider reported a truncating completion ...)` warns:
  the letter review (21:18:58 UTC) and the slice 8 editor (21:38:28 and 21:41:22 UTC).
  Its `abandoned 1` is the letter's recovery round (21:21:58 UTC).
- `JUDGE SEATS` prints 11 lines (49 in `zheermao21`).
  The first (21:05:46 UTC) reads `bedrock=wet hyper=wet unserved=none`.
  From 21:06:56 UTC (the picture phase) it reads `bedrock=dry` and
  `unserved=google.gemma-4-e2b,google.gemma-4-31b`.
  From 22:18:33 UTC (after Synthetic went dry) it adds
  `withheld=hf:Qwen/Qwen3.8-27B,hf:moonshotai/Kimi-K3`,
  the two classed alike this time
  (`zheermao21` classed Kimi `withheld=` and Qwen `unserved=`;
  Finding 10 of the causes document explains that difference).
  Nine of the 11 carry `threw-withheld=none`;
  the two re-seat lines do not carry the field.
  The lines carry a new `seeing=` field:
  6 at preparation (Bedrock wet) and 5 once Bedrock is held out.
- The re-seat line:
  `chunk re-seated under a hold` prints 0 times (23 in `zheermao21`),
  `slice re-seated under a hold` prints 2 times (19),
  and `translate lane re-seated before starting` once (line 2637).
  In all,
  `re-seated` is on 3 lines against 50.
  The two slice lines are at 22:18:33.116 UTC (the start of consolidation slice 12)
  and 22:19:04.018 UTC (the start of slice 14),
  right after Synthetic went dry (22:14:48 UTC);
  in `zheermao21` Synthetic went dry at 05:53:09 UTC,
  while the repair lane still had 23 chunk starts ahead.
  The count follows from when the hold began,
  and closes the "still not traced" item of the `zheermao21` read.
- Did any quorum close short?
  No.
  All 14 introduced-defect probe rounds read `3/3 heard` (`rg --count`),
  with no `bench short of quorum` line,
  and none of the 16 absolute naturalness reviews read `quorum-not-met`.
  The reviews read `4/5` to `5/8 usable` against `quorumNeeded=4`.

Bedrock,
Synthetic,
Hyper,
OpenRouter:

- Bedrock:
  the key was still expired.
  The first refusal is at 21:05:47.771 UTC (`SEAT CREDENTIAL REFUSED gemma-4-26b-a4b-it on bedrock ... HTTP 401 ... The security token included in the request has expired`).
  `METERS` reads `bedrock=wet` and `bedrockUsd=5.02` on every line,
  while `JUDGE SEATS` reads `bedrock=dry` from 21:06:56 UTC;
  as in `zheermao21`,
  the difference is reported and not diagnosed.
- Hyper:
  wet all run (`hyper=wet` on 57 of 57 `METERS` lines);
  `hyperBalance` 250 at the first line and 204 at the last.
  The `SPEND provider=hyper` lines count 368 (a script over the `SPEND` lines):
  `minimax-m3` 140,
  `gemma-4-26b-a4b-it` 123,
  `glm-5.3` 75,
  `hf:Qwen/Qwen3.8-27B` (as `qwen3.8-27b`) 14,
  `hf:moonshotai/Kimi-K3` (as `kimi-k3`) 12,
  `glm-5.3-flash` 3 and `deepseek-v4.1-flash-1017` 1.
  `zheermao21` had no Hyper line.
- Synthetic:
  weekly meter 10 percent at the first reading,
  dry at 22:14:48.707 UTC,
  0 percent at the last.
  `Qwen3.8-27B` and `Kimi-K3` answered `HTTP 429` from 22:14:42 UTC and retried;
  `markRefused` logged
  `synthetic: refused us while its meter reads dry ... held out for 300000ms` (22:14:53 and 22:14:55 UTC).
  There was no `HTTP 402`.
  Synthetic took 254 calls (Qwen 121,
  Kimi 133) and 414,163 completion tokens (a script over the `SPEND` lines).
- OpenRouter:
  13.59 USD at the first `METERS` line to 13.47 USD at the last;
  the `SPEND provider=openrouter` lines sum to 0.1344 USD over 192 calls
  (`inception/mercury-2.5` 119 calls,
  0.0816 USD).

The condition against `zheermao21`:
both Hyper and Synthetic were available for most of the run,
so the full panel of nine writers and judges was seated and heard.
The cost shows in the time (see "Run facts") and in the energy,
because the two NeuralWatt models are two of the nine.

### Why repairIssues rose from 28 to 31 and repairFindings fell from 321 to 258

The log and the artifact show why (a script over the issue lists of each artifact):

- Issues by slice (the script’s output):
  slice 0 holds 7 (`zheermao21`:
  10),
  slice 5 holds 6 (1),
  slice 9 holds 4 (2),
  slice 4 holds 2 (2),
  and slices 1,
  2,
  3,
  6,
  8,
  11,
  12,
  13 and 14 hold 1 or 2.
  The status split is 22 accepted (18),
  7 rejected (8) and 2 `needs-human` (2).
- Slice 5’s six issues are four accepted (the "friends" narrowing three ways and "always") and two rejected
  (three omission claims);
  slice 9’s four are two major (the singular "one member") and two minor (连夜 as "immediately").
- `repairResolved` 21 of 22 accepted:
  the one not resolved is slice 8’s orphaned-fragment claim
  (`chunk 8: repaired, 1/2 served accepted issues resolved`,
  21:46:11 UTC;
  the checkers split `fixed` / `not-fixed` on it,
  and `hf:Qwen/Qwen3.8-27B` was cut at the grace and not heard).
- `repairFindings` is 63 lower,
  by kind:
  `stage-voice-lost` 34 (97),
  `stage-roster-incomplete` 32 (66),
  `panel-packet` 59 (62),
  `select-self-vote` 10 (7),
  `duplicate-issue-merged` 8 (3),
  and one-offs:
  `refine-rolled-back-by-probe`,
  `empty-quote`,
  `refine-declined`,
  `introduced-repetition`,
  `content-survival`.
  The drop in voice-lost and roster-incomplete findings is the Hyper and Synthetic effect:
  fewer seats were unserved,
  so fewer stages recorded a loss.

### Whether the new rulings fired

The rulings since `zheermao21`:
polished text wins ties,
reason before vote everywhere,
tied finalists run off,
Gemma readers lifted,
the KaTeX shim.

- Reason before vote.
  Fired on every provider except Hyper.
  A script over the 961 reply payloads groups the vote-bearing replies (those carrying a `reason` key and
  one of `best`,
  `choice` or `acceptable`) by serving provider:
  NeuralWatt 63 reason-first and 0 vote-first,
  OpenRouter 63 and 0,
  Synthetic 90 and 0,
  all reason-first;
  Hyper 14 reason-first and 176 vote-first.
  In `zheermao21` the same count over its payloads was 376 vote-first and 0 reason-first.
  The Hyper key order is alphabetical:
  A second script finds 200 of 227 multi-key Hyper objects in alphabetical key order,
  against 0 of 118 on Synthetic,
  0 of 81 on NeuralWatt and 0 of 88 on OpenRouter.
  So Hyper’s order is a serialization artifact of its `tool_use` path,
  and Hyper replies can neither confirm nor refute the ruling;
  They are not called vote-first as a model behaviour.
  The cause is untraced:
  `anthropic-tool.ts:235` passes `responseFormat.json_schema.schema` as given,
  and `git grep` for `sort`,
  `stableStringify` and `canonical` in `provider/src` finds no sorting call,
  so the sort is either in the gateway or in code the read did not find.
  No probe was run (the brief forbade it).
- Polished text wins ties.
  Not exercised.
  Five polish gates ran:
  `polished 5, base 0`,
  `polished 2, base 3`,
  `polished 1, base 4`,
  `polished 3, base 2` and `polished 5, base 0`;
  none tied.
  The rule name `tie-polished-wins` exists in the settlement type
  (`consolidation-polish-gate-stage.ts:113` to `118`),
  and the new log line names the rule on every gate (`by rule polished-lead`,
  `by rule base-lead`),
  which fired five times.
  This record says "no tie occurred",
  not "worked".
- Tied finalists run off.
  Fired in two consolidation slices and once in the translate lane:
  - Slice 4:
    `judges tied at weight 1.5` (22:07:59 UTC),
    `the standing drew no ballot, so the same panel is asked again over the 2 tied finalists`,
    and the run-off decided (`candidate 1 from hf:Qwen/Qwen3.8-27B won weight 3`,
    22:08:13 UTC),
    as the 2026-10-11 ruling says ("Run off the tied finalists").
  - Slice 12:
    `winner drew only weight 1.5 ... against a minimum of 2.00` (22:19:09 UTC),
    the run-off,
    `judges tied at weight 2` (22:19:18 UTC),
    and the standing shipped,
    again as the ruling says ("the standing ships on a second tie").
    The line says "2 tied finalists" for what was a plurality (not a tie);
    `translate-runoff.ts:90` takes "the candidates tied at the top,
    or the leader and its runner-up",
    so the wording is the line’s and the behaviour is the code’s.
  - Translate lane slice 0:
    `judges tied at weight 2` (21:55:12 UTC),
    `declined-indecision; challenging same panel under distinct responsibility`,
    and a second round decided (`glm-5.3 won weight 4`,
    21:56:47 UTC).
    This is the older path with its older wording.
  - The lines `a strict majority of contest ballots called the archive flawed; a tied slate is run off rather than left to the standing` (slices 3 and 5,
    22:05:15 and 22:14:03 UTC) are announcements of a rule;
    neither slice tied (`composite ... won weight 3` and `deepseek-v4.1-flash won weight 2.5`).
- Ledger B731 (the polish gate sheet shows the accepted claims):
  fired.
  Four ballots on three slices cite an accepted claim:
  `hf:moonshotai/Kimi-K3` on slices 8,
  11 and 14,
  and `hf:Qwen/Qwen3.8-27B` on slice 8 (`rg` for `consolidation polish gate ballot`).
- Ledger B730 (polish tie to polished),
  B733 (the cache-refusal line):
  not exercised (no tie,
  no short probe).
- The Gemma readers lift:
  fired.
  `gemma-4-26b-a4b-it` read all three photos (530,
  478 and 419 characters;
  log lines 280,
  299 and 329) and each photo was `corroborated by 5 of 5 readers` (4 of 4 in `zheermao21`);
  the new `seeing=` field of the `JUDGE SEATS` lines reads 6 with Bedrock wet and 5 once Bedrock is held out
  (what the field counts was not read).
- The KaTeX shim:
  not exercisable on this entry;
  nothing in the log touches `katex`.
- B713 (the writers’ window):
  the launch line prints it (`WRITER GRACE built in`),
  and the letter’s recovery round waited 180 s;
  the review rounds closed on all voices heard or unreadable,
  so the window changed no outcome here.
- B714 (kept findings):
  fired (`alignmentFindings` 61).
- B715 (anchor and logged drops):
  the six `archive review dropped` warns are new.
- B716 (the composite carries its contributors’ assessments):
  not exercised (no composite built).
- B712 (the composite):
  attempted three times (`archive composite not built` at 21:10:56 UTC,
  21:15:30 UTC and 21:21:58 UTC),
  refused three times:
  23 against 24 lines (the email),
  an overlap on line 1 of 1 (the email’s label),
  34 against 36 lines (the letter).
  It built nothing.
- B719 (low reasoning effort on a NeuralWatt picture call):
  fired.
  The three picture reads by `glm-5.3-flash` on NeuralWatt (21:07:03 UTC,
  21:07:27 UTC and 21:08:08 UTC;
  `SPEND` lines) used 312,
  247 and 282 completion tokens,
  none near the 18,316 cap.
  B718 (the re-ask after a picture reply the length limit cut) was not exercised.
  The cap-outs in this run are text calls,
  which the lift does not reach (see K1 in "Traces to code").

### Traces to code

All at `2a336603bf`,
read with `git show` and `git grep`.
"Class" means the cause is in code or policy and would recur on other entries;
"one-off" means the cause is local to this entry or a single event.

- K1 (class).
  NeuralWatt text calls run at the provider’s default reasoning effort,
  and a reasoning-only model returns nothing when it hits its cap.
  Four `glm-5.3-flash` calls on NeuralWatt ran to 18,316 completion tokens with 0 content characters
  (21:17:32 UTC,
  21:20:36 UTC,
  21:37:15 UTC and 21:40:06 UTC;
  12.0174 Wh,
  29 percent of the run’s 40.9099 Wh).
  Each was followed by a re-ask on Hyper:
  three ran to the cap too (three `max_tokens` replies at 18,316 tokens,
  by a script over the finish reasons of the replies) and the fourth was cut at the grace.
  Two recovery rounds then waited the full writer grace and heard nothing (180 s on the letter,
  175 s on slice 8).
  The same call on the letter block ran to the cap in `zheermao20` (4.1361 Wh) and `zheermao21` (3.2377 Wh),
  so it is the third run in a row.
  The trace:
  `neuralwatt-client.ts:392` sets `pictureEffort` only when `carriesPicture` is true,
  and `:424` sends `reasoning_effort` only then;
  the header comment (`:66` to `73`) says "no reasoning parameter on a text call" with the owner’s lift of
  2026-10-21 ("only for NeuralWatt") applied to pictures.
  Ledger B719
  (`audit-ledger-neuralwatt-replies-cut-at-the-default-effort-and-the-energy-estimate.md:71` to `98`)
  records that the model’s documented
  `default_effort` is `max` and `supported_efforts` are `max`,
  `high` and `low`.
  The cap is `model-cards.ts:84` (18,316,
  a pooled 99th percentile).
  The same family of cap-outs hit `deepseek-v4.1-flash` on NeuralWatt once (13,082 tokens,
  `model-cards.ts:451`) and `minimax-m3` on Hyper four times (10,822 tokens,
  `model-cards.ts:259`;
  21:10:14 UTC,
  21:16:49 UTC,
  21:42:25 UTC and 22:11:15 UTC):
  14 of the 961 replies ended on a length or token limit (by the same script).
  Whether the NeuralWatt lane of `deepseek-v4.1-flash` accepts an effort parameter was not read.
- K2 (class).
  A stream that sends bytes but no content is indistinguishable from a slow answer until the straggler grace cuts it.
  Eleven `inception/mercury-2.5` streams were abandoned 120 s after quorum with `0 answer chars and 0 reasoning chars`,
  on 7,575 to 13,000 raw characters,
  `maxGap` 0.5 to 0.8 s and first byte under 1 s
  (contest slices 1,
  4,
  0 and 9;
  consolidation slices 3,
  6,
  5,
  6,
  14,
  12 and 14;
  21:59:56 to 22:23:44 UTC).
  Every one of the 11 raw-character counts is a multiple of 25,
  the length of `: OPENROUTER PROCESSING` plus two newlines,
  the comment line OpenRouter sends while it waits for the upstream provider
  (`openrouter-client.unit.test.ts:5` to `6` and `:99` record the line);
  the log holds counts and not bytes,
  so that the bytes are those comments is an inference,
  but 11 of 11 multiples of 25 is not a coincidence.
  Four more `inception/mercury-2.5` requests failed with `code 502, type provider_unavailable` after a success status
  (21:51:20 UTC,
  21:55:52 UTC,
  22:07:18 UTC and 22:13:54 UTC);
  `zheermao21` had one such line in all.
  `STREAM_FIRST_BYTE_MS` and `STREAM_IDLE_MS` are 600,000 ms (`stream-idle-guard.ts:60` and `:89`;
  the artifact’s `callConfig` reads `perCallTimeoutMs: 360000, streamFirstByteMs: 600000, streamIdleMs: 600000`),
  both longer than the per-call timeout,
  so neither can fire,
  and the idle guard times silence between body bytes,
  which a comment line breaks.
  The only bound is the straggler grace after quorum (120 s,
  `stage-round.ts:55` to `80`),
  so each of the 11 cost its round 120 s.
  Mercury’s consolidation stream time is 1701 s against 462 s in `zheermao21`
  (the phase script’s output for each log).
  Each cut call is ledgered at the cap (`SPEND ... completion=3831` at the abandon time).
- K3 (class by mechanism;
  an owner decision,
  not a bug).
  A repair-lane fix for a minor accepted claim is lost whenever the lane contest settles `neither`.
  The contest sheet is shown the dispute notes only (`lane-contest-driver.ts:143` and `:356` to `365`),
  and a claim disputes the archive only when it is an addition (at any severity) or an accuracy claim at major or
  critical severity (`archive-dispute.ts:60` to `75` and `:245` to `257`;
  the rule is the owner’s of 2026-09-27,
  "Major+ accuracy").
  Slice 8’s 因着 claim is `accuracy/mistranslation minor`,
  so the contest judges did not see it and called "because of" and "following" a "synonym pair"
  (`hf:moonshotai/Kimi-K3`’s ballot,
  22:02:27 UTC).
  The polish writers do not see it either:
  `consolidation-polish-round.ts:304` to `321` calls `runRefineStage` with no claims note,
  and the note reaches only the polish gate (`:445` to `451`;
  `consolidation-polish-gate-wire.ts:394` to `419`),
  where two judges noticed it and could only choose between two texts that both lacked the fix.
  One occurrence in this entry (slice 8);
  the other accepted claims were carried.
- K4 (class;
  a recorded design narrowness,
  to be vetoed or kept by the owner).
  The B712 composite builds nothing when a revision changes the block’s line count
  (`archive-composite-revision.ts:36` to `42` states the rule,
  and `:232` to `254` applies it).
  Both large blocks fell to it here (email 23 against 24,
  letter 34 against 36),
  and the third attempt fell to an overlap (`:280` onward).
  The letter shipped one reviewer’s partial revision for that reason:
  the only complete revision removed two lines.
- U1 (untraced).
  Hyper’s alphabetical key order (see "Whether the new rulings fired"):
  the ruling’s purpose (a reply cut short loses its choice,
  not its reason) is not met for 176 of 190 Hyper vote-bearing replies.
  The read looked in `anthropic-tool.ts`,
  `hyper-client.ts` and `anthropic-delta-scan.ts` and found no sort.
- U2 (untraced).
  The slice boundaries between slices 1 and 2 moved against `zheermao21`
  (58 and 67 source characters against 80 and 45).
  `chunk-document.ts` (`chunkByHeadings`,
  line 242) is not among the files changed between the two builds;
  The read did not find what changed the input.
- One-off.
  Two Hyper `gemma-4-26b-a4b-it` contest replies ran to 3,831 tokens (`max_tokens`,
  21:58:19 and 22:00:28 UTC,
  slices 0 and 8).
  Their text is 288 and 508 characters long and is cut inside an invented key,
  `unsupported_list`;
  The read did not trace what filled the rest of the budget.
- One-off.
  `select-self-vote (glm-5.3)`,
  10 findings against 7:
  the vote of a writer for its own candidate is weighted at 0.5 by design.
- One-off.
  Repair slice 11’s refinement was rolled back by the probe (`refine-rolled-back-by-probe`,
  21:50:06 UTC,
  1 added-damage claim),
  and the page text of slice 11 is the translate lane’s.

### Class verdict

Three classes need a code fix or a decision (K1,
K2,
K3);
K4 is a recorded design narrowness for the owner.
None touches who died,
when,
how she is named or what the writers tell her.
D23 leaves vague what she hoped to succeed at,
as the archive did.
The log shows none of the five markers that name a class on their own and no `[error]`.

Counts by class,
over the 30 departures D01 to D30
(the departures are in [the page of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-page.md) and [the letter of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-letter.md)):

- C (traced to code or policy):
  3 (D01,
  D05,
  D25).
- WD (a decision rule chose what shipped):
  4 (D07,
  D09,
  D19,
  D27).
- WB (the whole-block archive review chose what shipped):
  13 (D10 to D18,
  D22,
  D23,
  D24,
  D30).
- W (model wording,
  no code cause):
  10 (D02,
  D03,
  D04,
  D06,
  D08,
  D20,
  D21,
  D26,
  D28,
  D29).

Against the archive,
19 repeat its wording (D02,
D04,
D06,
D08,
D10,
D11,
D13,
D15,
D16,
D18,
D19,
D22,
D23,
D24,
D26,
D27,
D28,
D29,
D30),
2 repeat it in part (D14,
D17),
6 are new to this run (D03,
D07,
D09,
D12,
D20,
D21) and 3 are code or policy effects (D01,
D05,
D25).
Of the 19,
`zheermao21` had fixed,
wholly or in part:
D04,
D08,
D10,
D11,
D15,
D16,
D18,
D26 and D27.
The causes of the letter’s regression are K1 and K4 (the caps and the composite’s line-count rule) together with
the seating the sheet edit moved.

The most serious departures,
by what a reader is told:

- D10 to D18 together:
  Arts’s letter reads,
  in places,
  as the archive’s text with ten lines changed,
  with the empty "Those who are truly guilty won’t feel guilty" [59] the worst line.
- D22 to D24:
  the last words of the email lose "very" in "very sorry",
  the object of the hope and the addressee line,
  all small.
- D19:
  the reason for her renewed resolve reads as a date.

Open items:

- K1:
  the owner’s lift of the reasoning parameter for NeuralWatt covers text calls;
  the client sends it only for pictures.
  Whether `deepseek-v4.1-flash` on NeuralWatt accepts it was not read.
- K2:
  whether the comment-only stream should be cut before the grace is a design decision;
  a probe of the bytes (not run here) would settle the inference.
- K3 and K4:
  both follow rules the owner set or the code records as vetoable.
- U1 and U2 as traced under "Traces to code".
- The seating move (the sheet-text rotation) is an inference from a code comment.

### What the run read did not do

- It did not read the declared-names sheet or any prompt
  (the run directory holds replies,
  not prompts),
  so what the contest and polish sheets showed is known only from the code and the ballot texts that quote them.
- It did not run a spread measurement.
- It did not probe the Hyper key order or the mercury bytes.
- It did not trace why the slice boundaries moved.
- It did not check the host for the cause of any slowness.
