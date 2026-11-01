# OpenRouter and the corpus passes: the run of zheermao21 on build 3637592bd4 (2026-10-21)

What was read on 2026-10-21 of the run of the first pass of the restarted rotation:
the log,
the NeuralWatt meter,
the seats,
the archive block review of the letter and the email,
and why the repair lane's counts rose.
Entry:
zheermao21 (entry `zheermao101`).
The shipped page,
line by line,
is in
[the page of zheermao21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-zheermao21-page.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the three passes of the restart share are in
[the rotation restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md).

## zheermao21 read, 2026-10-21, 06:28 UTC: SETTLED in 40.7 min of pass; no class named by the read; the letter recovered, the quote and the email regressed, two U+2014 characters

`zheermao21` (entry `zheermao101`) ran on tip `3637592bd43652fe2c6ff3cf2ec31e5a27c39c7d`
and pipeline `sha256-tree-v1:06c97a69446dc452fa2c602da834c36febb7f0d7b4b1010b5dfa9fab81097eec`,
and its `START` line carries both values as given.
The launch brief gave the start as 05:20:09 UTC on 2026-10-21;
the `START` line carries no time.
The log's first timestamped line is 05:21:48.250 UTC and its last is 06:02:32.294 UTC.
The corpus input is the pin `a41fc607ea`
(the artifact's `corpusSha` reads `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`).

The reader read the log (4366 lines),
the run directory (the artifact `zheermao101.json`,
the stored prompt payloads and the shipped `fixed/people/zheermao101/page.en.md`)
and the source of the entry in the one-among-us data tree.
The three photos were read as images.
No file in any git checkout was edited,
no pass,
probe or provider call was run,
and no `.env*`,
auth or `environ` file was read.
Code was read with `git show` and `git grep` at `3637592bd4`,
not from the working tree.
Every count names its method:
a script that counts log lines matching a pattern,
the same as `rg --count`.

The data tree is not a git checkout,
so equality of its `page.en.md` with the pin is not verified.
What could be checked:
the artifact's `preparation.archiveText` is the archive after the review round,
and a `diff` of it against the working-tree `page.en.md` differs only in the reviewed blocks and the moved note,
so the working-tree copy agrees with what the lanes used everywhere the review did not touch.

### Run facts

The `TALLY` line reads:

```text
TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired repairIssues=28 repairAccepted=18 repairResolved=16 repairFindings=321 repairChanged=10 translateStatus=complete translateChanged=10 documentsDiffer=11 pageChanged=10 pageSilent=0 alignmentFindings=13 selection=contested ms=2443411
```

The `DONE` line reads `DONE processed=1 of pending=1; artifacts=1/1 declined=0 ... elapsed=2443778ms`,
which is 40.7 min.
`DESTINATIONS zheermao101 source=2 page=3 dropped=0`,
and `publish: wrote 15 slices into a page of 5869 characters`
(`wc --chars --bytes` gives 5869 characters and 6005 bytes;
`zheermao20`:
6007 and 6137).
`FRONT MATTER entry=zheermao101 authority=archive` (05:21:51 UTC).

Beside `zheermao20` (figures from its read;
the `TALLY` ones are the line it quotes):

- `repairIssues` 28 against 16.
- `repairAccepted` 18 against 13,
  `repairResolved` 16 against 13.
- `repairFindings` 321 against 220.
- `repairChanged` 10 against 7.
- `translateChanged` 10 on both.
- `documentsDiffer` 11 against 10,
  `pageChanged` 10 against 8.
- `alignmentFindings` 13 on both,
  and `selection=contested` on both.
- `ms` 2443411 against 1977563,
  so 7.7 min longer.

Where the extra 7.7 min went,
from the log timestamps (this run,
then `zheermao20`,
each read with `rg` on the same phase lines):

- Start of log to `both lanes over 15 slices`:
  27.3 min against 20.4 min.
- Lanes to the end of the lane contest (`consolidation: N contested slices to settle`):
  9.4 min against 8.6 min.
- Consolidation to `publish`:
  4.0 min against 4.0 min.
- Inside the first phase,
  the step the log words `read N characters without a model` (a local read of each photo)
  took 98 s,
  120 s and 47 s for `photo1.webp`,
  `photo2.webp` and `photo3.webp` here,
  against 1.6 s,
  0.8 s and 0.8 s in `zheermao20`;
  the three read the same 515,
  422 and 388 characters in both runs.
  That is about 4.4 min of the 7.7 min.
  Nothing in the log says why;
  load on the host is a guess,
  and the host was not checked.
- The archive block review ran 05:29:56 to 05:48:28 UTC,
  18.5 min (1113 s,
  45.5 percent of the entry),
  against 996.9 s and 50 percent in `zheermao20`
  (figures from the `zheermao20` read and the investigation of its letter).

Counts over the log
(the `zheermao20` figure in brackets is from its read):

- `[warn]` lines:
  491 (424);
  `warn` appears on 491 lines,
  so no summary line quotes one (`zheermao20`:
  425).
- `[error]`:
  none.
- `DEFECTS`,
  `REATTEMPT`,
  `STALLED`,
  `mdx-downgraded`,
  `StreamOverrunError`:
  none of the five.
- `Interrupted` and `TranslationRepairInterruptedError`:
  none (`rg --count` exits 1).
- `NoProviderForModelError`:
  none as a string;
  the refusals are worded `no provider can take <model>: ... seat unreachable`.
- `seat unreachable`:
  420 lines (245).
- `HTTP 401`:
  3 lines (3);
  `HTTP 402`:
  none (1);
  `HTTP 429`:
  10 lines (12),
  all from 05:52:57 to 05:53:09 UTC.
- `schema-mismatch`,
  `SyntaxError` or `unusable pairing`:
  16 lines (7),
  nearly all `inception/mercury-2.5`
  (its `SEAT` line says `unusable=6`) and one `minimax-m3`.
- `closed on a short bench`:
  none (none).
- Decision lines reading `keeping the fallback` or `keeping the repaired text`:
  12 (6).
- `consolidation: 11 contested slices to settle` at 05:58:31 UTC (`zheermao20`:
  10);
  consolidation ended 06:02:31 UTC.
- `abandoned ... ms after quorum`:
  6 lines,
  all `hf:Qwen/Qwen3.8-27B`.
- Warn kinds that `zheermao20` had and this run does not:
  `refused by the page grammar` (3 there,
  0 here) and `MdxParseError` (2 there,
  0 here).
  The picture-tag slice (slice 7) matched on all candidates this time.
- `archive-revision-refused`:
  2 lines (1),
  both `gemma-4-26b-a4b-it`;
  `introduced repetition`:
  1 (1).
- The build output carries 3 `UNRESOLVED_IMPORT` lines (6);
  they are build output,
  not run warns.

### NeuralWatt: baseline, stop and energy

Baseline:

- One `takeBaseline` line,
  at 05:21:49.671 UTC:
  `NeuralWatt quota at the start of the run: 3.1687 kWh charged this period`.
  It is the plain wording,
  not the late one (`... after the first read of the run failed ...`).
  The read behind it is the `stream quota: completed, elapsed 1198ms` line at 05:21:49.667 UTC.
- `laterStartAt` appears in no log line (`rg` for the name finds nothing);
  the `warn` that opens a failed baseline
  (`NeuralWatt quota could not be read before the first call`) appears zero times;
  no `NeuralWatt /quota returned HTTP` line appears.
- All 26 `stream quota:` lines of the run read `completed`.
  So the first read succeeded,
  the 60 s retry rule of `neuralwatt-quota-session.ts` never had a failure to act on,
  and this run cannot say whether it works.
- `decideNeuralWattDayAnchor` (05:21:49.672 UTC):
  the day starts at 19:15:08.000 UTC,
  the current day having begun 2026-10-20T19:15:08.000Z.
  This run (05:21 to 06:02 UTC) is inside the same NeuralWatt day as `zheermao20` (02:15 to 02:48 UTC).
- End line:
  `NeuralWatt plan energy: no plan energy was spent between the start and the end of the run; the free daily energy is not in this figure`.
  That is the wording of `describePlanSpend` for a measured difference of 0 with no later start.

Stop:

- No stop line fired.
  `rg` for `markRefused` finds two lines,
  both `synthetic`;
  none names NeuralWatt.
  All 25 `METERS` lines read `neuralwatt=wet`.
- The meter's own stop rule (`neuralWattIsDry`,
  `neuralwatt-meter.ts`) is:
  no day start,
  a blocked key,
  or less left than
  `NEURALWATT_SMALLEST_ESTIMATE_WH` (0.0011364 Wh at the tip,
  the smallest first-bucket mean times the margin of 1).
  The lowest remaining read was 424.911 Wh,
  so the rule was never near.

Energy and the meter:

- The energy tally script on the log gives
  247 settled calls and 44.7156 Wh measured.
  A second script,
  `meter-reconcile`,
  gives the same 247 calls and 44.7156 Wh,
  and there are 247 `SPEND provider=neuralwatt` lines (`rg --count`).
  Estimated for the same calls:
  36.4328 Wh,
  so measured is 1.23 times the estimate (`zheermao20`:
  31.332 Wh measured,
  30.728 Wh estimated,
  215 calls).
- `deepseek-v4.1-flash`:
  180 calls,
  19.1671 Wh measured (13.4865 Wh estimated).
  `glm-5.3-flash`:
  67 calls,
  25.5485 Wh measured (22.9463 Wh estimated).
- Energy per call:
  0.181 Wh against 0.146 Wh in `zheermao20` (31.332 Wh over 215 calls).
- The single largest call was a `glm-5.3-flash` review of the letter:
  3.2377 Wh against an estimate of 0.3725 Wh,
  77,405 ms,
  70,467 reasoning characters,
  0 content,
  18,316 completion tokens,
  `finish_reason=length` (05:42:39 UTC).
  In `zheermao20` the same call on the same block ran 4.1361 Wh to the same 18,316 tokens.
  The largest `deepseek-v4.1-flash` call was 1.4175 Wh.
- The estimate table in this build is the refit one
  (`0.0758` and `0.3725` Wh for the 4 to 16 thousand token bucket,
  from `zheermao19`);
  64 of 177 `deepseek-v4.1-flash` calls and 20 of 60 `glm-5.3-flash` calls in that bucket came in over their estimate.
- `neuralwattWh` (what the meter reports as left of the 550 Wh day):
  468.773 Wh at the first `METERS` line (05:21:51 UTC)
  to 424.911 Wh at the last (06:01:35 UTC);
  a fall of 43.862 Wh.
- The 231 calls settled between those two lines sum to 43.490 Wh.
  The meter's `neuralwattAccountWh` (the account's own covered energy) rose from 81.226 Wh to 124.717 Wh,
  which is 43.491 Wh,
  matching the call sum to the third decimal.
