# OpenRouter and the corpus passes: the evidence of zheermao23 on build 8864316eb2 (2026-10-22)

The log line or artifact field behind every figure of
[the run of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-run.md)
and
[the stages of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-stages.md),
and the figures no log can settle.
A second reader recounted every figure from the logs on 2026-10-22 with scripts of its own
(under `temp/agent/tr-fan/w281-read/check/`):
93 confirmed,
3 corrected in place in the run and stages files,
4 unsourceable,
which are the cause of the 326.9 Wh of plan energy
(settled by the owner afterwards:
the dsh harness shares the NeuralWatt account),
the 90,000 ms Mercury window (a card value and no log line),
the launch time (from the launch unit and not the log)
and the ledger and class labels (from other records).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## zheermao23 evidence, 2026-10-22

### Evidence for the figures

Log line numbers are of `corpus-pass-zheermao23.log`.
Artifact fields are of `artifacts/zheermao101.json`.
Template line numbers are of the `zheermao22` run record.

- Tip,
  pipeline digest,
  files,
  pending:
  line 196;
  artifact `tip`,
  `pipelineDigest`,
  `corpusSha`.
- Writer grace built in:
  line 197.
- `PROVIDER OFF bedrock` (and the only `HTTP 401` text):
  line 198.
- First timestamped line 12:31:31.474 UTC:
  line 199.
  Last timestamped line 13:20:58.428 UTC:
  line 4210.
  Log length 4224 lines:
  `wc --lines`.
  `Finished in 2989.20s`:
  line 4224.
- `OVERLAP zheermao101 value=4 source=fallback`:
  line 200.
- Baseline 5.6318 kWh:
  line 202.
  Day anchor:
  line 203.
- First `METERS` (neuralwatt dry,
  minus 5.188,
  555.188,
  synthetic 3.97 percent,
  hyper 250,
  openrouter 4.10):
  line 209.
  Last `METERS` (synthetic 0 percent,
  hyper 194,
  openrouter 3.99):
  line 4163.
  Synthetic first reads dry:
  line 2842 (hyper 231,
  openrouter 4.02).
  The 38 `METERS` lines:
  209,
  267,
  314,
  362,
  381,
  397,
  434,
  461,
  502,
  526,
  552,
  577,
  631,
  726,
  792,
  927,
  1094,
  1251,
  1502,
  1563,
  1730,
  1976,
  2102,
  2208,
  2304,
  2372,
  2518,
  2649,
  2772,
  2842,
  3018,
  3152,
  3349,
  3454,
  3653,
  3856,
  3985 and 4163.
- `JUDGE SEATS`:
  lines 210,
  268,
  632,
  633,
  2680,
  3153 and 3350.
- `FRONT MATTER`:
  line 211.
  Reference step:
  lines 214 and 215 (`REFERENCE 1`,
  `REFERENCES cited=1 cached=1 bought=0`),
  230 and 231 (`ATTESTED`).
- Block pairing:
  lines 245,
  246,
  260,
  338 to 340,
  355,
  606 to 608,
  624.
  `PROMPT-REUSE`:
  lines 332 to 337 and 600 to 605.
- Pictures:
  start 268 and 269;
  OCR reads 271,
  289,
  307;
  model reads 274 to 286,
  292 to 304,
  317 to 329;
  `google.gemma-4-31b failed outright`:
  287,
  305,
  330;
  corroboration:
  288,
  306,
  331.
- Archive block reviews:
  email 385 to 390,
  424 to 427;
  email label 448 to 454,
  475 to 483;
  letter 508 to 510,
  536 to 545;
  letter label 565 to 570,
  591 to 599.
  Composite not built:
  388,
  509,
  569.
- Both lanes start:
  634 and 635.
  Repair `SLICE-COST`:
  828 (slice 1),
  853 (2),
  980 (3),
  1384 (6),
  1446 (7),
  1479 (4),
  1492 (5),
  1528 (10),
  1861 (11),
  2005 (12),
  2076 (13),
  2109 (8),
  2288 (14),
  2357 (0),
  2365 (9).
  Translate `SLICE-START`:
  2685 onward;
  translate `SLICE-COST`:
  2728 to 3140.
  Consolidation `SLICE-COST`:
  3516 (slice 3),
  3608 (5),
  3706 (4),
  3837 (6),
  3885 (0),
  3913 (9),
  4016 (8),
  4078 (11),
  4186 (12),
  4204 (14).
- Lane contest start:
  line 3154.
  Consolidation start:
  line 3351.
  Publish:
  line 4206.
