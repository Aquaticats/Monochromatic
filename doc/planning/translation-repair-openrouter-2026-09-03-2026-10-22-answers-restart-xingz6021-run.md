# OpenRouter and the corpus passes: the run of XingZ6021 on build bcff23b8ce (2026-10-24 to 2026-10-25)

What was read on 2026-10-25 (UTC) of the run `XingZ6021`,
on the frozen build `bcff23b8ce`,
which carries the fixes for the defects that the read of `XingZ6020` found
(the build of `XingZ6020` was `1d46232457`):
the log,
the single attempt,
the providers,
the stages and where the time went,
and the Synthetic and NeuralWatt meters.
Entry:
`XingZ6021` (entry `XingZ60`,
the same 118-slice page as `XingZ6019` and `XingZ6020`).
It settled on its first attempt in 2 h 5 min 45 s and shipped a page,
and that page carries archive text which the source does not contain
(the Shinonome section,
in the defects file).

Hyper,
OpenRouter and Bedrock were out of the run from its first line
(Bedrock by the owner's ruling,
Hyper and OpenRouter dry on every reading),
so the run drew on NeuralWatt,
Synthetic,
Xiaomi,
DeepSeek and the owner's Claude key.
Claude is new against `XingZ6020`,
whose read names four providers and no Claude model:
Haiku 5.5 sits on the wide bench and Sonnet 5.5 edits
(`shared-contract/src/claude-card-sides.ts:33` to `:40`).