- The two figures do not reconcile the way `zheermao20`'s did:
  550 less 424.911 is 125.089 Wh counted as spent,
  against 124.717 Wh on the account.
  The meter counts from the larger of the ledger's spend and the account's (`neuralWattMeterLevel` text),
  so the ledger side reads 0.372 Wh higher than the account at the end.
  Traced on 2026-10-21 (UTC),
  original text kept:
  the meter also subtracts the energy reserved for calls in flight,
  and 0.372 Wh is the estimate of one `glm-5.3-flash` call ([the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md),
  "Finding 9").
  Cause not yet traced (before that tracing);
  `neuralwattReckonedWh` reads 0.000 on every line,
  so the gap is not a share resting on estimates.
- The 16 calls after the last `METERS` line sum to 1.226 Wh,
  so the meter would stand near 423.685 Wh at the end;
  that figure is computed,
  no line prints it.
- The first reading is 21.244 Wh under `zheermao20`'s last (490.017 Wh),
  on the same NeuralWatt day.
  The counted spend went from 59.983 Wh (550 less 490.017) to 81.226 Wh between the runs (02:48 to 05:21 UTC);
  the read does not explain that gap.
  Cause not yet traced.
- Plan energy:
  the run's end difference is 0 kWh (the `3.1687 kWh` baseline and the end reading agree),
  so no plan energy was spent.

