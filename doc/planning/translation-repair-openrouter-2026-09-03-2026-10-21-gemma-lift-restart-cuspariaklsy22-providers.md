# OpenRouter and the corpus passes: the providers of CuspariaKLSY22 on build 2a336603bf (2026-10-21)

What was read on 2026-10-21 of the providers behind the run of the second pass after the Gemma readers were lifted:
the NeuralWatt meter and the plan energy,
the seats,
the unreachable and abandoned seats,
the stalled `inception/mercury-2.5` stream,
the quorums,
and the archive blocks.
Entry:
CuspariaKLSY22 (entry `CuspariaKLSY`).
The log,
the rulings,
the classes named,
the open items and the fixes decided are in
[the run of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-run.md),
and the shipped page,
line by line,
is in
[the page of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-page.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of the restart share are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).

## CuspariaKLSY22 provider read, 2026-10-21, 23:41 UTC: the plan energy rose 37.3 Wh, Bedrock was asked once, mercury stalled

The provider read is part of the read of `CuspariaKLSY22` (SETTLED in 39.3 min),
made at 23:41 UTC on 2026-10-21 on tip `2a336603bf38a6a53aeb4f3e70f0a3447acf5487`.
Every count names its method:
a script that counts log lines matching a pattern,
lists the round lines,
sums the `SPEND` lines or sums the NeuralWatt energy,
run over the pass log.

### NeuralWatt, stop lines and energy

NeuralWatt's free-energy day started at 19:15:08 UTC on 2026-10-21
(`decideNeuralWattDayAnchor`,
line 204:
`the current day began 2026-10-21T19:15:08.000Z`).
The run started 3 h 10 min into that day,
at 22:25:13 UTC.
The first `METERS` line (22:25:27.683 UTC) read `neuralwattWh=509.090` left of 550
and `neuralwattAccountWh=40.910` used on the whole account since the day began.

- Baseline:
  taken once at 22:25:27.032 UTC,
  `NeuralWatt quota at the start of the run: 3.712 kWh charged this period`
  (line 203,
  the plain wording;
  no `could not be read` line,
  no late-start line).
- Stop lines:
  none.
  Every `METERS` line (27 of them,
  22:25:27.683 UTC to 23:02:16.203 UTC) says `neuralwatt=wet` and `neuralwattKeyAllowance=none`;
  no stop fired.
- Use:
  74 calls (`deepseek-v4.1-flash` 55,
  `glm-5.3-flash` 19),
  74 `provider=neuralwatt` `SPEND` lines,
  21.1113 Wh measured against 10.8604 Wh estimated
  (`CuspariaKLSY21`:
  85 calls,
  21.0145 Wh against 10.5074).
  `deepseek-v4.1-flash` 9.1800 Wh (estimate 4.1165),
  `glm-5.3-flash` 11.9313 Wh (estimate 6.7439).
  Measured stays near twice the estimate in all three of the last runs (1.94 now).
  One call on repair slice 2 at 22:28:29.109 UTC measured 0.9760 Wh against 0.0758 Wh estimated (line 277).
- Meter and calls agree:
  `neuralwattWh` fell from 509.090 to 487.978,
  which is 21.112 Wh,
  and `neuralwattAccountWh` rose from 40.910 to 62.022,
  the same 21.112 Wh;
  no call settled after the last `METERS` line,
  and the call sum is 21.1113 Wh.
  The two figures sum to 550.000 on most lines and differ on some
  (549.848 at 22:56:03.446 UTC,
  549.255 at 22:49:53.908 UTC):
  multiples of the in-flight holds,
  0.0758 and 0.3725 Wh,
  the explanation
  [the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md)
  gives for its Finding 9
  (a guess here as there).
- The account meter is consistent with this pipeline alone:
  the first `neuralwattAccountWh`,
  40.910 Wh,
  equals the sum of the settled calls of `zheermao22`
  (a script over the `zheermao22` log:
  40.910 Wh,
  last call 22:24:33 UTC),
  so no other user of the covered models touched the account between 19:15 UTC and this run.
- The closing line (after `DONE`,
  at the quota read of 23:04:42.719 UTC) reads
  `NeuralWatt PLAN ENERGY SPENT: 37.3 Wh of the owner's plan were charged between the start and the end of the run;
  anything else done on the account in that time is in this figure too`.
  This is the first nonzero figure in the series of this entry's runs
  (`CuspariaKLSY20`,
  `CuspariaKLSY21` and `zheermao22` all print `no plan energy was spent`).

The contradiction,
as three numbers:

- the plan figure moved 37.3 Wh while the free energy lasted:
  487.978 Wh were left at the last reading;
