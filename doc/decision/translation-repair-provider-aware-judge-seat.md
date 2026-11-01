# Judge seats follow the provider that would serve them

Decided by the owner on 2026-09-03 ("Seat per provider reach"),
in answer to the question the
morning's drop of Qwen3.8-27B raised.
Landed in `package/translation-repair/run/src/corpus-run/run-seats.ts`.

## The evidence

- Served by Hyper,
  `hf:Qwen/Qwen3.8-27B` reasons past the 60 s round window in every judge role:
  cut in 30 of 34 translate-lane and 21 of 24 consolidation-slate select rounds on XIEPT2 with Hyper the
  only provider (2026-09-03),
  and on Carena0442 (1,648 of 1,938 calls on Hyper) in 14 of 19 and 11 of 19
  select,
  13 of 25 critic,
  15 of 19 panel,
  17 of 19 lane-contest and 7 of 14 consolidation-gate rounds.
- Served by Synthetic (Toka_ls,
  2026-09-02) it answered 25 of 28 select rounds.
- So the seat is lost to one provider's serving speed,
  not to the model.
  Dropping it outright
  (`4ad08d5dc`,
  the morning's answer under the standing authorisation to drop a model on evidence) threw
  away a judge Synthetic serves well.

## The rule

- `HYPER_SLOW_JUDGES` names the judges Hyper serves too slowly for the window:
  Qwen3.8-27B today.
- Before each phase of an entry (lanes,
  lane contest,
  consolidation;
  once per entry until 2026-09-03),
  `readJudgeSeats` reads Synthetic's meter through the routed client's quota surface
  and derives the benches:
  with Synthetic dry the Hyper-slow judges are withheld from the wide seats
  (critics,
  panel,
  both lanes' select judges) and the late seats (lane contest,
  consolidation slate,
  consolidation gate);
  with Synthetic wet they sit.
  The static benches in `run-config.ts` are the
  Synthetic-wet ones:
  seven wide seats (quorum 4) and eight late judges;
  six (quorum 3) and seven when dry.
- An unreadable meter seats the full bench:
  it is not evidence of dryness,
  and a seat asked in vain
  costs one cut where a seat withheld on a guess costs a voice.
- `HYPER_SLOW_SELECT_JUDGES` names the judges Hyper serves too slowly in the select role alone
  (Kimi-K3 since 2026-09-03):
  withheld from both lanes' slate select seats and the consolidation slate
  while Synthetic is dry,
  kept as critic,
  panel,
  lane-contest judge and gate.
  Evidence:
  cut in 0 of 69
  select rounds served by Synthetic (Toka_ls,
  2026-09-02) and in 43 of 83 and 38 of 101 with Hyper
  serving most or all of them (XIEPT2 rerun5,
  where 55 of its 61 cut streams were Hyper-served,
  and
  the postscript run),
  against 0 and 1 of 28 lane-contest rounds,
  0 of 9 critic,
  0 to 1 of 5 panel and
  0 of 14 gate rounds;
  its cut streams averaged 71 s,
  its answers 14 s.
  Same evidence bar as the owner's standing authorisation to drop a model from a role.
  The dry benches
  are then five select judges (quorum 3) beside six wide seats,
  and six slate judges beside seven late.
- Writer seats are untouched:
  translator,
  checker,
  introduced-defect probe,
  pairing and insertion
  admission keep the roster.

## What this cannot do

- The router chooses per call:
  Synthetic first until the model's per-model concurrency is taken,
  then
  Hyper.
  With Synthetic wet a burst could overflow some of this seat's calls to Hyper,
  and those may
  be cut;
  the seat follows the state that decides most calls,
  not each call.
  Measured on keyword233
  at overlap 4 (2026-09-03,
  `~/temp/agent/keyword233-seats-20260903`):
  none did,
  every Qwen call went
  to Synthetic.
- Synthetic serves the seat slowly too,
  only less often:
  on that run it was cut from 5 of 21 judge
  rounds on Synthetic (67 to 85 seconds of reasoning),
  against 30 of 34 and 21 of 24 on Hyper.
  The
  seat buys 16 answers in 21 there;
  the rule does not make it a fast judge.
- One reading per entry is too coarse for a long entry.
  XIEPT2 the same day read wet at 08:16 UTC,
  Synthetic
  ran dry at 08:19 UTC,
  and the seat sat on Hyper for three and a half hours:
  abandoned in 102 judge calls,
  75 rounds at the full 60 s grace,
  consolidation 134 minutes for 28 slices
  (`~/temp/agent/xiept2-postscript-20260903`).
  The meter is now read again before the lane contest and
  before consolidation,
  so a seat given at the lanes is withdrawn once Synthetic is dry.
- The Synthetic-wet path ran live the same day,
  at 1.5% of the weekly meter:
  `JUDGE SEATS
  synthetic=wet wide=7 late=8 hyper-slow seated=yes`,
  the rounds heard 7 of 7 thirteen times and 8 of
  8 six times.

## Rejected alternatives

- Leave the seat dropped:
  simplest,
  loses the judge in Synthetic weeks.
- Also drop Kimi-K3 under the same rule for every judge role:
  it is cut in a majority of select rounds
  on Hyper but in 0 or 1 of 28 lane-contest rounds and almost never as critic,
  panel or gate,
  so the
  whole-bench rule would throw away seats Hyper serves well.
  Superseded on 2026-09-03 by the select-only
  rule (`HYPER_SLOW_SELECT_JUDGES`),
  once the per-role counts were measured.
- A longer window for Hyper-served seats:
  reverses the owner's 2026-09-02 choice of the seat over the
  window,
  and under a 1,000-an-hour window the request,
  not the second,
  is scarce.

## Three providers, 2026-09-03

OpenRouter joined as the third provider (`doc/decision/translation-repair-openrouter-fallback.md`),
so "Synthetic dry" stopped being the whole question:
with Hyper dry too,
a shared seat's calls go to
OpenRouter,
where Hyper's serving speed is not the reason to withhold anything.
`readJudgeSeats` therefore reads the router's own dryness view over every provider and asks,
per seat,
which provider would take its calls (`providerServing`,
the first in `PROVIDER_ORDER` that serves the
model and reads wet).
The Hyper-slow rules in "The rule" apply where that provider is Hyper;
the owner's cost decision on Kimi-K3
applies where it is OpenRouter,
with gemma seated as the substitute checker.
That cost decision reaches the translator and picture-reader seats too (`8848f070e`):
the first OpenRouter-only pass (keyword233,
2026-09-03 18:15 UTC) had every bench without Kimi-K3
and still bought six of its translations,
because the writer roster and the reader roster were the static
lists,
so `JudgeSeats` now carries `translators` and `readers` filtered by the same rule
and the picture stage reads its own seats before the lanes read theirs.
Qwen3.8-27B is seated when OpenRouter would serve it:
its chat-completions median there sat in the band
of the other models on the probe,
unlike its Hyper serving,
and a pass with Synthetic dry is where
that is checked on corpus-sized prompts.
The first pass with OpenRouter in the order (keyword233,
2026-09-03 16:38 UTC) did not check it:
Synthetic stayed wet and served every Qwen call.
The OpenRouter-only pass did:
31 of 31 completed answers usable,
completed latencies matching its
Synthetic ones (p50 18.3 s against 16.5 s,
p90 52.8 s against 48.8 s),
and 7 of 38 asks cut at the
60 second straggler grace while still reasoning,
which is the grace's doing rather than the endpoint's
and is put to the owner in `translation-repair-openrouter-fallback.md`.

## Addendum 2026-09-09: the reach honours the Kimi-K3 withholding

The seat reader honoured the cost decision at phase start and nothing honoured it afterwards.
On the third `noname` pass of 2026-09-09 (frozen `33a023445`) Synthetic dried at 17:25 UTC,
inside the translate lane,
and the router sent the seated Kimi-K3's calls to OpenRouter as the next wet provider serving it:
22 calls,
1.14 USD at 3 and 15 USD per million,
61 percent of the 1.87 USD the pass spent there,
two of them abandoned streams at 0.28 and 0.08 USD.
Since `19b6043a3` the withheld set lives in `openrouter-catalog.ts` (`OPENROUTER_WITHHELD`),
`reachOf` and the picture reach say OpenRouter does not serve a withheld model,
so the router refuses the call as `NoProviderForModelError` and the seat counts as unreachable
(the select minimum of the same day sizes for it),
and the seat reader withholds the seat whose only wet provider would have been OpenRouter,
which is the same seat as before and keeps the substitute checker sitting where it did.
The catalog still lists the model,
so its spelling and prices stay known;
what changed is that the run never buys it there.

## Where the evidence lives

- `doc/planning/translation-repair-roster-calibration-2026-09-01.md`:
  the cut counts per seat and
  round kind,
  split by lane.
- `doc/decision/translation-repair-roster-seating-2026-09-01.md`:
  the 2026-09-02 and 2026-09-03
  addenda this supersedes in part.