### Seats, units and meters

The `SEAT` lines (12) print after the `DONE` line,
with the new `unreachable=` field:

- `glm-5.3`:
  asked 0,
  unreachable 93.
- `minimax-m3`:
  asked 183,
  usable 182,
  unusable 1.
- `google.gemma-4-e2b`:
  asked 0,
  unreachable 148.
- `gemma-4-26b-a4b-it`:
  asked 162,
  usable 162,
  unreachable 3.
- `deepseek-v4.1-flash`:
  asked 181,
  usable 181.
- `hf:zai-org/GLM-5.3-Flash`:
  asked 70,
  usable 70.
- `inception/mercury-2.5`:
  asked 180,
  usable 174,
  unusable 6.
- `hf:Qwen/Qwen3.8-27B`:
  asked 22,
  usable 22,
  abandoned 6,
  unreachable 160.
- `hf:moonshotai/Kimi-K3`:
  asked 24,
  usable 24,
  unreachable 19.
- `google.gemma-4-31b`:
  asked 0,
  unreachable 3.
- `xiaomi/mimo-v2.6-flash`:
  asked 28,
  usable 28.
- `typesafe/jev-1.13`:
  asked 36,
  usable 36.

Read against the `zheermao20` seat table (its read):

- `threw=0` on all 12 lines.
  In `zheermao20` the same refusals were `threw`:
  `hf:Qwen/Qwen3.8-27B` 137,
  `hf:moonshotai/Kimi-K3` 19,
  `glm-5.3` 4,
  `google.gemma-4-e2b` 4,
  `google.gemma-4-31b` 3,
  and 3 of the 5 for `gemma-4-26b-a4b-it`.
  The change shows:
  a refusal is now `unreachable` and no longer a throw or an ask.