- the run's own settled calls are 21.1113 Wh,
  so 16.2 Wh of the 37.3 cannot be this run's covered-model use even if every call had been charged to the plan;
- the baseline was 3.712 kWh at 22:25:27 UTC and was also 3.712 kWh at the start of `zheermao22`
  (21:05:45.428 UTC,
  its `takeBaseline` line),
  so nothing moved the plan across the 79 min and 40.910 Wh of free-energy use of `zheermao22`,
  and whatever moved it began after 22:25:27 UTC.
  For scale,
  the baseline was 3.1687 kWh at 06:04:40 UTC (`CuspariaKLSY21`),
  so the plan figure rose 543.3 Wh between the two baselines:
  something other than the free-energy pipeline draws on the plan at hundreds of Wh a day.

`package/translation-repair/provider/src/neuralwatt-quota.ts:9` to `26` says this figure cannot separate the owner's
other account use ("anything else the owner does on the account moves `kwh_used` too"),
and `provider/src/neuralwatt-usage.ts:21` records a non-covered model of the account
(`mimo-v2.6-pro`,
3.83 kWh on one day) that must not count against the free energy.
The owner also uses the models of this key outside the pipeline.
The run's own settled calls came to 21.11 Wh,
and the other 16.2 Wh of the 37.3 Wh cannot be attributed from the log:
the log reads the quota only twice,
and the by-model route is read as one covered-model total.
Where that 16.2 Wh came from is unverified,
and it cannot be settled without the owner's dashboard for the window 22:25 UTC to 23:05 UTC.
This is an open item for the owner
(the NeuralWatt dashboard for that window,
by model),
not a code defect.

Other meters,
from the same 27 `METERS` lines:

- Synthetic `syntheticWeekly=0%` and `synthetic=dry` throughout.
- Hyper `hyperBalance` 204 at 22:25:27 UTC,
  167 at 23:02:16 UTC
  (37 hypercredits over about 190 calls;
  in `CuspariaKLSY21` Hyper read `0` and was dry).
- Bedrock `bedrockUsd=5.02` with `bedrockReckonedUsd=1.32` and `bedrock=wet` on every `METERS` line,
  while the `JUDGE SEATS` lines read `bedrock=dry` from the second line on.
  The read finds this by design in `run-seats.ts`:
  it reads the router's answer with holds folded in,
  and the `METERS` line prints the account's meter without them
  (the 401 hold is a different fact from the balance).
  A reader of the meters alone is nonetheless misled while the credential is refused.
- OpenRouter `openrouterUsd` 13.47 to 13.42;
  the 109 OpenRouter `SPEND` lines sum to 0.07300 USD
  (`CuspariaKLSY21`:
  232 lines,
  0.1728 USD),
  of which `inception/mercury-2.5` is 0.05155 USD over 69 lines.
  The 33 abandoned mercury calls carry an estimate (`estimated=abandoned`) of 0.02907 USD for replies that were empty.
- The NeuralWatt `SPEND` lines carry a nominal 0.1147 USD that the read does not count
  (`neuralwatt-meter.ts` says it is nominal on free-energy requests).

### Seats, unreachable seats and quorums

The ten `SEAT` lines after `DONE` read
(usable out of asked;
the first is the unreachable seat):

- `google.gemma-4-e2b` asked 0,
  `unreachable=45`.
- `deepseek-v4.1-flash` 55 of 55.
- `glm-5.3` 30 of 31 (1 unusable).
- `hf:zai-org/GLM-5.3-Flash` 18 of 19 (1 unusable).
- `hf:moonshotai/Kimi-K3` 43 of 45 (`threw=2`).
- `inception/mercury-2.5` 36 of 36,
  `abandoned=33`.
- `gemma-4-26b-a4b-it` 55 of 55.
- `minimax-m3` 58 of 58.
- `typesafe/jev-1.13` 24 of 24.
- `xiaomi/mimo-v2.6-flash` 10 of 10.

Totals:
333 asks,
329 usable,
2 unusable,
2 threw,
33 abandoned,
45 unreachable
(`CuspariaKLSY21`:
312,
311,
1,
0,
0 and 143).
Checks on the counts:

- The 45 equals the 45 warn lines that carry `seat unreachable`,
  all for `google.gemma-4-e2b` after the credential hold.
- `asked=36` for mercury excludes the 33 abandoned calls on purpose:
  `seat-tally.ts:120` to `122` and `346` to `352` say a seat its callers cut off after quorum is not counted as asked
  and cannot be withheld for it.
  The log shows 69 mercury streams (36 `completed`,
  33 `cut`),
  and 69 mercury `SPEND` lines.