- `TALLY`:
  line 4207.
  `DESTINATIONS`:
  4208.
  `DONE`:
  4209.
  End `stream quota`:
  4210.
  Plan energy:
  4211.
  `SEAT` lines:
  4212 (`gemma-4-e2b`),
  4213 (`deepseek-v4.1-flash`),
  4214 (`gemma-4-26b-a4b-it`),
  4215 (`hf:zai-org/GLM-5.3-Flash`),
  4216 (`glm-5.3`),
  4217 (`minimax-m3`),
  4218 (`inception/mercury-2.5`),
  4219 (`hf:Qwen/Qwen3.8-27B`),
  4220 (`hf:moonshotai/Kimi-K3`),
  4221 (`gemma-4-31b`),
  4222 (`xiaomi/mimo-v2.6-flash`),
  4223 (`typesafe/jev-1.13`).
- Mercury:
  maximum first content line 2112 (12,820 ms);
  empty streams 3731 and 3790;
  their `SPEND` lines 3732 and 3791;
  the 502 warns 3733 and 3792;
  `schema-mismatch` warns 3176,
  3313,
  3483;
  `no re-ask elsewhere`:
  3175,
  3312,
  3482;
  trailing text 4065;
  other long first-content streams:
  2134 (`glm-5.3-flash`,
  122,849 ms),
  2623,
  3881,
  2617,
  2613.
  Stream counts,
  percentiles and medians:
  `stats.ts` over the stream lines.
- Qwen cuts:
  stream lines 386,
  537,
  897,
  1148,
  1394,
  1693;
  warn lines 390,
  545,
  920,
  1158,
  1425,
  1711;
  rounds 385,
  536,
  896,
  1147,
  1393,
  1692.
  Slice 9 editor round:
  2136 (stream 2134).
- Cap-outs:
  372,
  493,
  517,
  527;
  re-asks 374 and 384,
  495 and 505,
  519 and 532,
  529 and 535;
  `max_tokens` payloads:
  `payloads.ts`.
- Contest verdicts:
  3197,
  3219,
  3230,
  3257,
  3271,
  3283,
  3308,
  3322,
  3334,
  3342;
  4/6 rounds without grace:
  3329 and 3337.
  Polish gates:
  3815,
  3851,
  4147,
  4158;
  polish ballots citing the claim:
  3846,
  3847,
  3848,
  3850.
  Consolidate gates:
  3469 (slice 5),
  3633 (4),
  4000 and 4001 (11),
  4040 (12),
  4050 (14).
  Strict-majority announcement:
  3900.
  `translate-archive-disputed`:
  2682 to 2684,
  3353,
  3708,
  3839.
  `introduced repetition`:
  2676,
  3144.
  `re-seated`:
  2681.
- Shipped page:
  `wc --chars --bytes --lines` of `fixed/people/zheermao101/page.en.md`
  (5972 characters,
  6102 bytes,
  151 lines);
  line 91 carries the slice 8 fix;
  log line 4206 prints 5972.
- Artifact fields:
  `callConfig`,
  `durationMs` (2966120),
  `lanes.repair.result.issues` (42),
  `lanes.repair.result.findings` (270),
  `lanes.repair.result.changedSliceIndices`,
  `lanes.translate.result.changedSliceIndices`,
  `laneSelection.slices`,
  `consolidation.slices`,
  `preparation.alignmentFindings` (63),
  `preparation.sliceCount`.
- Payloads:
  892 files in `prompt-payloads/`,
  fields `reply.finishReason` and `reply.servedBy`
  (`payloads.ts`,
  `payloads2.ts`).
- Ledger directory:
  40 files,
  each a selection record with the fields `task`,
  `at`,
  `candidates`,
  `ballots`,
  `selectedIndex` and `context` (a script over the directory).
- `zheermao22` figures:
  the template's `TALLY` (line 81),
  the `DONE` line (84),
  the phase times (111 to 154),
  the counts (224 to 287),
  the NeuralWatt section (309 to 398),
  the `SEAT` lines (402 to 455),
  the Mercury class (760 to 798).
- Sibling figures:
  `XingZ6017` providers file lines 292 and 293 (minus 5.188 and 555.188 Wh),
  353 to 355 (54.3 Wh,
  about 3.8036 kWh),
  and its Mercury counts at lines 194 to 216;
  `CuspariaKLSY22` providers file lines 98 and 99 (37.3 Wh)
  and 286 to 317 (the mercury stalls).

### Figures that could not be sourced

- The 90000 ms Mercury first-content window is in no line of the log or the artifact;
  it is read from the worktree card only.
- The end baseline of NeuralWatt (about 5.9587 kWh) is computed and printed nowhere.
- What drew the 326.9 Wh of plan energy is in no log line;
  the owner settled it on 2026-10-22
  (the NeuralWatt key of the owner's dsh harness is on the same account).
- The host time of the launch (12:30:44 UTC) is from the brief and not from the log.