The seven checks of the build's fixes,
the page that shipped against the source and the archive,
and what the run could not test are in
[the stages of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-stages.md).
The Shinonome defect in full,
the other nine defect candidates,
the owner's rulings of 2026-10-25,
the second reader's corrections and the points no log settles are in
[the defects of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-defects.md).
The refusals,
errors,
lost voices and short-bench rounds,
with the log lines behind them,
are in
[the evidence of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-evidence.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of this rotation share are in
[the answers-restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md).
The runs before this one on the same entry are in
[the XingZ6019 record](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6019.md).

## How this record was made

A first reader read the log `temp/agent/corpus-pass-XingZ6021.log`
(21,947 lines by `wc`,
7,448,487 bytes)
and the run directory `corpus-pass-XingZ6021/` beside it,
and quoted the source at the run's tip `bcff23b8ce`
(`git show bcff23b8ce:<path>`,
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

- The read is `temp/agent/tr-fan/w312-xingz21/XingZ6021-read.md` (719 lines)
  and the check is `XingZ6021-read-check.md` beside it (418 lines).
- Line numbers in this record are those of the log.
  Times are UTC.
- Counts come from scripts in `temp/agent/tr-fan/w312-xingz21/` or from `rg --count`,
  and each figure names its method.
  The first reader's scripts are `calls.ts`,
  `rundir.ts`,
  `meters.ts`,
  `census.ts`,
  `gaps.ts`,
  `models.ts`,
  `energy.ts`,
  `httpcodes.ts`,
  `warns.ts`,
  `voicelost.ts`,
  `artkeys.ts`,
  `artshape.ts`,
  `phases.ts`,
  `stagetime.ts`,
  `lanes.ts`,
  `seatrefusals.ts`,
  `synthetic.ts`,
  `slicegrace.ts`,
  `idle.ts`,
  `drycalls.ts`,
  `tail.ts`,
  `noquorum.ts`,
  `spendmodels.ts`,
  `nwcheck.ts`,
  `gracesplit.ts`,
  `footnotes.ts`,
  `shippedlane2.ts`,
  `tags.ts`,
  `adjud.ts`,
  `majors.ts`,
  `warnbuckets.ts` and `windowcost.ts`.
  The first group was copied from the scripts of the `XingZ6020` read and had every constant of that run replaced or removed;
  the rest are new.
  The checker's own scripts are `c1.ts` to `c15.ts`,
  `a1.ts` to `a8.ts` and `dump.ts` under `check/`;
  it did not open the first reader's scripts or outputs.
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
  auth file,
  `/proc/<pid>/environ` or file under `/tmp/claude-1000/` was opened.
- The figures of `XingZ6020` quoted here for comparison are the lead's read of that run,
  which is not yet filed in this record
  (it lives at `temp/agent/tr-fan/w312-xingz20/XingZ6020-read.md`),
  so they carry no log lines in this record.

## Headlines

- The run ended `SETTLED` on its first and only attempt (line 21929),
  `artifacts=1/1 declined=0` (line 21931),
  `elapsed=7544901ms` (2 h 5 min 45 s;
  `Finished in 7560.86s`,
  line 21947).
  `attempts.json` reads `{"XingZ60": 1}`.
  One artifact (8,482,810 bytes) and one page (`fixed/people/XingZ60/page.en.md`,
  48,712 bytes,
  745 lines) were written.
  No `[error]`,
  `STALLED`,
  `INCOMPLETE`,
  `REATTEMPT` or `TALLY ERROR` line;
  one `OVERLAP` (line 200);
  one `TALLY` (line 21929);
  no `DEFECTS` line,
  which the build prints when a publish content check fails
  (`pass-entry-persist.ts:150` to `:160`);
  none failed.
- All 118 slices were filled:
  `translateStatus=complete`,
  `pageSilent=0`,
  `pageSourceShipped=0`.
  The thirty tail slices (88 to 117),
  which no wording backed in `XingZ6020`,
  were written by the translate lane and shipped.
  All nine footnote references on the page have a definition,
  both destinations survived (`DESTINATIONS source=2 page=2 dropped=0`,
  line 21930),
  and the HTML tags balance.
- Not one HTTP error status was returned by any provider
  (`rg` finds the string `HTTP` once,
  in the Bedrock line 198),
  one transport retry of 911 ms (line 4184),
  no hold,
  no hold backoff,
  no wait for a hold and no idle stretch.
- Idle wall clock with no model stream in flight:
  0.4 min in all (11 gaps of one second or more),
  no gap of 5 s or more.
  The check recomputed it by its own method and agrees:
  11 gaps of 1 s or more,
  0.38 min,
  14 gaps of 0.2 s or more,
  0.40 min.
  `XingZ6020` had 249.0 idle minutes after 00:28 UTC alone.
- Why 2 h 5 min and not 10 h 33 min,
  measured:
  no hold waits (0 against 228 waits and 1,030.8 summed minutes in `XingZ6020`),
  one attempt (against a 7 h deadline attempt plus a 3 h 33 min second attempt),
  the 30 tail slices written once in the translate lane
  (against about 64 min in `XingZ6020` for 48 attempts that filled nothing),
  the lane contest in 0.06 s (against 73 min),
  and the consolidation of 101 slices in 21.1 min (against 85.4 min for 63 slices).
  Part of the saving is a loss:
  the Synthetic weekly credit (23.9 percent at the start) reached 0 at 00:01:55 UTC,
  and Kimi and Qwen were out of every bench from there except in a window of 5.6 min,
  so later benches were mostly one or two voices.
- That loss is the quality story of the run.
  After 00:01:55 UTC,
  225 of the 233 issue adjudications (96.6 percent) ended `needs-human`,
  among them 78 major and 3 critical,
  against 9 of 108 before it (the check recounted both).
  224 of the 225 had a panel of exactly two voices and an electorate of weight 2,
  and the floor is weight 3
  (`adjudication/src/tally-votes.ts:81`,
  `shared-contract/src/adjudicate-model.ts:312`);
  the other one had four voices and a split.
  `repairChanged` is 14 of 118 slices (80 in `XingZ6020`).
  The page carries the repair or archive wording in 51 slices where the translate lane wrote something else
  (read by the first reader only).
  Among them is the Shinonome section (slices 67 to 70),
  which is the archive's text as it stands,
  with a quotation from *Metal Gear Solid 2* and two paragraphs built on it that the source does not contain,
  after the run's own critics said so.
- Ten defect candidates and observations (N1 to N10) are in the defects file;
  none was fixed in this run's build.
  The owner ruled on N1 on 2026-10-25 and the change is in progress (no commit yet).

## Run facts

- The build finished in 11.03 s (line 193).
  `ONLY XingZ60 ...` (line 195) says the entry was chosen by hand for a throwaway runs directory.
- `START` (line 196):
  tip `bcff23b8ce4179b56b7ab4029ce35c2dd06cfd47`,
  pipeline `sha256-tree-v1:899b55a69b741024df2ddd2039e9362b2adb79e83447d41c680778cfdc967862`,
  `files=3 pending=1 done=0 soft=259200000ms hard=25200000ms call=360000ms claude-call=460000ms`.
  The artifact's `tip`,
  `pipelineDigest` and `corpusSha` (`a41fc607ea5a70d8a7625cc67d5ed8c444f53379`) match.
  `WRITER GRACE built in`:
  writer rounds abandon stragglers 180,000 ms after quorum,
  the others 120,000 ms (line 197).
  `PROVIDER OFF bedrock` (line 198,
  the owner's ruling of 2026-10-20:
  Bedrock's token expired,
  HTTP 401 on every call).
- `OVERLAP XingZ60 value=4 source=fallback` (line 200):
  the pass default of four slices in flight,
  as in `XingZ6020`.
- The first timestamped line is 22:49:32.545 UTC on 2026-10-24 (line 199);
  the last stream line is 00:55:17.701 UTC on 2026-10-25 (line 21932);
  `DONE` (line 21931) reads `elapsed=7544901ms`;
  the artifact's `durationMs` and the `TALLY` `ms` are 7,543,633.
  The first and last timestamps are 7,545,156 ms apart,
  255 ms more than `DONE`.
- Run directory `temp/agent/corpus-pass-XingZ6021/` (`rundir.ts`;
  the check recounted the files):
  `attempts.json` `{"XingZ60": 1}`;
  `artifacts/` 1 file;
  `fixed/` 1 file;
  `ledger/` 354 files;
  `prompt-payloads/` 2,505 files
  (equal to the 2,505 `SPEND` lines);
  `slice-cache/` empty
  (a settled entry discards its slice cache).
- The entry:
  `15 aligned units, 118 slices, 45 alignment findings` (line 1470).
  The archive page stops at the ninth friend's section and carries the stub marker `Under Construction`
  (archive line 358),
  which the pass strips (lines 213 to 216:
  four stub markers stripped,
  one warn each).
  `untranslated tail: 30 slice(s) after the last agreed pair, 5131 source code points at expansion 2.94,
  expected 15074, last pair renders 147: admitted on that bound` (line 1465)
  is the pass's admission of the tail it expects the translate lane to fill.
- `TALLY XingZ60 status=SETTLED slices=118 repairStatus=repaired repairIssues=341 repairAccepted=89 repairResolved=65
  repairFindings=3925 repairChanged=14 translateStatus=complete translateChanged=100 documentsDiffer=101 pageChanged=78
  pageSilent=0 pageSourceShipped=0 standingUnendorsed=66 alignmentFindings=45 selection=contested ms=7543633`
  (line 21929).
  Against `XingZ6020` (in parentheses,
  from the lead's read of that run):
  `repairChanged` 14 (80),
  `repairAccepted` 89 (254),
  `repairResolved` 65 (250),
  `translateChanged` 100 (41),
  `documentsDiffer` 101 (63),
  `pageChanged` 78 (61),
  `translateStatus` `complete` (`unfilled`),
  `pageSilent` 0 (30),
  and two fields its `TALLY` did not print,
  `pageSourceShipped` and `standingUnendorsed`.
- `DESTINATIONS XingZ60 source=2 page=2 dropped=0` (line 21930):
  both the profile link and the bilibili link survived.
- `NeuralWatt plan energy: no plan energy was spent between the start and the end of the run` (line 21933).
- `[warn]` lines:
  7,692;
  `[info]` lines:
  14,016;
  239 lines carry no level or are blank
  (228 non-empty lines of build output and control lines,
  and 11 blank lines;
  the first reader's 240 counted one blank line too many,
  as the check found).
  The sum is 21,947.

## Timeline and stages

Times are UTC.
Each wall-clock figure is from the log's timestamps (`phases.ts`,
`lanes.ts`,
`idle.ts`).
"Idle" means no model stream was in flight
(each stream is the span from its end minus its elapsed time to its end,
meter reads excluded).
Summed times add concurrent rounds and are work time,
not wall clock.
The check recounted the windows of the three lanes and of the lane counts from the `SLICE-START` and `SLICE-COST` lines,
and they hold.

- 22:49:32 to 23:11:34 UTC (22.0 min),
  preparation (lines 199 to 1465):
  front matter from the archive (line 217,
  `authority=archive`);
  `JUDGE SEATS phase=preparation` (line 212,
  an info line) reads `wide=8 select=8 late=7 slate=7`,
  `waited=0ms`;
  the page-title lexicon round took 139.9 s with 120 s in grace on Qwen (line 241);
  block pairing took 24 rounds (22:52:44 to 23:03:10 UTC);
  two pictures were read (`image0.webp` by `hf:zai-org/GLM-5.3-Flash`,
  line 588,
  and by Kimi,
  line 591),
  and three more readers failed outright on the same picture because their providers were dry
  (`minimax-m3`,
  `gemma-4-26b-a4b-it` and `google.gemma-4-31b`,
  lines 595 to 597);
  coverage took 32 rounds (23:03:27 to 23:11:34 UTC),
  three of them with 100 s or more in grace.
- 23:11:34 UTC (line 1466):
  `both lanes over 118 slices, repair first`.
- 23:11:34 UTC to 00:13:32 UTC (62.0 min),
  the repair lane's chunk pass:
  118 `SLICE-START lane=repair` and 118 closes;
  88 `computed` and 30 `no-translation` (the tail,
  lines 10404 to 10491);
  summed slice time 246 min,
  of which 85 min (35 percent) was grace after quorum
  (`XingZ6020`:
  57 percent of 793 min).
  The first reader's median of 35.5 s for the 88 computed slices was not recomputed by the check,
  whose median over all 118 closes is 21.8 s;
  the slowest slice was slice 3 at 1,517,167 ms (line 3285).
- 00:01:55 UTC (line 5431):
  Synthetic reads `dry` (weekly 0 percent) for the first time,
  in the stretch of the chunk pass;
  see the Synthetic section in this file.
- 00:13:37 to 00:20:10 UTC (6.6 min):
  refine pass,
  checker rounds and the introduced-defect probe;
  `repair shipped: 14/118 slices changed, 341 issues` (line 12390,
  00:20:10 UTC);
  `semantic wrap: rewrapped 11 of 118 repair outcomes` (line 12389).
- 00:20:12 UTC (line 12399) to 00:34:02 UTC (13.9 min),
  the translate lane:
  118 `SLICE-START` and 118 `computed` closes
  (median 19.7 s,
  slowest slice 56 at 154,791 ms,
  line 15169),
  four in flight;
  the 30 tail slices (88 to 117) were written from 00:29:20 UTC (line 15729,
  `SLICE-START lane=translate chunk=88`) to 00:34:02 UTC.
  `JUDGE SEATS phase=translate lane short of quorum: translators 2 of 8 reachable, quorum 4 ...
  no provider has named a return the pass can wait for, so the phase runs on what is reachable` (line 12399):
  no wait.
- 00:34:06.7 UTC (lines 17099 to 17253),
  the lane contest:
  101 contested slices and no round opened;
  71 slices closed with
  `lane contest: 1 of 7 seats reachable, below the floor of 2; no round is opened and the slice keeps its incumbent`
  (`quorum-not-met`,
  `usable=0`) and 30 closed `sole-lane`;
  0.06 s of wall clock.
- 00:34:06.9 UTC to 00:55:12 UTC (21.1 min),
  the consolidation of 101 slices (lines 17281 to 21837),
  four in flight;
  383 NeuralWatt calls (69.6 Wh),
  14 Haiku,
  3 DeepSeek and 23 Synthetic calls (`windowcost.ts`,
  lines 17099 to 21837).
- 00:55:13 to 00:55:17 UTC,
  page consistency (`JUDGE SEATS` at line 21838;
  one round,
  `1/7 heard, 2904ms total, no quorum (1 of 2 needed)`,
  lines 21849 and 21850);
  assembly rewrote 36 slices and withdrew none (line 21892);
  `publish: wrote 118 slices into a page of 48105 characters` (line 21928,
  00:55:17.389 UTC).
- Idle (`idle.ts`,
  wall clock 125.8 min,
  27 merged busy spans):
  no gap of 5 s or more;
  11 gaps of 1 s or more and 14 of 0.2 s or more,
  0.4 min in all;
  no stage window has a gap of 30 s or more.
  The same method on the `XingZ6020` log found 249.0 idle minutes after 00:28:48 UTC,
  which is the positive control that the method can show idle;
  the check ran its own script on that log (bounds 2026-10-23 00:28:48 and 06:05:34 UTC)
  and found 249.04 minutes in 59 gaps of 30 s or more,
  exactly the figure the check of the `XingZ6020` read found.
  The silences of 20 s or more between timestamped lines (`gaps.ts`,
  43 of them,
  27.7 min) all end in a stream line that logs on completion,
  with that stream in flight
  (for example line 473,
  a Qwen stream of 333,644 ms).

### Where the 2 h 5 min went, against the 10 h 33 min of XingZ6020

Measured differences,
each from the scripts named
(the `XingZ6020` figures are the lead's read of that run):

- No hold waits:
  0 wait lines (`rg --count "waiting [0-9]+ms"` finds none),
  `waited=0ms` on all 161 `JUDGE SEATS` info lines.
  `XingZ6020`:
  228 waits,
  1,030.8 summed minutes,
  249.0 idle minutes.
- One attempt:
  `XingZ6020` spent its first 7 h on a deadline it hit during a hold wait and then 3 h 33 min on the second attempt.
- Tail slices written once:
  the whole translate lane,
  tail included,
  took 13.9 min here,
  the 30 tail slices from 00:29:20 to 00:34:02 UTC (4.7 min);
  in `XingZ6020` the 48 tail-slice attempts cost 87.0 Wh and about 64 min of lane time and filled nothing.
- Lane contest:
  0.06 s here (no round opened) against 73 min for 63 rounds.
- Consolidation:
  21.1 min for 101 slices against 85.4 min for 63.
- Grace after quorum (`gracesplit.ts`,
  first reader only):
  before Synthetic ran dry (22:49 to 00:02 UTC) 216 rounds,
  239 summed min,
  151 min of grace,
  40 rounds with 100 s or more in grace;
  after it 915 rounds,
  209 summed min,
  21 min of grace,
  7 rounds with 100 s or more.
  Qwen cut 34 of 196 streams (17 percent;
  `XingZ6020`:
  135 of 892,
  15 percent),
  5,605 s summed (1.56 h),
  1,345,413 reasoning characters and 350 content characters;
  29 of the 34 came before the dry reading.
  The check confirms the cut count (`abandoned=34` in the seat tally) and did not recompute the rest.
- A faster second voice:
  Haiku 5.5 answers at a median first content of 1,958 ms (p90 2,826 ms) against Qwen at 3,725 ms,
  and its longest stream was 15.5 s against Qwen's 355 s.
  The split between "a faster bench" and "a bench that lost Kimi and Qwen" is not measured:
  the grace figures are split at the dry reading,
  but the repair lane's 62 min chunk pass straddles it
  (dry from 00:01:55 UTC to its end at 00:13:32 UTC).

## Providers and models

### Meters

102 `METERS` lines (`meters.ts`),
from line 211 (22:49:34.289 UTC) to line 21775 (00:54:25.444 UTC).

- NeuralWatt,
  Anthropic,
  DeepSeek and Xiaomi `wet` on all 102.
- Hyper `dry` (`hyperBalance=0`),
  OpenRouter `dry` (`openrouterUsd=-0.10`) and Bedrock `dry` (turned off by the owner,
  line 198) on all 102.
- Synthetic:
  `wet` on 56 readings and `dry` on 46,
  and the weekly credit read as a percentage on all 102
  (23.933 percent at line 211,
  0.901 percent at line 5235,
  00:00:54 UTC).
  The transitions are in the Synthetic section in this file.
- Spend,
  first reading to last (line 21775):
  Anthropic 0.0000 to 2.3630 USD (booked by `SPEND`:
  Haiku 1.705633 plus Sonnet 0.659256 is 2.364889);
  Xiaomi 0.0000 to 0.0366 (booked 0.036556);
  DeepSeek `deepseekUsd` 9.70 to 9.63 (booked 0.147093 over 15 calls).
  The DeepSeek gap is explained in "The DeepSeek balance against the booked cost".

### Models that served

Eight (provider,
model) pairs have `SPEND` lines,
2,505 in all (`spendmodels.ts`,
`models.ts`;
the check recounted them):

- `anthropic claude-haiku-5-5`:
  610 (prompt 18,990,221,
  completion 452,438,
  cached 4,753,439,
  booked 1.705633 USD);
- `anthropic claude-sonnet-5-5`:
  18 (prompt 425,024,
  completion 2,402,
  booked 0.659256);
- `deepseek deepseek-flash`:
  15 (prompt 247,241,
  completion 85,479,
  booked 0.147093);
- `neuralwatt deepseek-v4.1-flash`:
  1,066 (prompt 18,774,278,
  completion 2,489,159,
  booked 1.433691);
- `neuralwatt glm-5.3-flash`:
  370 (prompt 5,381,386,
  completion 73,952,
  booked 0.209683);
- `synthetic hf:moonshotai/Kimi-K3`:
  208 and `synthetic hf:Qwen/Qwen3.8-27B`:
  162 (no cost field);
- `xiaomi mimo-v2.6-flash`:
  56 (prompt 910,808,
  completion 53,983,
  booked 0.036556).

The 2,505 `SPEND` lines equal the 2,505 files in `prompt-payloads/`.
A scan of the whole log for `claude-*`,
`mimo-*`,
`xiaomi/*`,
`deepseek-*` and `deepseek/*` ids finds only the ids of these eight pairs,
so no model other than these appears on the Xiaomi,
DeepSeek or Claude keys (the check's scan).

- Xiaomi served only `mimo-v2.6-flash` (56 streams;
  first content median 2,558 ms,
  p90 4,896,
  maximum 7,509 ms at line 3709;
  longest elapsed 131,306 ms,
  line 3576).
- DeepSeek's own key served only `deepseek-flash` (15 streams,
  one with no content;
  first content median 1,396 ms,
  maximum 2,152 ms,
  line 2853;
  longest elapsed 70,064 ms,
  line 12369).
  The wire id is `deepseek-flash`,
  which the check confirmed from the source
  (`provider/src/direct-client.ts:292` sends `model: servedId`;
  `:260` gets it from `route.servedIdFor`;
  `provider/src/deepseek-client.ts:164` to `:171` returns `spelling.id`;
  `provider/src/direct-reach.ts:85` to `:100` returns `entry.id`;
  that entry is `DEEPSEEK_V4_1_FLASH_ON_DEEPSEEK` with `id: 'deepseek-flash'`,
  `shared-contract/src/direct-card-sides.ts:62` to `:63`).
  The roster id stays `deepseek-v4.1-flash` (`shared-contract/src/model-cards.ts:306` to `:309`),
  and the card comment at `:340` to `:343` says the key is granted that one model.
  The first reader wrote that `deepseek-flash` is the roster's name for `deepseek-v4.1-flash-1017`;
  the check corrected that:
  nothing in the source maps to a `-1017` wire id,
  and the identity rests on DeepSeek's own documentation
  (the comment at `direct-card-sides.ts:36` to `:41`:
  "THE SERVED ID IS `deepseek-flash`,
  AND THE 1017 CLAIM RESTS ON DEEPSEEK'S DOCUMENTATION").
  The log never prints `system_fingerprint` (0 hits),
  which the client is documented to log (`direct-card-sides.ts:43` to `:44`;
  `provider/src/direct-usage.ts:134` to `:148`),
  so the run itself gives no evidence of the build behind `deepseek-flash`.
  The NeuralWatt model `deepseek-v4.1-flash` is a different key and route.
- Claude served only `claude-haiku-5-5` (610 streams;
  first content median 1,958 ms,
  p90 2,826,
  p99 4,281,
  maximum 5,595 ms at line 8184;
  longest elapsed 15,453 ms,
  line 5028) and `claude-sonnet-5-5` (18 streams,
  all in the repair lane,
  lines 1614 to 5559;
  the roster seats it only as an editor;
  first content median 1,778 ms,
  maximum 4,697 ms;
  longest elapsed 7,059 ms).
  No `-effort-low`,
  `-effort-medium` or `-effort-high` id and no other Claude model appears in any `SPEND` or stream line.
  The source seats Haiku on the owner's "Haiku wide" of 2026-10-22
  (it is also the fourth in the checker order,
  `run/src/corpus-run/run-config.ts:394`)
  and Sonnet as an editor on "Sonnet edits" (`:648`).
- NeuralWatt:
  `deepseek-v4.1-flash` 1,067 streams (1,052 with content,
  14 completed with no answer,
  1 `overrun` at line 6606);
  first content median 1,075 ms,
  p90 1,754,
  p99 3,020,
  maximum 7,421 ms (line 9048);
  longest elapsed 66,678 ms (line 19272).
  `glm-5.3-flash` (seated as `hf:zai-org/GLM-5.3-Flash`) 370 streams,
  all with content;
  first content median 1,404 ms,
  p90 2,413,
  p99 4,289,
  maximum 10,124 ms (line 19345);
  longest elapsed 34,179 ms (line 585).
  19 `schema-mismatch on neuralwatt; re-asking another provider` lines (`rg --count`).

### Seat tally (lines 21934 to 21946)

- `hf:moonshotai/Kimi-K3` asked 220,
  usable 219,
  unusable 1,
  unreachable 812.
- `hf:Qwen/Qwen3.8-27B` asked 174,
  usable 174,
  abandoned 34,
  unreachable 680.
- `deepseek-v4.1-flash` asked 1,081,
  usable 1,078,
  unusable 2,
  threw 1 (the overrun at line 6606).
- `hf:zai-org/GLM-5.3-Flash` asked 382,
  usable 382.
- `claude-haiku-5-5` asked 622,
  usable 620,
  unusable 2 (two `schema-mismatch` in coverage,
  lines 913 and 1236).
- `claude-sonnet-5-5` asked 18,
  usable 18.
- `xiaomi/mimo-v2.6-flash` asked 56,
  usable 56.

Seats that never reached a provider:
`minimax-m3` 1,054,
`gemma-4-26b-a4b-it` 974,
`google.gemma-4-e2b` 973,
`inception/mercury-2.5` 853,
`glm-5.3` 423,
`google.gemma-4-31b` 1
(the `unserved=` list of line 212;
their providers are Hyper,
OpenRouter or Bedrock,
all dry).
None of these has a stream or a `SPEND` line.
`PROMPT-REUSE` answers replayed from memory:
63 lines,
which accounts for most of the excess of the asked counts over the stream counts.

### Synthetic (Kimi-K3 and Qwen3.8-27B)

- Kimi:
  208 streams,
  all served;
  first content median 3,077 ms,
  p90 4,339,
  p99 7,007,
  maximum 8,790 ms (line 3007);
  longest elapsed 276,741 ms (line 1689);
  prompt 4,089,901,
  completion 192,267.
  One transport failure with a retry that succeeded
  (line 4184:
  `the request failed before any answer came back ... retrying in 911ms (attempt 1 of 5)`).
- Qwen:
  196 streams,
  162 served and 34 cut (`Stalled ... emitted no content`,
  reasoning only);
  first content median 3,725 ms,
  p90 4,570,
  p99 7,021,
  maximum 9,371 ms (line 3013);
  longest elapsed 355,022 ms (line 1724);
  prompt 3,318,644,
  completion 584,846 for the served streams;
  the 34 cuts have no `SPEND` line.
- By UTC hour (served/cut):
  22h Kimi 12/0,
  Qwen 11/1;
  23h Kimi 167/0,
  Qwen 136/28;
  00h Kimi 29/0,
  Qwen 15/5.
  Every one of the 404 Synthetic streams started under a `wet` reading (`drycalls.ts`);
  none started under a `dry` one.
- No refusal reply,
  no HTTP 429,
  no hold and no `refused us` line exists in the log.

### Not reached

Hyper,
OpenRouter and Bedrock:
no request,
no `SPEND` and no stream (`spendmodels.ts`).

## The Synthetic weekly credit

- The build read the weekly credit on every reading:
  102 of 102 `METERS` lines carry `syntheticWeekly=<percent>`;
  the log has no `budget unreadable`,
  no `treating as spendable` and no `violated expectations` line (`rg --count`,
  all zero).
  The `/quotas` reply is 285 or 286 characters while the credit is positive and 269 characters when it reads 0 percent
  (`rg` over the `stream quotas` lines),
  so the 269-character reply of `XingZ6020`
  (read there as the credit spent) now parses to `0%`.
- The credit fell from 23.933 percent (line 211,
  22:49:34 UTC) to 0.901 percent (line 5235,
  00:00:54 UTC),
  23.0 points in 71.3 min (0.32 points a minute),
  and read `0%` and `dry` from line 5431 (00:01:55.805 UTC).
  `syntheticWeeklyReturn=2026-10-25T00:44:35.000Z` is printed from that reading
  (the credit's named return).
- Kimi and Qwen went dark at once:
  `went dark` lines 5432 (Kimi) and 5438 (Qwen),
  and the first seat refusal naming Kimi is line 5436;
  812 Kimi and 680 Qwen seats were unreachable afterwards (`SEAT` lines).
  Seat refusals of the six seats that only Hyper,
  OpenRouter or Bedrock serve began at line 221 (22:49:35 UTC),
  so the run never had those voices;
  the roster at the start (line 212) read `wide=8 select=8 late=7 slate=7 checkers=3 translators=8 readers=6 seeing=3
  writers=9 roster=10`,
  `waited=0ms`.
- 00:44:43 UTC (line 20414):
  `synthetic=wet`,
  `syntheticWeekly=2%`,
  the return having come;
  the build used it:
  23 `SPEND` lines fall between 00:44:43 and 00:52:53 UTC,
  and every Synthetic stream that ended in that stretch started under a `wet` reading (`drycalls.ts`).
  00:50:18 UTC (line 20781):
  `0%` and `dry` again,
  and `syntheticWeeklyReturn=2026-10-25T04:06:11.000Z` from there.
  The last Synthetic `SPEND` is line 21317 (00:52:53 UTC).
  The window of 5.6 min spent the 2 points;
  four Qwen streams in it were cut after 187 to 206 s (lines 20552,
  20571,
  20650,
  20700) with no answer.
- Cost of the dry credit,
  measured:
  the benches that name Kimi and Qwen lost them from 00:01:56 UTC to the end
  (1,492 of the 5,770 unreachable seats are Kimi and Qwen);
  96.6 percent of the issue adjudications after the dry reading ended `needs-human`
  (the defects file,
  N1 and N2).
  The credit stood at 23.9 percent when the run began and read 0 percent 72.4 min later,
  while this run's Kimi and Qwen streams
  (the log shows 3,318,644 and 4,089,901 prompt tokens in `SPEND`)
  were the traffic on the key that the log can see;
  the log cannot show whether other users of the same key spent from it too.
  The log does not say how much of the credit an hour of this run's Synthetic calls costs,
  so the drain rate (0.33 points a minute over that stretch) is a rate of the meter,
  not of the run's own spend.

## NeuralWatt against the 550 Wh day

- The day anchor is line 204:
  the day starts at 19:15:08 UTC every day,
  the current day began 2026-10-24T19:15:08Z,
  so it would end at 19:15:08 UTC on 2026-10-25,
  18 h 20 min after the run ended;
  no day reset was in reach.
  The run began 3 h 34 min into the day.
- Start `neuralwattWh=550.000`,
  `neuralwattAccountWh=0.000` (line 211);
  at the last reading (line 21775) `267.881`,
  `0.324` reckoned and `280.882` on the account.
  The summed `NeuralWatt energy` lines give 283.068 Wh over 1,436 calls (`energy.ts`;
  the check recounted both):
  `deepseek-v4.1-flash` 1,066 calls,
  247.706 Wh;
  `glm-5.3-flash` 370 calls,
  35.363 Wh.
  So 283.068 of 550 Wh (51.5 percent) were drawn.
  The running sum of the lines matches the account meter to 0.053 Wh at worst (line 16796)
  and to 0.001 Wh at the last reading (`nwcheck.ts`,
  `c3.ts`);
  the 2.186 Wh of the sum that the last reading lacks came after it (00:54:25.444 UTC).
- By hour (UTC,
  Wh):
  22h 0.9 (26 calls),
  23h 40.1 (202),
  00h 242.1 (1,208).
  Remaining at selected readings:
  549.160 (line 466,
  22:54:56 UTC),
  503.410 (line 6142,
  00:04:07 UTC),
  417.535 (line 12595,
  00:20:37 UTC),
  333.962 (line 17349,
  00:34:15 UTC),
  267.881 (line 21775,
  00:54:25 UTC).
- `neuralwatt=wet` on 102 of 102 readings,
  no `markRefused neuralwatt`,
  no NeuralWatt hold,
  no NeuralWatt HTTP status.
  So the NeuralWatt hold (E2) was not exercised:
  NeuralWatt was never short and no hold named it.
- `neuralwattReckonedWh` is 0.000 until line 6847 and 0.324 on every reading after it.
  The step comes after one call:
  a NeuralWatt `deepseek-v4.1-flash` critic stream in repair slice 30 that the pass ended at its content bound
  (line 6606,
  status `overrun`,
  24,037 ms,
  443,334 raw characters,
  32,016 content characters against a bound of 32,000;
  the warn is line 6607,
  `voice lost`),
  which has no `SPEND` and no energy line.
  The check traced the mechanism in the code and judged it by design (N7 in the defects file):
  the client hands every abandoned attempt to the day's reckoning
  (`provider/src/neuralwatt-client.ts:503`),
  and 0.324 Wh is the model's estimate for a prompt of about 16,000 to 17,000 tokens
  (`0.3237 Wh estimated for 16941 prompt tokens`,
  line 1505).
  The overrun call's own prompt size is not logged,
  so the link to that call is inferred from timing and size.
- The estimate beside each measurement (`energy.ts`,
  first reader only):
  summed estimate 354.425 Wh against 283.068 measured;
  `deepseek-v4.1-flash` measured above its estimate on 225 of 1,066 calls,
  `glm-5.3-flash` on 154 of 370;
  measured over estimate has min 0.03,
  median 0.50,
  p90 2.19,
  p99 5.94 and maximum 9.75.
  In `XingZ6020` the `glm-5.3-flash` estimate was 14 times the measurement (395.852 against 28.006 Wh);
  here it is 25.416 against 35.363 Wh (0.72 times).
  The estimator of `XingZ6020` (D6 in that read) shows a smaller spread on this build;
  the change was not traced.

## The DeepSeek balance against the booked cost

The `deepseekUsd` meter fell from 9.70 (line 211) to 9.63 (line 18027),
stepping 0.01 at lines 485,
2424,
2539,
13589,
15933,
16796 and 18027,
while 15 `SPEND provider=deepseek` lines booked 0.147093 USD.
The last of those (line 21811,
00:54:30 UTC,
0.001125) came after the last reading,
so 0.145968 was booked by the last reading.
The first reader left the gap as an observation (N8).
The check found the mechanism:

- The build prices DeepSeek at peak rates always.
  `provider/src/direct-cost.ts:15` to `:20` says "DEEPSEEK IS PRICED AT PEAK,
  ALWAYS:
  the upper bound",
  and that DeepSeek's page halves every rate off peak
  (01:00 to 04:00 UTC and 06:00 to 10:00 UTC on weekdays other than Chinese public holidays are peak,
  all other hours off peak);
  `shared-contract/src/direct-card-sides.ts:51` to `:56` says "PRICED AT PEAK,
  ALWAYS" with rates of
  0.006,
  0.3 and 1.2 USD per million tokens.
- The run lay between Saturday 22:49 UTC and Sunday 00:55 UTC,
  so every call was off peak and the true rate was half.
  Half of 0.145968 is 0.0730 and half of 0.147093 is 0.0735,
  against a meter fall of 0.07 USD
  (the meter moves in cents,
  so a fall of 0.07 means a true fall between 0.06 and 0.08).
  That fits.
- So the 2.1 times gap is the documented upper-bound pricing,
  not rounding.
  The check did not fetch DeepSeek's page:
  the peak hours are the build's own comment on it.
  Whether booking at peak when the run is off peak is wanted is filed as N8 in the defects file.
