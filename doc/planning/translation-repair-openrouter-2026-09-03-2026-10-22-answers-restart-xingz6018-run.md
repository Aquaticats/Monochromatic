# OpenRouter and the corpus passes: the run of XingZ6018 on build 8864316eb2 (2026-10-22)

What was read on 2026-10-22 of the run that came after `CuspariaKLSY23` on the same build:
the log,
the stages and what a later run can reuse,
the stop,
and the figures of the lead's brief that were checked.
Entry:
`XingZ6018` (entry `XingZ60`,
a long page of 118 slices).
It ended with no artifact and no page:
every provider went dry,
and the entry stopped at the writing bench's floor.

The provider budgets,
the stop,
the third Mercury recheck and the seats are in
[the providers of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-providers.md).
The quality issues the panel raised,
the seven defects,
the second reader's check and the points no log settles are in
[the defects of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-defects.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).

## How this record was made

The first reader read the log `temp/agent/corpus-pass-XingZ6018.log`
(16,238 lines by `wc --lines`,
5,651,226 bytes)
and the run directory,
and quoted source from the run's tip `8864316eb29b9848d95ceeaf4c039671c0c430e8`
(`git show <tip>:<path>`;
the files cited agree with the main checkout except where stated).
Line numbers in this record are of that log.
Counts come from scripts under `temp/agent/tr-fan/w287-xingz18/` or from `rg --count`,
and each figure names its method.
The scripts are `meters.ts`,
`calls.ts`,
`summary.ts`,
`mercury.ts`,
`reconcile.ts`,
`postdry.ts`,
`newstreams.ts`,
`refusals.ts`,
`refused.ts`,
`after.ts`,
`rounds.ts`,
`pace.ts`,
`quality.ts`,
`tense.ts`,
`voicelost.ts`,
`slow.ts`,
`payloads.ts`,
`cache.ts`,
`cachedslices.ts`,
`rundir.ts`,
`templates.ts` and `dry.ts`.
They only read local files.
The first scripts were run before the instruction to strip the environment;
none of them opens a network connection,
and every later one was run as `env --ignore-environment PATH=... HOME=... node <file>`.
The checker searched all 22 for `fetch(`,
`node:http(s)`,
`node:net`,
`child_process`,
`process.env` and `WebSocket`,
and found none.
No provider call,
build,
test,
`mise` task or `.env*` or auth file was touched,
and nothing outside the scratch directory was written.

A second reader checked the read against the log and the source as built
(`git show 8864316eb2:package/translation-repair/<path>`)
with the scripts `counts.ts`,
`odd.ts`,
`streams.ts`,
`postmark.ts`,
`misc.ts`,
`mer.ts` and `slice85.ts`
under `temp/agent/tr-fan/w287-xingz18/check/`.
This record is the read as corrected by that check:
where the two disagree,
the check's confirmed reading is the one filed,
and a point the check did not examine is marked as read by the first reader only.
The corrections and the unconfirmed points are listed in
[the defects of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-defects.md).
The scratch directory also holds files that are not among the scripts named in this section;
the checker did not open them and this read does not rest on them.

## Headlines

- Stage reached:
  the preparation finished,
  the repair lane's chunk pass finished for all 118 slices
  (88 computed,
  30 with no translation to repair),
  and the refine pass began and heard nobody.
  The translate lane never started:
  the writing bench was unreachable (line 15797).
  No artifact and no page were written.
- Hyper ran out at 14:28:15 UTC (balance 0,
  line 6720),
  and OpenRouter at 14:45:42 UTC (first wire HTTP 402,
  line 14065).
  Line 14105 is the first that quotes the provider's body;
  the first 402 on the wire is 1.8 s earlier,
  at lines 14065 to 14072.
- After the dry signal no chat stream was started
  (0 streams on either provider started after its mark),
  but the stage layer treated the refusal as an ordinary lost voice 994 times
  and kept going for 77.7 s.
  Meter reads continued after the marks.
- Mercury,
  third recheck:
  594 streams,
  all `completed`,
  none cut,
  none abandoned;
  the maximum first content was 15,283 ms (line 4544),
  so the 90,000 ms window was never approached.
- The plan energy moved 250.6 Wh with no NeuralWatt request in the run,
  and the by-model covered count did not move on any of the 63 `METERS` lines.
- Seven defect candidates were found and none was fixed when the read was made.
  D1 to D3 were fixed afterwards,
  as ledger B818,
  B819 and B820,
  with the follow-ups B821 to B825;
  D4 to D7 stay candidates,
  with no ledger entry until they are traced.
- The all-providers-dry refusal had not printed in the seven earlier logs searched,
  so this run was the first of those to print it.
  That is not a first for the pipeline:
  older logs and the audit ledger record earlier all-dry errors
  (`audit-ledger-provider-streams-usage-and-retries.md` counts 166 and 34 in two older logs).

## Run facts

- The build printed `Finished in 11.93s` (line 193).
- The `ONLY XingZ60 ...` line (line 195) says the entry was chosen by hand and the run should go into a throwaway runs
  directory.
