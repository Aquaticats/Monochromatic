# OpenRouter and the corpus passes: the rotation on build 8864316eb2 after the owner's answers (2026-10-22)

What was read on 2026-10-22 of the passes of the rotation that ran on the build
that landed the owner's answers of 2026-10-22 (UTC):
`zheermao23`,
`CuspariaKLSY23`
and `XingZ6018`,
and of the two passes of `XingZ60` that ran on the build after it,
`XingZ6019` and `XingZ6020`,
and of `XingZ6021`,
which ran on the build after those (2026-10-24 and 2026-10-25,
UTC).
The passes of the build before it (`2a336603bf`) are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).
The standing account of the readiness signal is [the readiness signal](translation-repair-readiness-signal.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## Where each pass is recorded

Each pass has its own file or files,
so that no file passes the repository's limit of 1,000 lines:

- `zheermao23`:
  [the run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-run.md)
  (log,
  NeuralWatt meter and plan energy,
  seats,
  the Mercury recheck,
  why the repair counts rose),
  [the stages](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-stages.md)
  (stage outcomes,
  slice verdicts and disputes,
  whether the new rulings fired,
  the recurrence of the known classes,
  side checks,
  open items)
  and [the evidence](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-evidence.md)
  (the log line behind every figure,
  the figures no log can settle).
  The shipped page was not read line by line.
- `CuspariaKLSY23` (entry `CuspariaKLSY`):
  [the run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-run.md)
  (log,
  NeuralWatt meter and plan energy,
  seats,
  the Mercury recheck),
  [the stages](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-stages.md)
  (stage outcomes,
  slice verdicts and disputes,
  the recurrence of the known classes,
  side checks,
  open items)
  and [the evidence](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-evidence.md)
  (the log line behind every figure,
  the figures no log can settle).
  It settled in 1,028,162 ms of pass;
  its log's last time is 13:39:30 UTC on 2026-10-22.
  The shipped page was not read line by line.
- `XingZ6018` (entry `XingZ60`):
  launched on 2026-10-22 at 13:44:40 UTC,
  the launcher printing `FROZEN BUILD MATCHES; run continues`.
  [The run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-run.md)
  (log,
  stages,
  what a later run can reuse,
  plan energy),
  [the providers](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-providers.md)
  (how Hyper and OpenRouter ran out,
  the stop,
  the third Mercury recheck,
  the seats)
  and [the defects](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-defects.md)
  (the quality issues,
  seven defects,
  the second reader's check,
  the points no log settles).
  It ended after 62.4 min (`TALLY status=ERROR`,
  then `STALLED`) with no artifact and no page:
  Hyper and OpenRouter ran dry in turn on top of Synthetic,
  NeuralWatt and Bedrock,
  and the writing bench's floor ended the entry.
  A second reader checked the read against the log and the source,
  and the check's confirmed reading is the one filed.
  The page was not written,
  so no page was read.
- `XingZ6019` and `XingZ6020` (entry `XingZ60`) are on the later build described in "The later build",
  and are recorded in
  [the XingZ6019 record](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6019.md).
  `XingZ6019` ended with no page after 19.2 min on a short writing bench;
  `XingZ6020` was running when the record was filed,
  and its read is filed since in the next item.
- `XingZ6020` (entry `XingZ60`):
  launched at 19:32:29 UTC on 2026-10-22 on the build `1d46232457`.
  [The run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md)
  (log,
  the providers and models,
  the NeuralWatt day,
  the Synthetic weekly credit and the rename that left it unreadable,
  the cost of the 300,000 ms hold,
  where the 10.5 hours went),
  [the stages](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-stages.md)
  (the timeline of the two attempts,
  the page that shipped without its last 30 slices,
  what the code does with a slice that ends unfilled and whether any check should have caught the gap),
  [the evidence](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-evidence.md)
  (control lines,
  refusals,
  lost voices,
  what the second reader found that the first missed,
  the points no log settles)
  and [the defects](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md)
  (S1,
  E2 and D1 to D7 with the second reader's verdicts,
  and the commit that fixed each).
  It SETTLED on its second attempt after 10 h 32 min 51 s (the first ended on the 7 h hard deadline during a hold
  wait),
  with the last 30 of 118 slices unfilled and no marker on the page.
  A second reader checked the read against the log,
  the corpus and the source,
  and the check's reading is the one filed.
  The fixes of its findings are what the build of `XingZ6021` carries.
- `XingZ6021` (entry `XingZ60`):
  launched at 22:49 UTC on 2026-10-24 on the build `bcff23b8ce`,
  described in "The build of XingZ6021".
  [The run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-run.md)
  (log,
  timeline,
  providers and models,
  the Synthetic credit,
  the NeuralWatt day,
  the DeepSeek balance),
  [the stages](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-stages.md)
  (the seven checks of the build's fixes,
  what the run could not test,
  the page that shipped against the source and the archive,
  and a note on footnote lines in quotes and list items),
  [the evidence](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-evidence.md)
  (refusals,
  errors,
  lost voices,
  short-bench rounds,
  the log lines behind the figures)
  and [the defects](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-defects.md)
  (the Shinonome section in full,
  N2 to N10 with the second reader's verdicts,
  the owner's rulings of 2026-10-25,
  the corrections and the points no log settles).
  It SETTLED on its first attempt after 2 h 5 min 45 s with all 118 slices filled and no hold,
  and the page that shipped carries archive text that the source does not contain
  (the Shinonome section),
  published without a `DEFECTS` line.
  A second reader checked the read against the log,
  the corpus and the source,
  and the check's reading is the one filed.

## The build

- Tip `8864316eb29b9848d95ceeaf4c039671c0c430e8`,
  pipeline `sha256-tree-v1:4e516f924c6f27eafe29e0d139d0cd04de5494428fc174da007db197d93f1f70`.
  The `START` line of each of the three passes carries both as given.
- It is the merge that landed after gate g165 on 2026-10-22 (UTC)
  (145 steps exit 0,
  the suite 2764 passing and none failing)
  and a clean rotation sweep (salts 1 to 9,
  no case failing a salt that salt 0 passes,
  3264 s),
  as [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-the-landings-the-zheermao22-restart-and-the-state-after-k4.md) records.
  The launcher holds that digest as the rotation's frozen build.
- The corpus input is the pin `a41fc607ea` (`corpusSha` `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`).
- What it carries that `2a336603bf` did not,
  by the handover and the commits on the branch since that build:
  the shared first-content window of 180,000 ms and the 90,000 ms window of `inception/mercury-2.5` (ledger B804,
  B809);
  the polish writers and rewrite judges shown the dispute note and the accepted claims (B805);
  the run-off line that names a tie or a plurality (B806);
  `reasoning_effort` `low` on NeuralWatt text calls to the GLM seat (B807);
  the recalibrated NeuralWatt buckets (B808);
  the K4 composite by line diff (B803);
  the audit that reads merge commits (B814);
  and the owner's answers of 2026-10-22:
  Qwen3.8-27B off both late benches,
  a short bench that closes one voice below every live seat,
  and code blocks in `doc/troubleshooting/**/*.md` exempt from the Rust rules.
- No pass waits for the NeuralWatt day.
  The owner,
  2026-10-22 at about 12:30 UTC:
  "You don't need to align launches to NeuralWatt starts."
  [The NeuralWatt decision](../decision/translation-repair-neuralwatt-provider.md) records it under "When a pass launches".

## The later build

- `XingZ6019` and `XingZ6020` ran on the frozen build `sha256-tree-v1:44ce3051...`,
  built from commit `1d46232457` (made at 18:52 UTC on 2026-10-22).
  It carries the direct DeepSeek and Xiaomi routes
  and not the fixes of the `XingZ6018` read (ledger B818 to B820),
  which are not among its ancestors.
- `XingZ6019` ran with NeuralWatt dry for its whole run,
  because its 550 Wh day had been spent by the earlier runs,
  and the account's day restarted at 19:15:08 UTC.
  The pass ended at 19:14:20 UTC,
  48 s before that,
  with a writing bench short of its floor
  (ledger B826).
  `XingZ6020` was relaunched at 19:32:29 UTC,
  after the restart,
  and what NeuralWatt held during it is not recorded here.
  Note of 2026-10-25 (UTC):
  it is recorded in
  [the run of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md):
  NeuralWatt read wet on all 518 meter readings,
  served 1,779 calls and drew 451.711 Wh of the 550 Wh day,
  and no hold ever named it.

## The build of XingZ6021

- `XingZ6021` ran on the frozen build `bcff23b8ce`
  (tip `bcff23b8ce4179b56b7ab4029ce35c2dd06cfd47`,
  pipeline `sha256-tree-v1:899b55a69b741024df2ddd2039e9362b2adb79e83447d41c680778cfdc967862`,
  corpus pin `a41fc607ea`),
  first log line at 22:49:32 UTC on 2026-10-24.
  The build of `XingZ6019` and `XingZ6020` was `1d46232457`.
- It carries the fixes for the defects that the read of `XingZ6020` found
  (filed in
  [the defects of XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md),
  with the commit that fixed each).
  The read of `XingZ6021` checked seven:
  the source text under a note when a slice has no archive and no backed wording (owner ruling of 2026-10-23),
  D1 (the wait that cannot help),
  D2 (tail slices tried twice to no gain),
  D3 (stages that ran in full on one voice,
  fixed in part by ledger B835),
  D4 (a rollback with no log line,
  the silent rollback of `XingZ6020`,
  ledger B829),
  E2 (the NeuralWatt hold and the NeuralWatt day,
  whose known return is ledger B826)
  and S1 (the Synthetic weekly credit read throughout).
  Of them S1 worked on the whole run,
  D2 and D4 did not recur,
  D3 is fixed for the lane contest and only in part for the consolidation,
  D1 was exercised only on its branch with no hold,
  and the source-text note and E2 were not exercised.
  The checks are in
  [the stages of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-stages.md).
- The run drew on NeuralWatt,
  Synthetic,
  Xiaomi,
  DeepSeek and the owner's Claude key;
  Hyper,
  OpenRouter and Bedrock were out from the first line.
  Synthetic's weekly credit ran dry 72.4 min in,
  and the benches that name Kimi and Qwen reached one or two voices from then on.
- The floors that left the Shinonome section as the archive's text are the subject of the owner's rulings of
  2026-10-25 and of a change in progress,
  with no commit yet;
  see N1 in
  [the defects of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-defects.md).
