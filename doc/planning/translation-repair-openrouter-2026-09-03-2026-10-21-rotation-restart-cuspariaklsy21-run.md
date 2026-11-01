# OpenRouter and the corpus passes: the run of CuspariaKLSY21 on build 3637592bd4 (2026-10-21)

What was read on 2026-10-21 of the run of the second pass of the restarted rotation:
the log,
the NeuralWatt meter,
the seats and quorums,
and the absence of archive blocks.
Entry:
CuspariaKLSY21 (entry `CuspariaKLSY`).
The shipped page,
line by line,
is in
[the page of CuspariaKLSY21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-cuspariaklsy21-page.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the three passes of the restart share are in
[the rotation restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md).

## CuspariaKLSY21 read, 2026-10-21, 06:31 UTC: SETTLED in 11.8 min of pass; the baseline and the unreachable count worked as built; the page is worse than CuspariaKLSY20's, most in slices 3 and 1

The read found the NeuralWatt baseline and the unreachable count working as built and no stage throwing.
The entry has no archive blocks,
so the archive block review was not tested.
The page is worse than the page of `CuspariaKLSY20`,
most in slices 3 and 1.

### Run facts

`CuspariaKLSY21` (entry `CuspariaKLSY`) ran on tip `3637592bd43652fe2c6ff3cf2ec31e5a27c39c7d`
and pipeline `sha256-tree-v1:06c97a69446dc452fa2c602da834c36febb7f0d7b4b1010b5dfa9fab81097eec`.
Its `START` line carries both as given,
and the artifact's `tip` and `pipelineDigest` are the same strings.
The launch brief gave the start as 06:03:23 UTC.
The log's first timestamped line is 06:04:38.972 UTC,
after a build that printed `Finished in 47.01s`.
The pass wrote its page at 06:16:25.926 UTC,
and the log's final line is `Finished in 778.14s`.
The corpus input is the pin `a41fc607ea`
(`corpusSha` of the artifact is `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`).
The pinned directory holds no `.git`,
so the bytes were checked instead:
the source `page.md` is 555 characters and 1329 bytes,
equal to `preparation.sourceChars` and `preparation.sourceBytes`,
and the archive `page.en.md` equals `preparation.archiveText` byte for byte (1720 characters).

The `TALLY` line reads:

```text
TALLY CuspariaKLSY status=SETTLED slices=4 repairStatus=repaired repairIssues=20 repairAccepted=16 repairResolved=14 repairFindings=169 repairChanged=4 translateStatus=complete translateChanged=4 documentsDiffer=3 pageChanged=4 pageSilent=0 alignmentFindings=1 selection=contested ms=706897
```

The `DONE` line reads `DONE processed=1 of pending=1; artifacts=1/1 declined=0 ... elapsed=706959ms`,
which is 11.8 min
(`CuspariaKLSY20`:
11.5 min at 690419 ms;
`CuspariaKLSY19`:
16.0 min).
The publish line reads `publish: wrote 4 slices into a page of 1932 characters`
(a count of the shipped file gives 1932 characters and 1954 bytes and a trailing newline;
`CuspariaKLSY20` wrote 1929).

Against the `TALLY` line of `CuspariaKLSY20`:

- `repairIssues` 20 against 21;
  `repairAccepted` 16 against 21;
  `repairResolved` 14 against 20.
- `documentsDiffer` 3 against 4:
  slice 3 is the one slice where the repair lane and the translate lane wrote the same text.
- `repairFindings` 169 against 66.
- Everything else is equal
  (`translateChanged=4`,
  `pageChanged=4`,
  `pageSilent=0`,
  `alignmentFindings=1`,
  `selection=contested`).

Log counts,
each by a script of whole-line regular expressions over the log (1875 lines);
the `CuspariaKLSY20` figure in brackets is the same script over its log (1648 lines):

- 163 lines start with `[warn]` [134].
  Of them 143 carry `seat unreachable` [57],
  2 carry `HTTP 401` [2],
  0 carry `HTTP 429` [2],
  7 carry `schema-mismatch` [7],
  1 carries `voice lost` [4],
  0 carry `cut-mid-reply` [1],
  and 0 carry `quorum-not-met` [1].
  The other 20 warns are named at the end of this section.
- `[error]`,
  `DEFECTS`,
  `REATTEMPT`,
  `STALLED`,
  `mdx-downgraded`,
  `StreamOverrunError` and `closed on a short bench`:
  none of the seven.
- Four decision lines read `keeping the fallback` [3]:
  translate slice 3 tied at weight 2;
  translate slice 1 drew 1.5 against a minimum of 2.00;
  consolidation slice 0 tied at weight 1 and again at weight 2.
- No `ARCHIVE BLOCK` or `archive block` line exists in the log.

Where the time went,
from the `SLICE-START` and `SLICE-COST` lines:

- The repair lane ran 06:04:53 to 06:08:05 UTC (slice 3 last,
  191880 ms).
- The translate lane started at 06:08:41 UTC and its last slice cost closed at 06:10:54 UTC
  (slice 2,
  133020 ms).
- The lane contest took 06:10:54 to 06:11:22 UTC and judged slices 0,
  1 and 2 (slice 3 agreed).
- Consolidation ran 06:11:25 to 06:16:25 UTC:
  5.0 min against 3.9 min in `CuspariaKLSY20`
  (slice 0 at 299882 ms,
  slice 1 at 296464 ms,
  slice 2 at 144363 ms).
  Two causes show in the log:
  the `select` round of slice 1 spent 134864 ms and ended without a quorum,
  and the `select` rounds of slice 0 each waited for their fourth voice
  (69606 ms and 112266 ms with 0 ms of grace).
- The writer grace of 180000 ms never fired:
  every `SEAT` line says `abandoned=0` (`CuspariaKLSY20`:
  one abandoned ask).

The 20 `[warn]` lines that are not `seat unreachable`
(all of them printed;
the groups sum to 20):

- 2 for the one Bedrock credential refusal (06:04:47.451 UTC).
- 7 `panel ballot from gemma-4-26b-a4b-it irregular: group-index-out-of-range`
  (one at 06:05:12 UTC and six at 06:05:16 UTC).
- 2 `translate-archive-disputed` (slices 1 and 3,
  06:08:41.510 UTC).
- 5 for consolidation candidates:
  4 that failed the publication rule (slice 1 once and slice 2 three times)
  and 1 `minimax-m3 answered unable to its findings` (slice 2,
  06:13:00.946 UTC).
- 2 for `deepseek-v4.1-flash` cut by `finish_reason=length` on the consolidation `select` of slice 1
  and the recovery round that followed.
- 2 for the page assembly's `name-gloss-restored` (06:16:25.909 UTC).

### NeuralWatt baseline, stop lines and energy

The baseline was taken once,
on the first attempt,
at 06:04:40.521 UTC:
`NeuralWatt quota at the start of the run: 3.1687 kWh charged this period`
(the plain,
not-late wording of `takeBaseline` in `neuralwatt-quota-session.ts`).
The quota stream line before it
(`stream quota: completed, elapsed 1399ms`,
06:04:40.499 UTC) shows the read succeeded.

- No baseline failure:
  the log holds no `NeuralWatt quota could not be read` line,
  so the 60 s retry spacing never ran.
- No late-start line exists
  (`NeuralWatt quota read at ..., after the first read of the run failed`),
  and the closing line carries no `the later reading taken at` text,
  so `laterStartAt` was never set.
  These two paths are not exercised by this run,
  and this run says nothing about whether they work.
- Stop lines:
  none.
  `rg` for `NeuralWatt` outside the per-call lines returns exactly three lines
  (the baseline,
  the day anchor at 06:04:40.521 UTC and the closing line).
  A case-insensitive search for `allowance is blocked`,
  `AllowanceError`,
  `out of energy`,
  `covered models` and `free daily`
  matches only the closing line and two judge reasons,
  none of them a refusal.
  Every `METERS` line says `neuralwatt=wet` and `neuralwattKeyAllowance=none`.
  So no NeuralWatt stop fired.
- The day:
  `decideNeuralWattDayAnchor` read `current_period_start` and put the day's start at 19:15:08.000 UTC
  (the day that began 2026-10-20T19:15:08.000Z).
- The closing line (06:16:26 UTC,
  after one more quota read) reads
  `NeuralWatt plan energy: no plan energy was spent between the start and the end of the run; the free daily energy is not in this figure`,
  the same text as `CuspariaKLSY20`'s.

The meter,
from the 11 `METERS` lines (first 06:04:42.594 UTC,
last 06:15:56.373 UTC):

- `neuralwattWh` (what is left of the day's free energy) fell from 424.057 to 403.232,
  which is 20.825 Wh.
  `neuralwattAccountWh` (used on the whole account since the day began) rose from 125.943 to 146.768,
  the same 20.825 Wh.
- The two sum to 550.000 on the first and last lines and not on the middle ones (549.848 on the second).
  `neuralWattMeterLevel` says the remainder counts from the larger of the ledger's spend and the account's.
  The read takes that as the cause,
  and calls it a guess.
- `neuralwattReckonedWh` stayed 0.000.

Per call,
by the energy tally script over the log:

- `all settled calls: n=85 total=21.0145 Wh`.
- `deepseek-v4.1-flash` 4k to 16k:
  n=69,
  total 12.7144 Wh,
  estimate 0.0758 each,
  36 calls over it;
  under 4k:
  n=1,
  total 0.0097 Wh.
- `glm-5.3-flash` 4k to 16k:
  n=14,
  total 8.2826 Wh,
  estimate 0.3725,
  9 calls over it;
  under 4k:
  n=1,
  total 0.0078 Wh.
- By model (a second script over the same `settleNeuralWattCall` lines):
  `deepseek-v4.1-flash` 70 calls,
  12.7241 Wh measured against 5.2535 Wh estimated;
  `glm-5.3-flash` 15 calls,
  8.2904 Wh against 5.2539 Wh;
  together 85 calls,
  21.0145 Wh against 10.5074 Wh.
- The 85 equals the 85 `provider=neuralwatt` `SPEND` lines,
  and the 70 and 15 equal the `asked=` of the two `SEAT` lines.

The meter and the calls agree:
the last `METERS` line (06:15:56.373 UTC) precedes 6 settled calls worth 0.1892 Wh,
and 20.825 plus 0.1892 is 21.0142 Wh against 21.0145 Wh from the calls.

Against `CuspariaKLSY20`:
80 calls and 21.192 Wh then,
85 calls and 21.0145 Wh now.
The measured energy ran about twice the estimate in both runs
(21.192 against 10.722,
then 21.0145 against 10.5074).
One `deepseek-v4.1-flash` call on consolidation slice 0 measured 0.7152 Wh against 0.0758 Wh estimated
(06:14:19.979 UTC),
and the tally's maximum for that bucket is 1.4752 Wh.

The other meters:

- Synthetic `0%` and Hyper `0` throughout (dry).
- Bedrock `5.02` USD throughout with `bedrockReckonedUsd=1.32`.
- OpenRouter `13.81` to `13.66` USD;
  the 232 OpenRouter `SPEND` lines sum to 0.1728 USD (`CuspariaKLSY20`:
  0.1415 USD).
- The NeuralWatt `SPEND` lines carry a nominal 0.1233 USD that the read does not count,
  as `neuralwatt-meter.ts` says NeuralWatt reports a nominal cost on free-energy requests.

### Seats, unreachable seats and quorums

The `SEAT` lines after `DONE` read (usable out of asked,
each checked against the `SEAT` text):

- `minimax-m3` 65 of 65.
- `deepseek-v4.1-flash` 69 of 70 (1 unusable).
- `inception/mercury-2.5` 69 of 69.
- `gemma-4-26b-a4b-it` 61 of 61.
- `typesafe/jev-1.13` 22 of 22.
- `hf:zai-org/GLM-5.3-Flash` 15 of 15 (0 abandoned).
- `xiaomi/mimo-v2.6-flash` 10 of 10.
- `glm-5.3` asked=0,
  `unreachable=24`.
- `hf:Qwen/Qwen3.8-27B` asked=0,
  `unreachable=65`.
- `google.gemma-4-e2b` asked=0,
  `unreachable=54`.

Totals:
312 asks,
311 usable,
1 unusable,
0 threw,
0 abandoned,
143 unreachable.
`CuspariaKLSY20` printed no `unreachable=` field;
there the same three seats were counted as asked and threw 4,
5 and 5 times (14 in all).

Checks on the count:

- The 143 equals the 143 `[warn]` lines that carry `seat unreachable`
  (a script counted `no provider can take <model>:` per model:
  24,
  65 and 54,
  equal to the three `SEAT` lines).
  In `CuspariaKLSY20` the same warn count was 57 against 14 `threw`,
  so that run's counter did not match its own warns.
  Whether that was the miscounting this build set out to remove was not tested (a guess).
- `NoProviderForModelError` appears in no log line,
  so a reader of the log sees the refusal only as the warn text.
- The 312 asks plus 5 re-asks on OpenRouter after `schema-mismatch` on NeuralWatt equal the 317 `SPEND` lines (232 plus 85).
  That match is arithmetic,
  and the read takes the 5 re-asks as the cause (a guess).
- `google.gemma-4-e2b` is counted `asked=0` although its first call returned `HTTP 401` at 06:04:47.451 UTC.
  The log gives that call two lines:
  `[chatText] <- google.gemma-4-e2b: HTTP 401`
  and `SEAT CREDENTIAL REFUSED google.gemma-4-e2b on bedrock`
  ("The security token included in the request has expired.").
  The block pairing round then logs the seat as `seat unreachable` at 06:04:47.452 UTC.
  So the refusal after the credential was held is one of the 54,
  and no ask of this seat is counted.
  This differs from `CuspariaKLSY20`,
  where the seat's 5 calls were counted as asked and thrown
  (that one among them,
  a guess).

The `JUDGE SEATS` lines (6,
at 06:04:42 UTC,
06:04:53 UTC twice,
06:08:41 UTC,
06:10:54 UTC and 06:11:25 UTC):

- The first (phase `preparation`) says `unserved=hf:Qwen/Qwen3.8-27B,glm-5.3` with `bedrock=wet`.
- The other five say `unserved=hf:Qwen/Qwen3.8-27B,google.gemma-4-e2b,glm-5.3,google.gemma-4-31b` with `bedrock=dry`,
  after the 401 held Bedrock out for the rest of the run.
  `google.gemma-4-31b` is named unserved but has no `SEAT` line,
  so it was never asked.
- All six carry `withheld=hf:moonshotai/Kimi-K3`,
  `threw-withheld=none` and `waited=0ms`.
  The bench widths are identical on all six
  (`checkers=3`,
  `translators=7`,
  `writers=8`),
  so the unserved seats stay seated.
  The translate lane's `re-seated before starting` line (06:08:41.510 UTC) lists `hf:Qwen/Qwen3.8-27B`,
  `glm-5.3` and `google.gemma-4-e2b` among the seven writers.

The repair lane's `chunk re-seated under a hold: checkers=... unserved-checkers=...` line printed zero times.
`pass-reseat.ts` prints it per chunk while a hold runs.
Whether the Bedrock credential hold should have triggered it was not tested.
The absence matters for one reason:
`hf:Qwen/Qwen3.8-27B` was the third checker seat in all 5 checker rounds
(5 warns `checker hf:Qwen/Qwen3.8-27B: ... seat unreachable`,
06:06:08 to 06:08:32 UTC),
so every checker round and every introduced-defect probe ran on 2 of 3 voices
(`checker round: 2/3 heard`,
`introduced-defect-probe round: 2/3 heard`).
The line that would have named that padding did not print.
In `zheermao21` the same line printed 23 times on the same build
(see [the run of zheermao21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-zheermao21-run.md)).
Why it printed there and not here:
cause not yet traced.

`SEATS DARK` did not print.
The `seatReportLines` helper in `seat-tally.ts` prints it only for a seat that was asked and produced nothing usable,
and states its count over seats asked.
No seat was asked and dark,
because the three dark seats are now `asked=0`.
So the run's dark count is 0 of 7 seats asked (`CuspariaKLSY20`:
3 of 10).
The `unserved=` fields and the `unreachable=` counts carry the fact that three seats served nothing;
a reader who greps only for `SEATS DARK` finds no line.

Quorums and throws:

- No stage threw:
  `threw=0` on all ten `SEAT` lines,
  0 `[error]` lines,
  and the entry settled.
- One round closed without a quorum:
  consolidation slice 1,
  `select`,
  at 06:14:41.381 UTC:
  `select round: 3/7 heard, 134864ms total, no quorum (3 of 4 needed), every ask settled`.
  `deepseek-v4.1-flash` returned an empty reply cut by `finish_reason=length` on NeuralWatt,
  and again on the OpenRouter re-ask (served by Morph,
  92612 ms),
  and the voice was lost.
  The `runRecoveryRound` re-asked that one voice
  (1 of 1 heard after 33737 ms on NeuralWatt,
  0.2875 Wh) and the decision closed with 4 ballots:
  `candidate 1 from incumbent(matched by minimax-m3) won weight 2.5 (ballots: 4)`.
  So the short close was recovered inside the same stage.
- With 7 seated and 3 unreachable the absolute naturalness lines print `quorumNeeded=4 quorumOver=7 unreachable=3`,
  so the reachable bench of 4 equals the bench quorum of 4
  (`stage-reachable-quorum.ts` shortens a quorum only when the reachable bench is short of the bench quorum).
  Any one lost or slow voice therefore either stalls the round to the slowest seat or closes it short.
  The 69606 ms,
  112266 ms and 134864 ms rounds are that effect
  (the read's reading of the code and the lines,
  a guess about cause).
- Elsewhere the lane rounds closed at their quorum:
  `lane-contest round: 4/7 heard` on all three contested slices,
  and the translate rounds at `4/7`.

### Archive blocks

The archive block review is not exercised by this entry.

- `preparation.blockPairing[0].pairs` of the artifact holds 18 pairs,
  `0-0` through `17-17`.
  The log says `paired 18 of 18 original and 18 of 18 translation blocks across 18 relations, from 5 usable voices of 5 heard`
  (06:04:52.997 UTC;
  `preparation.sectionPairing.kind` is `deterministic`).
- An archive block review is for archive blocks that pairing leaves unclaimed by any source block
  (`archive-block-review-stage.ts`,
  `pass-prepare.ts`).
  Here none is unclaimed.
- The log has no `archive block` line of any kind (`rg --ignore-case "archive block"`,
  0 matches),
  and a count of `archive.?block|unclaimed` over the artifact JSON matches nothing.
- So there are no candidates,
  assessments,
  dropped replies or winner to report.
  The composite candidate,
  the writer window,
  the kept findings and the whitespace-free quote anchoring are not exercised by this entry.

`DESTINATIONS CuspariaKLSY source=0 page=0 dropped=0`:
the same three zeros as in `CuspariaKLSY18` to `CuspariaKLSY20`.
The entry has no link,
image or reference to compare
(a search of the source,
archive and shipped page for `http`,
`](` and `![` finds none),
so `dropped=0` is true but could not have been otherwise here.

### What the run read did not do

- It could not read the pinned commit `a41fc607ea` from disk (no `.git` in the corpus directory);
  it confirmed the source and archive by bytes against the artifact.
- It read code only with `git show` and `git grep` on `3637592bd4`.
  It did not read the tip of `CuspariaKLSY20` (`31ce5369e1`),
  so any statement about what changed since then rests on the two runs' logs and artifacts,
  not on a diff of code.
- The NeuralWatt retry spacing and `laterStartAt` paths were not exercised:
  the first quota read succeeded.
  The run cannot say that they work.
- The Bedrock credential was already expired (HTTP 401);
  whether the `chunk re-seated under a hold` line should have printed was not tested.