- The `START` line (line 196) carries tip `8864316eb29b9848d95ceeaf4c039671c0c430e8`,
  pipeline `sha256-tree-v1:4e516f924c6f27eafe29e0d139d0cd04de5494428fc174da007db197d93f1f70`,
  and `files=3 pending=1 done=0`.
  The tip and the pipeline are those of `zheermao23` and `CuspariaKLSY23`.
- `PROVIDER OFF bedrock` (line 198),
  and a writer grace of 180,000 ms built in (line 197).
- The first timestamped line is 13:45:00.310 UTC (line 199).
  The launcher printed the launch as 13:44:40 UTC.
  The log ends `Finished in 3745.98s` (line 16238),
  so the invocation lasted 62.4 min.
- The first attempt ended `TALLY ... status=ERROR ms=3721659` (line 15798),
  which is 62.0 min.
  `DONE processed=0 of pending=1; artifacts=0/1 ... elapsed=3726996ms` (line 16222).
- `attempts.json` reads `{"XingZ60": 3}`:
  three attempts in this invocation,
  the first long and two of 3,464 ms and 1,867 ms (TALLY lines 16184 and 16220).
  `artifacts/` and `fixed/` were empty (`rundir.ts`).
- The entry has 15 aligned units,
  118 slices and 21 alignment findings (line 1052).
  The archive page ends early:
  the log says `30 slice(s) after the last agreed pair, 5131 source code points ... admitted on that bound` (line 1050),
  and those 30 slices (88 to 117) are `no translation to repair; the translate lane owns this passage`
  (first at line 14296).

## Timeline and stages

Every time is UTC on 2026-10-22.

- 13:45:00 to 13:56:21 UTC (11.35 min),
  the preparation (lines 199 to 1051):
  the front matter is the archive's (line 216,
  `authority=archive`);
  page titles were heard 5 of 6 (line 231);
  one cited reference was already cached and in error (`REFERENCES cited=1 cached=1 bought=0`,
  line 233),
  with attestation `heard=5 answered=0 verified=0` (line 246);
  section pairing paired 13 of 15 sections (line 259);
  block pairing ran in many rounds (the last paired 13 of 76 original blocks,
  the picture-bearing page section);
  one picture was read and corroborated by 3 of 3 readers at overlap 0.606
  (line 460;
  `google.gemma-4-31b` had no provider);
  coverage checks of slices 88 to 117 ran at lines 613 to 1045.
- 13:56:21.424 UTC (line 1051),
  `both lanes over 118 slices, repair first`.
- 13:56:21 to 14:46:58 UTC (50.6 min),
  the repair lane's chunk pass:
  118 `SLICE-START lane=repair` and 118 `SLICE-COST lane=repair` lines (`pace.ts`),
  the first close at line 1482 (13:59:16 UTC)
  and the last at line 14557 (14:46:58.587 UTC).
  Up to 34 slices were in flight at once (`pace.ts`).
  Closes per five minutes from the lane start were 2,
  7,
  4,
  9,
  6,
  5,
  11,
  14,
  12,
  then 47 in the 45 to 50 minute bucket,
  of which 30 are the no-translation tail closing together at 14:45:44.377 UTC (lines 14296 to 14384),
  right after the dry mark.
  The slowest slices were slice 4 at 574,641 ms (line 3368),
  slice 3 at 423,506 ms (line 2217)
  and slice 23 at 415,860 ms (line 5865).
  The median computed slice took 103,951 ms.
- Outcomes (`quality.ts`,
  `rg --count`):
  69 chunks `repaired`;
  7 `nothing to edit, unchanged`;
  6 `no validated claims, unchanged`;
  4 `unchanged, the archive won`;
  `repair shipped: 69/118 slices changed, 298 issues` (line 15761);
  `semantic wrap: rewrapped 59 of 118 repair outcomes` (line 15760).
  Of 164 served accepted issues,
  159 were resolved,
  and four chunks left one or more open:
  slice 13 at 3 of 5 (line 3903),
  slice 43 at 6 of 7 (line 8805),
  slice 57 at 1 of 2 (line 10682)
  and slice 80 at 3 of 4 (line 13978).
  The cluster verdict lines count accepted 193 (2 critical,
  88 major,
  103 minor),
  needs-human 60 and rejected 61.
- Five translated slices did not settle with a full bench because the dry mark fell on them:
  slices 82,
  83,
  84,
  86 and 87 were refused a cache entry (lines 14556,
  14181,
  14493,
  14236 and 14293).
  Slice 85 is a separate case,
  recorded as defect D3 in
  [the defects of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-defects.md).
- 14:46:58.6 to 14:46:59.0 UTC,
  the refine pass:
  240 `refiner round: 0/3 heard` lines (`rg --only-matching`),
  which is four rounds (one plus three retries) over 60 slices,
  and 60 `stage-quorum-unmet (refiner 0/3)` findings.
  The pass heard nobody and cached nothing for those slices.