- The `unreachable=` figures sum to 426.
  The 420 `seat unreachable` warn lines split as `glm-5.3` 93,
  `google.gemma-4-e2b` 148,
  `hf:moonshotai/Kimi-K3` 19 and `hf:Qwen/Qwen3.8-27B` 160,
  which equal the `SEAT` figures.
  The other 6 are the picture reads:
  `rg` for `failed outright` finds 6 lines,
  three each for
  `gemma-4-26b-a4b-it` and `google.gemma-4-31b` (the Bedrock-only vision route,
  `credential refused this run (bedrock)`).
- `gemma-4-26b-a4b-it` answers text calls through OpenRouter (`served by "DeepInfra"`)
  after the credential hold-out moved it,
  as in `zheermao20`;
  its two content-bound cuts of `zheermao20` did not recur.
- `SEATS DARK` does not print.
  The seat-tally helper appends it only when a seat that was asked produced nothing usable,
  and counts only seats asked;
  its own text says a seat the router refused before any request is on its `SEAT` line
  as `unreachable=N` and is not counted.
  The three seats with `asked=0` are exactly that.
  (`zheermao20` printed `SEATS DARK: 3 of 12`.)
- `JUDGE SEATS` lines print `unserved=`:
  49 lines.
  The first,
  at 05:21:51 UTC,
  reads `unserved=glm-5.3` (Hyper dry).
  From 05:23:05 UTC it adds `google.gemma-4-e2b` and `google.gemma-4-31b` (Bedrock refused).
  From 05:53:10 UTC it adds `hf:Qwen/Qwen3.8-27B` and carries `withheld=hf:moonshotai/Kimi-K3`.
  All 49 carry `threw-withheld=none`.
  Kimi is `withheld=` and Qwen is `unserved=` in the same lines.
  Why the two are classed differently:
  traced on 2026-10-21 (UTC),
  original text kept:
  Kimi is in the withheld list and was dropped from the benches on purpose,
  Qwen stays seated and no wet provider serves it ([the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md),
  "Finding 10");
  cause not yet traced (before that tracing)
  (in `zheermao20` Kimi was `withheld=` and Qwen was not listed at all;
  here Qwen is listed as `unserved=`).
- The repair lane's re-seat line ends `unserved-checkers=hf:Qwen/Qwen3.8-27B`,
  23 lines,
  each beginning
  `chunk re-seated under a hold: checkers=xiaomi/mimo-v2.6-flash,inception/mercury-2.5,hf:Qwen/Qwen3.8-27B`.
  (`CuspariaKLSY21` printed no such line;
  see [its record](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-cuspariaklsy21-run.md).)

Did any quorum close short?
Two rounds did,
and nothing threw.