- 373 `SPEND` lines in all (hyper 190,
  openrouter 109,
  neuralwatt 74)
  against 333 asks plus 33 abandoned,
  which leaves 7.
  The log's `reaskElsewhereNudged` lines show 9 second calls sent to another provider
  (OpenRouter for `gemma-4-26b-a4b-it` 3 and `minimax-m3` 3,
  Hyper for `deepseek-v4.1-flash` 2 and `hf:zai-org/GLM-5.3-Flash` 1),
  and the `SPEND` lines by provider and model show the same nine
  (OpenRouter `gemma` 3 and `minimax` 3,
  Hyper `deepseek-v4.1-flash-1017` 2 and `glm-5.3-flash` 1).
  The read of `CuspariaKLSY21` balanced 312 asks and 5 re-asks against 317 `SPEND` lines;
  here 333,
  9 and 33 make 375 against 373,
  so two `SPEND` lines are not accounted for
  (unreconciled;
  a reply that ended without a usage record is a guess).

Seat by seat,
what is new:

- Hyper wet seated `glm-5.3` (`CuspariaKLSY21`:
  unreachable 24 times) and served `hf:moonshotai/Kimi-K3` (43 usable calls),
  and moved `minimax-m3` and `gemma-4-26b-a4b-it` off OpenRouter.
  `hf:Qwen/Qwen3.8-27B` is no longer unserved but withheld (`HYPER_SLOW_JUDGES`,
  `run-seats.ts:77`),
  so it has no `SEAT` line at all.
  `hf:moonshotai/Kimi-K3` is named in `withheld=hf:Qwen/Qwen3.8-27B,hf:moonshotai/Kimi-K3`
  on all six `JUDGE SEATS` lines (210,
  239,
  240,
  1407,
  1659,
  1748)
  and still asked 45 times:
  `withheld=` names a model that lost at least one seat (`run-seats.ts:386`),
  and Kimi-K3 lost its select seats
  (`HYPER_SLOW_SELECT_JUDGES`,
  line 94;
  `select=6` and `slate=6` on every line,
  against `late=7`)
  but is seated elsewhere
  (it voted in the gates and critic rounds,
  and the translate lane's writer line at 1408 names it),
  which fits Finding 10 of the causes document.
- `google.gemma-4-e2b` is the only dark seat.
  `SEATS DARK` did not print,
  for the reason the read of `CuspariaKLSY21` found:
  no seat was asked and dark.
  `google.gemma-4-31b` is named `unserved=` and has no `SEAT` line.
- Bedrock's credential refused again on the first call:
  `[chatText] <- google.gemma-4-e2b: HTTP 401` and
  `SEAT CREDENTIAL REFUSED google.gemma-4-e2b on bedrock`
  (22:25:28.979 UTC and 22:25:28.980 UTC,
  lines 214 to 215),
  `The security token included in the request has expired`,
  and `bedrock is held out for every model for the rest of the run`.
  This is the third run in a row (`CuspariaKLSY20`,
  `CuspariaKLSY21`,
  `CuspariaKLSY22`) that meets the expired token,
  and the owner ruled on 2026-10-20 to leave Bedrock off rather than issue a new key.
  `JUDGE SEATS` reads `bedrock=wet` and `unserved=none` once (line 210),
  then `bedrock=dry` and `unserved=google.gemma-4-e2b,google.gemma-4-31b` on the five later lines
  (239,
  240,
  1407,
  1659,
  1748).
- `chunk re-seated under a hold` printed 0 times (`CuspariaKLSY21`:
  0).
  The translate lane's own `re-seated before starting` line (1408,
  22:48:59.619 UTC) lists `google.gemma-4-e2b` among the seven writers although its credential had been refused,
  as in `CuspariaKLSY21`.
  Checker rounds closed at `2/3 heard` where mercury was abandoned
  (for example line 1353,
  22:48:52.140 UTC).
  The read did not test whether the `chunk re-seated` line should have printed.

The stalled seat,
in full:

- `inception/mercury-2.5` (OpenRouter,
  endpoint `Inception`) answered 36 asks in 4 to 22 s (median 6.1 s)
  and left 33 streams with `firstByte` near 800 ms,
  7300 to 10650 raw characters,
  `0 content chars`,
  `0 reasoning chars` and `opening ""` until the round's grace closed them (123 to 179 s):
  for example line 228 (the block pairing round,
  125589 ms) and line 2293 (the last round,
  145727 ms).
  The raw characters carry no model content (the stream lines say `0 content chars`);
  that they are keepalive frames is a guess,
  since the raw count grows with the wait (7300 at 123 s,
  10650 at 179 s).