- 14:47:00 to 14:47:01 UTC,
  the `content lost at slice N` lines,
  and two `introduced repetition` warnings (lines 15764 and 15765).
- 14:47:01.971 UTC (line 15797),
  `writing bench unreachable: translators 0 of 8 reachable, floor 2; stopping the entry`.
  There was no `SLICE-START lane=translate`,
  no lane contest,
  no consolidation and no page assembly.
- 14:47:02 to 14:47:07 UTC,
  two re-entries that stopped in the preparation or at the lane check
  (lines 15799 to 16184 and 16185 to 16220),
  then `STALLED` (line 16221) and `DONE` (line 16222).

### What a later run can reuse

- `REATTEMPT XingZ60 queued: 157 more cache records than it had` (line 15799),
  then 16 more (line 16185),
  and `STALLED XingZ60: its 173 cache records are what it started with` (line 16221).
- `slice-cache/XingZ60` held 173 parseable records (`cache.ts`):
  83 repair-slice settlements (slices 0 to 81 and 85;
  `cachedslices.ts`),
  58 refine records,
  26 pairing records,
  2 page-title records,
  2 section-pair records and
  2 picture records,
  plus seven `*-generation.txt` stamps,
  each equal to the `START` pipeline digest.
- Not cached were repair slices 82,
  83,
  84,
  86 and 87 (none has a record)
  and the refinement of the 60 slices in the refine pass.
  The 30 tail slices need the translate lane,
  which produced nothing.
- Also on disk were 2,476 stored payloads (`payloads.ts`) and 147 ledger files (`rundir.ts`).
- The directory is the throwaway one the `ONLY` line asks for,
  so a later run reuses it only when pointed at this directory with the same pipeline digest.
  That the stamp is what invalidates a record on a digest change was read from the file contents and not from code.
- A later run on this directory would also resume slice 85 as examined,
  which is the defect D3 records;
  that is read from the code path (`storedOutcome` returns `stored`) and was not run.

## Plan energy

- The end line `NeuralWatt PLAN ENERGY SPENT: 250.6 Wh` (line 16224)
  reads against a baseline of `6.3214 kWh charged this period` (line 204,
  13:45:01.115 UTC).
  The end baseline would stand near 6.5720 kWh (computed;
  no line prints it).
- The run lasted 62.1 min (`DONE` elapsed 3726996 ms),
  so the figure is 4.0 Wh per min.
- The series of the plan figure in the rotation,
  from the earlier reads,
  is
  37.3 (`CuspariaKLSY22`),
  54.3 (`XingZ6017`),
  326.9 (`zheermao23`),
  255.7 (`CuspariaKLSY23`)
  and 250.6 Wh (this run).
  The `CuspariaKLSY23` end baseline was computed there as about 6.2181 kWh,
  so about 103 Wh was charged between that run's last line (13:39:30 UTC) and this run's baseline (13:45:01 UTC),
  5.5 min (computed;
  19 Wh per min).
- The by-model covered count did not move.
  `neuralwattAccountWh=555.188` stood on all 63 `METERS` lines (lines 210 to 14514),
  with `neuralwattWh=-5.188` and `neuralwattReckonedWh=0.000` on all 63,
  so nothing was in flight on NeuralWatt at any reading.
  The same pair stood at the last reading of `XingZ6017` and at every reading since.
  The 2,627 `SPEND` lines are 775 Hyper and 1,852 OpenRouter;
  none is `neuralwatt` or `synthetic`.
  `markRefused` never named NeuralWatt,
  since the meter already read dry,
  and the NeuralWatt day (started 2026-10-21T19:15:08Z,
  line 205) ended at 19:15:08 UTC on 2026-10-22,
  4.5 h after the run.
- The owner said the key of the dsh harness is on the same account ("Yes on the same account."),
  so the 250.6 Wh is read as that harness's use.
  The log cannot show it,
  and it is not a defect of the pass.

## Figures of the brief that were checked

- Confirmed:
  the `START` line 196 and its values;
  files 3,
  pending 1;
  the `METERS` line at 210 (13:45:02 UTC) with balances 153 and 3.92;
  the NeuralWatt 555.188 against the 550 Wh day and the day start 2026-10-21T19:15:08Z;
  775 Hyper `SPEND` lines,
  the last at 14:29:14 UTC (line 7014);
  1,852 OpenRouter lines,
  the last at 14:46:57 UTC (line 14498);
  `Every provider is out of budget at once` first at line 14107;
  the bench-unreachable line 15797 (14:47:01 UTC,
  translators 0 of 8,
  floor 2);
  the TALLY lines 15798,
  16184 and 16220 with `status=ERROR` and the two re-entries of 3,464 and 1,867 ms;
  `DONE` at line 16222;
  the plan energy 250.6 Wh at line 16224;
  and `Finished in 3745.98s` at line 16238.
- Corrected:
  the first OpenRouter HTTP 402 on the wire is at line 14065 (14:45:42.292 UTC);
  line 14105 (14:45:44.317 UTC) is the first that quotes the provider's body.