- Slice 3 (the game title),
  repair lane,
  05:53:10 UTC:
  `introduced-defect-probe: bench short of quorum, reachable 1 of 3; closing on 2 voices`,
  then
  `introduced-defect-probe round: 1/3 heard ... no quorum (1 of 2 needed)`,
  then
  `a stage heard fewer than quorum (stage-quorum-unmet (introduced-defect-probe 1/3)), so the slice ships unchanged and is NOT cached`.
  Its cause is the Synthetic hold:
  `hf:moonshotai/Kimi-K3` and `hf:Qwen/Qwen3.8-27B` were refused 0.8 s apart in that probe.
- Slice 11 (the Telegram sentence),
  consolidation,
  06:01:35 UTC:
  `absolute naturalness review: 3/7 usable, quorum-not-met, quorumNeeded=4 quorumOver=7 unreachable=3`.
  `inception/mercury-2.5` answered invalid JSON (an unescaped quote in a `problem` string),
  so the fourth voter was lost.
  `consolidation-polish.ts` says what that does:
  `absolute naturalness review quorum not met; recorded as evidence`,
  and the gated text ships.
- The other 14 of the 15 naturalness reviews read `4/7 usable ... quorumNeeded=4 quorumOver=7 unreachable=3`.
  The `zheermao20` read warned that any one of the four voters failing would leave a review short;
  once of 15 did.
- `[error]`:
  none;
  `Interrupted`:
  none;
  `SEAT ... threw=`:
  0 on 12 lines.

Bedrock,
Synthetic,
Hyper and
OpenRouter:

- Bedrock:
  the key was still expired.
  The first refusal is at 05:21:54.992 UTC
  (`SEAT CREDENTIAL REFUSED gemma-4-26b-a4b-it on bedrock ... HTTP 401 ...`),
  followed by the same hold-out wording as `zheermao20`.
  `METERS` reads `bedrock=wet` and `bedrockUsd=5.02` on every line,
  while the `JUDGE SEATS` lines read `bedrock=dry` from 05:23:05 UTC.
  The read reports the difference and does not diagnose it.
- Hyper:
  `hyper=dry` and `hyperBalance=0` on all 25 `METERS` lines;
  `rg` finds no `SPEND provider=hyper` line.
  `glm-5.3` had no answer:
  all 93 attempts were refused before any request as `every provider serving this model is out of budget`.
- Synthetic:
  weekly meter 2% at 05:21:51 UTC,
  0.02% at 05:51:28 and 05:52:43 UTC,
  then dry at 05:53:09.782 UTC
  (`synthetic=dry`,
  `syntheticWeekly=0%`),
  31.4 min into a 40.7 min run (`zheermao20`:
  22 of 33.0 min).
  `Kimi-K3` and `Qwen3.8-27B` answered `HTTP 429` and retried 4 times from 05:52:57 UTC;
  `markRefused` logged
  `synthetic: refused us while its meter reads dry ... held out for 300000ms` (05:53:09 and 05:53:10 UTC).
  There is no `HTTP 402` this time.
  Kimi's last refusal line is at 05:56:07 UTC;
  Qwen's refusals run to the last stage (06:02:24 UTC).
  Synthetic took 42 calls by the `SPEND` lines (20 `Qwen3.8-27B`,
  22 `Kimi-K3`) and 97,358 completion tokens;
  the `SEAT` lines count 22 and 24 asked for the two,
  each 2 more than the `SPEND` lines;
  the difference was not reconciled.
- OpenRouter:
  14.35 USD at the first `METERS` line to 13.87 USD at the last;
  the `SPEND provider=openrouter` lines sum to 0.543 USD
  (`minimax/minimax-m3` 0.290,
  `google/gemma-4-26b-a4b-it` 0.115,
  `inception/mercury-2.5` 0.104,
  `xiaomi/mimo-v2.6-flash` 0.017,
  `typesafe/jev-1.13` 0.016,
  `z-ai/glm-5.3-flash` 0.001).

The condition against `zheermao20`:
the bench was the same four voters in every consolidation round (`4/7 usable`),
and one of the 15 naturalness reviews fell one voter short.
What changed is when the thinking voices went:
Synthetic ran dry at 05:53:09 UTC,
4.0 min after the lanes started (05:49:08 UTC),
where in `zheermao20` it ran dry 1.5 min after they started (02:37:26 UTC against 02:35:53 UTC).
Slices 0 to 3 each had 5 critics heard here (see why `repairIssues` rose).

### The letter block