- The cut share rises with the clock,
  and it is a provider window and not this build.
  By half hour of UTC,
  the log of `zheermao22` (frozen build,
  same tip) reads
  ok 17 and cut 0 (21:00 to 21:29 UTC),
  ok 46 and cut 1 (21:30 to 21:59 UTC,
  the first cut at 21:59:56 UTC),
  ok 45 and cut 10 (22:00 to 22:29 UTC);
  this run reads
  ok 4 and cut 1 (22:25 to 22:29 UTC),
  ok 31 and cut 28 (22:30 to 22:59 UTC)
  and ok 1 and cut 4 (23:00 to 23:04 UTC);
  the `XingZ6017` log so far reads ok 2 and cut 2.
  `CuspariaKLSY21` had 0 cuts at 06:00 UTC on tip `3637592bd4`.
  `zheermao22` ran this same tip and had no cut in its first 17 streams (21:00 to 21:29 UTC)
  and one in the next 47,
  so the build alone does not explain the stall;
  a provider window from about 22:00 UTC is the read's reading.
- The cost to the run is the grace:
  `stage-round.ts:471` to `474` abandons after `graceMs` and prints `abandoned ... after quorum`,
  the seat is recorded as not answered,
  and nothing remembers it for the next round,
  so each round of the repair lane,
  the lane contest and the consolidation waited 120 s again whenever mercury drew a stalled stream
  (see the classes named in
  [the run of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-run.md)).

Other seat events:

- `hf:moonshotai/Kimi-K3` threw twice:
  `kimi-k3: ended a call that exceeded its content bound, 32008 characters on the content channel against a bound of
  32000, voice lost`
  (repair slice 2,
  line 442,
  22:29:43.192 UTC;
  repair slice 1,
  line 477,
  22:29:52.622 UTC;
  both critic rounds).
  The bound is `CONTENT_OVERRUN_CAP = 32_000` (`stream-runaway-watch.ts:85`).
  The text `exceeded its content bound` is the rendering of `StreamOverrunError`,
  whose name the log never prints.
- `glm-5.3` (Hyper only) leaked its native tool markup into a lane contest ballot:
  `raw="{\"archive<arg_key>flawed</arg_value><arg_key>choice\": \"translate\", ...` (line 1693,
  22:52:22.270 UTC),
  and `reaskElsewhereNudged` answered
  `no re-ask elsewhere (no provider other than hyper serves it with budget)` (line 1692);
  that contest slice (slice 1) closed on 4 of 6 usable voices.
  One event;
  no sign it recurs.
- `hf:zai-org/GLM-5.3-Flash` as editor of repair slice 3:
  both NeuralWatt and Hyper cut the reply
  (`finish_reason=length`,
  then `finish_reason=max_tokens`,
  raw `""`),
  voice lost,
  a recovery round ran for the one unreadable answer
  (lines 751 and 753,
  22:32:42 UTC).
- `panel ballot from gemma-4-26b-a4b-it irregular`:
  6 lines (lines 513 to 516,
  549,
  550),
  all in repair slices 2 and 0:
  `unknown-regrade-severity` with the value `"minor"` three times,
  with `nil` twice,
  and `group-index-out-of-range (2)` once
  (`CuspariaKLSY21`:
  7 lines of `group-index-out-of-range`).
- `schema-mismatch`:
  11 info lines on the re-ask path
  (`gemma-4-26b-a4b-it` on Hyper 3,
  `minimax-m3` on Hyper 3,
  `deepseek-v4.1-flash` on NeuralWatt 2,
  `hf:zai-org/GLM-5.3-Flash` on NeuralWatt 1 and on Hyper 1,
  `glm-5.3` on Hyper 1)
  plus 2 warns (lines 751 and 1693),
  13 in all.

Quorums:
no stage threw (0 `[error]` lines,
0 `quorum-not-met`),
and the entry settled.
Many rounds closed short of their full bench with the quorum met,
most of them at the end of a 120 s grace on a stalled mercury stream
(the unreachable `google.gemma-4-e2b` seat accounts for one missing voice in every round it sits in):
translate slice 3 `select round: 3/5 heard` (line 1605),
consolidation slate select slices 2 and 3 `3/5` (lines 2115 and 2042),
lane contest `4/6` on slices 0 and 1 (lines 1732 and 1707),
`4/5` on slice 2 (line 1723) and `5/6` on slice 3 (line 1698),
and absolute naturalness review `4/6` on slice 0 (line 2286) and `4/5` on slice 3 (lines 2266 and 2292).

### Archive blocks

The entry has none.
The `preparation.blockPairing` of the artifact pairs 18 of 18 blocks
(`paired 18 of 18 original and 18 of 18 translation blocks across 18 relations, from 4 usable voices of 4 heard`,
line 229,
22:27:33.922 UTC),
so no archive block is unclaimed and the archive block review,
the composite candidate and the whitespace-free quote anchoring are not exercised.
The log has no `archive block` line (0 matches).