This is the question the investigation of the archive block review was written about.
The fixes B712 to B715 are not in this build,
so the review still picks one whole revision,
still waits the 120 s reader grace,
and still anchors a retention quote by exact text.
All three are visible in the log (`120031ms in grace`;
`candidate 1 from deepseek-v4.1-flash won`;
the quote handling in the email block).

The reviews ran last block first,
one block at a time.
The artifact records `archive block reviewed and revised` for `block/20`,
`block/19`,
`block/14` and `block/13`,
and `unclaimed archive blocks remain after the single correction round: 5`;
that last sentence is in the artifact only,
not in the log.
`block/14` is the letter,
`block/13` its label,
`block/20` the email,
`block/19` its label.
`block/21` was not reviewed and the read did not identify it:
a count of the archive's top-level blocks makes it the `Later, ...` paragraph,
but slice 11 carries that paragraph as its archive text,
so the count is not reliable and the read does not rely on it.

The letter's review ran 05:41:21.5 to 05:47:10.2 UTC (349 s;
`zheermao20`:
463 s for review and select).

Review round (log lines 514 to 541;
the replies are in the stored prompt payloads,
named here by the first eight characters of their ids):

- Seated 8,
  the round closed `5/8 heard, 205897ms total, 85866ms to quorum, 120031ms in grace`.
- `inception/mercury-2.5`,
  7.6 s:
  `source-supported` (reply `212ef0f3`).
  Its `sourceQuote` is an English sentence taken from the archive block (`For example, as a trans person, ...`).
- `google/gemma-4-26b-a4b-it`,
  8.6 s:
  `revise` (`76e47496`),
  1968 characters,
  15 lines.
  Refused:
  `the block is 1 block (blockquote) and the revision is 7 blocks (paragraph x7); a revision keeps the block's own shape`.
- `deepseek-v4.1-flash` on NeuralWatt,
  35.0 s:
  `revise` (`8c55f4eb`),
  2093 characters,
  31 lines;
  29,052 reasoning characters,
  8,050 completion tokens,
  0.6496 Wh.
- `hf:zai-org/GLM-5.3-Flash` on NeuralWatt:
  77.4 s,
  0 content,
  18,316 tokens,
  `finish_reason=length`,
  3.2377 Wh,
  `schema-mismatch`;
  re-asked on OpenRouter (Wafer):
  6.6 s,
  `revise` (`7b43e731`),
  2137 characters,
  36 lines.
- `hf:moonshotai/Kimi-K3` on Synthetic,
  84.3 s:
  `revise` (`6b66eba2`),
  2163 characters,
  36 lines.
- `hf:Qwen/Qwen3.8-27B` on Synthetic:
  cut `120000ms after quorum`
  (`cut-mid-reply after 0 answer chars and 46344 reasoning chars (1249371 chars on the wire), first byte at 2957ms`),
  voice lost.
  It is the third archive review round in which Qwen was cut
  (the email,
  the email's label,
  and this one;
  each with 0 answer characters).
- `minimax-m3` was not asked in this round
  (no stream line for it between the two block rounds).

Candidates and assessments:

- `archive review: comparing 3 admissible revisions in 4 candidates; 4 eligible assessments from 5 heard reviews`.
- Candidates:
  the three admissible revisions and the archive text.
  By the ballot texts (a reading,
  not a field) candidate 1 is `deepseek-v4.1-flash`'s,
  candidate 2 is the OpenRouter `GLM-5.3-Flash` re-ask (it fixes the QQ sentence and nothing else),
  candidate 3 is `Kimi-K3`'s (the QQ sentence and `clinging to life`),
  candidate 4 is the archive.
- Dropped:
  `gemma-4-26b-a4b-it`'s revision,
  for shape (7 paragraphs for 1 blockquote,
  per the refusal line quoted for that reply).
  Its reply still counts among the 4 eligible assessments,
  since `revise` is kept unconditionally (`archive-block-review-stage.ts`).
- Dropped:
  `inception/mercury-2.5`'s `source-supported`.
  The count says so:
  5 heard,
  4 eligible,
  and it is the only reply that is not a `revise`.
  Why:
  `isArchiveSourceQuoteAnchored` looks for the quote in the source text,
  and an English quote of the archive cannot be in a Chinese source
  (inference from the code and the quote).
- Dropped for a hard line wrap:
  none in the letter.
  The wrap refusal appears in the email block.
- Not heard:
  Qwen.

Select round:
`6/7 heard, 142408ms total, 66844ms to quorum, 75564ms in grace`.
Qwen completed inside the grace this time (135.3 s,
681 content characters) and voted for candidate 1.
All six ballots chose candidate 1;
the result is `candidate 1 from deepseek-v4.1-flash won weight 5.5 (ballots: 6)`
(its own ballot at 0.5).
The ballot texts name what candidate 1 fixes that 2 and 3 do not:
the QQ sentence with `so I couldn't add you as a friend`,
`our feelings and wishes ... our future`,
`trans women` for MtF,
and `clinging to life and dreading death ... scrape by`.

What shipped:
the letter on the page (lines 53 to 83) is `8c55f4eb` exactly;
a `diff` of those 31 lines against the reply's `replacementText` differs only in the trailing newline.
The archive text was not kept.

Against the investigation of the `zheermao20` letter:

- It ranked the failure as disjoint partial revisions and a Qwen cut at the 120 s grace.
  Here the revisions overlapped (all three fix the QQ sentence) and one of them carried every defect any reviewer named,
  so a whole-revision pick was enough,
  and the Qwen cut did not matter (Qwen's select ballot agreed).
- It predicted `deepseek-v4.1-flash` would run to its cap on this block,
  from `zheermao19`.
  It did not:
  35 s,
  8,050 tokens,
  a complete 31-line revision.
  So the depth claim ("every thorough revision came from Hyper") does not hold for this run;
  the comprehensive revision came from a NeuralWatt call.
- The `glm-5.3-flash` NeuralWatt call that hit its cap with no content in `zheermao20` (4.1361 Wh)
  did so again (3.2377 Wh),
  and was re-asked on OpenRouter again;
  that is the one repeat.
- One sample each cannot separate this from run to run spread,
  and the read did not measure that spread.

The email block (`block/20`) for comparison,
same terms:

- Seated 7,
  round `5/7 heard, 179766ms total, 59643ms to quorum, 120123ms in grace`;
  Qwen cut (0 answer characters,
  40,744 reasoning characters).
- `comparing 2 admissible revisions in 3 candidates; 4 eligible assessments from 5 heard reviews`.
- Revisions:
  `minimax-m3` (`c1f72b2c`,
  which glosses `Zheermao September (Antares)`) and `Kimi-K3` (`77af4421`).
  `gemma-4-26b-a4b-it` was refused (`the block carries 3 quoted passages and the revision carries 1`).
- Dropped:
  one of the two retention (`source-supported`) replies.
  The counts prove that much:
  5 heard,
  4 eligible,
  3 of the 5 are `revise` (always kept),
  so exactly one `source-supported` reply failed the anchor test,
  and no log line says which.
  Which one,
  and why,
  is inferred from the quote shapes and the code:
  `inception/mercury-2.5`'s reply (`7beb775b`) quotes `如果成功死去了 请把我挂在上面，好嘛`,
  two transcript lines joined by a space,
  while the `minimax-m3` reply's own `sourceQuote` (`c1f72b2c`)
  shows the transcript with a newline between those two lines,
  and `quote-normalize.ts` keeps `LINE_BREAK_CANON` out of the evidence fold
  ("admitting them is a behaviour change awaiting a decision").
  On that reading this is the hard-wrap refusal the brief named,
  seen once,
  in the email block,
  and it removed an archive-supporting assessment.
  The other `source-supported` reply (`GLM-5.3-Flash`,
  `d58d1afe`) quotes `是玖月折耳猫，我。。尽力活着了但是。。很抱歉，谢谢`,
  a single transcript line,
  so by the same reading it was kept.
- Select:
  `5/7 heard, 318255ms total, 318255ms to quorum, 0ms in grace`.
  Quorum came only when Qwen finished (309.5 s),
  the same race the investigation describes in its section 2.3.
  `inception/mercury-2.5`'s select answer was the schema template (`{"best": "integer>", "reason": "<string>"}`),
  which started a recovery round (`1 off-shape`,
  9.6 s) and was heard.
- Result:
  `candidate 1 from hf:moonshotai/Kimi-K3 won weight 3.5 (ballots: 6)`;
  the shipped email (lines 101 to 124) is `77af4421` exactly,
  differing only in the trailing newline.
  The note between translators was kept and put after the block
  (`replaced with 1 note between translators kept`,
  05:38:25 UTC).
- The two labels:
  `English translation of the email conversation:` won with weight 4.5 and
  `English translation of Arts’s letter:` with weight 5.5 (`GLM-5.3-Flash`);
  both remove the positional word `preceding` under the house rule.
  For the email's label,
  4 of 5 heard replies were not eligible (`1 eligible assessment from 5 heard reviews`).

### Why repairIssues rose from 16 to 28 and repairFindings from 220 to 321

The log and the artifact show why
(counts from per-run scripts over the artifact `zheermao101.json` of each run):

- Critics filed more claims:
  42 distinct claims against 26,
  and 64 critic hearings over the 15 slices against 59.
  `inception/mercury-2.5` filed 13 (7),
  `deepseek-v4.1-flash` 10 (5),
  `hf:Qwen/Qwen3.8-27B` 8 (5),
  `hf:moonshotai/Kimi-K3` 8 (8),
  `gemma-4-26b-a4b-it` 3 (1).
- Most of the extra claims are in the first slices:
  slice 0 had 5 critics heard and 14 claims (`zheermao20`:
  3 and 3),
  slice 1 had 5 critics and 5 claims (4 and 2),
  slice 2 had 5 and 2 (4 and 1),
  slice 3 had 5 and 3 (4 and 3).
  Slice 0 alone is 11 of the net 16 extra claims;
  slices 0 to 2 are 15.
  Slices 8,
  13 and 14 add 3 more (1,
  1 and 1 claims against none),
  and slices 4 and 9 each have one fewer.
  Slices 0 to 3 are the ones criticised before Synthetic went dry (05:53:09 UTC,
  slice 3 itself at the edge),
  with `Qwen3.8-27B` and `Kimi-K3` heard;
  `zheermao20` lost them 1.5 min after its lanes began.
  Why slice 0 here heard `inception/mercury-2.5` and `Qwen3.8-27B` where `zheermao20`'s slice 0 did not:
  cause not yet traced;
  the Synthetic timing is the read's guess at the cause,
  from the timestamps and the per-slice counts.
- The issues adjudicated:
  18 accepted (13),
  8 rejected (1),
  2 `needs-human` (2).
  So 5 of the 12 extra issues were accepted and 7 were rejected;
  slice 0 alone holds 9 issues.
  Three of the 8 rejected claims concern defects the page still has:
  the 有一天 claim (0 to 4),
  the 就好了呢 claim (1 to 3),
  and the "that long" claim.
- `repairFindings` is 101 higher,
  by kind:
  `panel-packet` 62 (30),
  `stage-voice-lost` 97 (60),
  `stage-roster-incomplete` 66 (47),
  plus the editor and refine kinds,
  and one-off findings
  (`stage-short-bench`,
  `stage-quorum-unmet`,
  `quote-not-found`,
  `introduced-repetition`,
  `refine-declined`,
  3 `duplicate-issue-merged`).
  Panel packets doubled because there were more issues to review.
  Voice-lost findings rose by 37,
  of which 34 are two seats that were never asked:
  `google.gemma-4-e2b` 25 (1) and `glm-5.3` 11 (1).
  Under the new counting neither is withheld (`withheld=none`) and every stage seats them as `unserved`,
  and each stage records a loss;
  `zheermao20` withheld them after they threw,
  so they were seldom seated.
  `hf:Qwen/Qwen3.8-27B` (49,
  was 46) and `hf:moonshotai/Kimi-K3` (12, 12) did not change.
  So much of the findings rise is a counting effect of the unreachable change,
  not worse behaviour;
  this is the read's inference from the counts.


### What the run read did not do

- It did not explain the 0.372 Wh ledger gap
  or the 21.244 Wh gap to `zheermao20`'s last reading.
- It did not trace why `Kimi-K3` is `withheld=` while `Qwen3.8-27B` is `unserved=`.
- It did not check the host for the cause of the slow local photo read.
- It did not reconcile the 2 asks by which the Synthetic `SEAT` lines exceed the `SPEND` lines.
- It did not run a spread measurement.
