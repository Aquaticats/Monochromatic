# Roster seating of 2026-09-01: addenda of 2026-09-02 to 2026-09-09

The addenda dated 2026-09-02 to 2026-09-09,
each recording a seat a model took or left and the evidence for it.

Part of [Roster seating of 2026-09-01, on the two calibrations under the owner blocklist](translation-repair-roster-seating-2026-09-01.md),
which lists every file of this record.

Superseded in part on 2026-10-25 by
`package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md`:
the `minBallotWeight` of 3 of 6 named in the addendum of 2026-09-03 no longer exists,
and two answering judges decide whatever their weight.

## Addendum 2026-09-02: GLM-5.3-Flash leaves every judge seat

Decided by the owner ("Unseat GLM-5.3-Flash as a judge,
keep it as editor") on the Toka_ls relaunch's
measurement,
landed as commit `9a7d48354` in `corpus-run/run-config.ts`.

-   Evidence:
    under the 60 s round window the model's reasoning streams (up to a million raw characters)
    were cut in 12 of 13 panel rounds,
    12 of 21 translate-select rounds,
    11 of 29 repair-select rounds,
    11 of 15 critic rounds and 5 of 15 contest rounds,
    51 of the run's 78 cuts.
    No round lost its decision
    without it;
    two translate-select rounds declined and were decided in a challenge round.
    Record:
    `doc/planning/translation-repair-roster-calibration-2026-09-01.md`,
    "The Toka_ls relaunch was killed at
    77 minutes,
    in consolidation".
-   Seats it leaves:
    critic,
    adjudication panel and both lanes' select judges (`WIDE_SEAT_DROPPED`,
    now
    seven seats,
    quorum 4),
    and the lane contest,
    the consolidation slate's judges and the consolidation
    gate (`LATE_JUDGE_DROPPED` behind `RUN_LATE_JUDGES`,
    eight seats;
    those rounds seat the whole roster
    including `glm-5.3`,
    which this addendum does not touch).
-   Seats it keeps:
    first editor (top three in every one of 4000 resamples),
    refiner,
    translator,
    and
    consolidation writer;
    pairing and insertion-admission rounds,
    which are not judgments of text and lost
    no voice to the window (27 of 27 and 9 of 9 heard),
    keep the roster.
-   The 60 s round window stays;
    the choice was between the window,
    the seat and the wall clock,
    and the
    owner chose the seat.

## Addendum 2026-09-03: Qwen3.8-27B leaves every judge seat

SUPERSEDED THE SAME DAY:
the owner chose to seat the judge per provider rather than drop it
(`doc/decision/translation-repair-provider-aware-judge-seat.md`,
`corpus-run/run-seats.ts`).
The seat is
withheld while Synthetic is dry and sits while Synthetic serves it;
the evidence bullets of this addendum stand as the
record of why.

Dropped under the owner's standing authorisation ("I authorize you to drop any model from any role,
as long as
you have evidence"),
landed in `corpus-run/run-config.ts` (`WIDE_SEAT_DROPPED` and `LATE_JUDGE_DROPPED`)
on 2026-09-03 while XIEPT2's fifth run was in flight;
it reaches the next launch.

-   Evidence,
    Hyper the only provider (XIEPT2 rerun5,
    whole run):
    cut in 30 of 34 translate-lane select
    rounds and 21 of 24 consolidation-slate select rounds (both rounds log as `judgeTranslateSlate`;
    the
    lane tag tells them apart);
    in the first 25 minutes it was the only late seat in 7 of 16 select
    rounds,
    so those rounds waited the whole 60 s window for a ballot that never came;
    the cut streams
    carried 700,000 to 1,200,000 raw characters of reasoning and no answer.
-   Evidence,
    mostly Hyper (Carena0442's landed pass,
    1,648 of 1,938 calls on Hyper):
    14 of 19
    translate-lane select,
    11 of 19 consolidation-slate select,
    21 of 59 repair-select,
    13 of 25 critic,
    15 of 19 panel,
    17 of 19 lane-contest and 7 of 14 consolidation-gate rounds;
    113 abandoned rounds
    against 106 successful calls.
-   Counter-evidence,
    served by Synthetic (Toka_ls,
    2026-09-02):
    answered 25 of 28 select rounds.
    The seat is
    lost to Hyper's serving speed for this model,
    not to the model;
    a provider-aware seat that keeps Qwen
    as a judge while Synthetic serves it is the open design question,
    not decided here.
-   Seats it leaves:
    critic,
    adjudication panel,
    both lanes' select judges (`WIDE_SEAT_DROPPED`,
    now six
    seats,
    quorum 3,
    `minBallotWeight` 3 of 6),
    the lane contest,
    the consolidation slate's judges and
    the consolidation gate (`LATE_JUDGE_DROPPED`,
    seven seats).
-   Seats it keeps:
    translator (2 cuts in 22 rounds),
    checker,
    introduced-defect probe,
    pairing and
    insertion admission (1 cut each in 8 to 26 rounds).
-   Kimi-K3 is recorded as the next candidate,
    not dropped:
    on rerun5 17 of 34 translate-lane select and
    18 of 24 consolidation-slate select rounds;
    on Carena0442 11 of 19 and 10 of 19 select,
    10 of 19
    contest,
    9 of 25 critic,
    5 of 19 panel.
    Above the precedent in the select and contest seats,
    under
    half in critic and panel;
    the seat sets are coarser than that profile,
    which is part of the
    provider-aware seat question.
-   The 60 s round window stays,
    as on 2026-09-02.

## Addendum 2026-09-07: `google.gemma-4-e2b` joins the judge seats; `google.gemma-4-31b` stays out

Taken on the same delegation as this record's 2026-09-01 seating,
under a rule pre-registered in `doc/planning/translation-repair-openrouter-2026-09-03.md`
("Measuring the two Bedrock-only sizes",
"The rule,
written before the next questions arrive")
before the seated roster was measured on the same questions.
Instrument:
`judge-fidelity-probe` over the three settled slices the four artifacts on disk yield,
twelve distinct questions per judge,
the candidates alone at Bedrock and then the seated roster,
build `3fc918013`.

-   `google.gemma-4-e2b`:
    11 of 12 with one damaged pick,
    against the six seated judges' median of 9.5 and worst damaged count of 1.
    Seated in critic,
    adjudication panel and judge in both lanes,
    the late bench and the slate (`b0b48d6f4`,
    `SEATED_BEDROCK_JUDGES` in `run-config.ts`).
    No writing seat:
    no producer calibration has measured it,
    and `RUN_WRITERS` keeps the consolidation writers to the measured roster.
-   `google.gemma-4-31b`:
    6 of 12,
    declining the rest,
    never the damaged text.
    Not seated.

Twelve questions over three slices separate a habit of declining from a habit of choosing the damaged text;
the 40-slice calibration that seated the writers was not repeated for a judge seat,
which the 2026-09-01 decision did not measure either.
The translator seat is `producer-calibrate`'s to give by the pooled null,
run with `--candidates` when the meters allow.

## Addendum 2026-09-08: `gemma-4-26b-a4b-it` and `google.gemma-4-31b` join the picture readers through Bedrock

Decided by the reader-seat rule recorded in
[`translation-repair-openrouter-2026-09-03.md`](../planning/translation-repair-openrouter-2026-09-03.md)
("The three Gemma sizes against the seated readers on nine pictures"),
written after the first run's numbers and before any others.
Instrument:
every picture whose seated readers left a `corroborated` record in a runs dir's picture cache,
nine pictures over five entries,
each candidate asked through the production Bedrock client with the reader stage's own prompt,
each reading scored by the trigram overlap and `readingsCorroborate` verdict `readImagePair` uses
against every seated reading of that picture,
the seated readers scored the same way against each other as the band,
build `c4a9682fe`.

-   `gemma-4-26b-a4b-it` (Bedrock `google.gemma-4-26b-a4b`):
    eight readings produced of nine asked,
    every one corroborated by every seated reader,
    mean overlap 0.668,
    against a seated band of 0.80 to 0.92 with every seated reader corroborated on every picture.
    Reads pictures through Bedrock (`readsImages: true` on its Bedrock card,
    `f7f9c9136`);
    its OpenRouter row is unchanged.
-   `google.gemma-4-31b`:
    eight of nine,
    every one corroborated by every seated reader,
    mean overlap 0.671.
    Reads pictures through Bedrock;
    holds no judge or writing seat (the 2026-09-07 addendum stands).
-   `google.gemma-4-e2b`:
    eight of nine,
    two corroborated by no seated reader
    (an invented reading and a drifted one),
    every reading opening with the prompt's `READING RESPONSIBILITY:` label.
    Not a reader.

The readers bench is six,
a Bedrock-only pass has two reachable readers,
and the readers' quorum in the phase shortfall reading is a pair,
since `readImagePair` corroborates from two readings.
On record beside the seat:
both seated sizes transcribe less exactly than the seated four
(0.67 against 0.80 to 0.92),
a 1.27 MB picture came back as an empty stream from every Gemma size on Bedrock,
and every cached picture reading is re-read once,
since the cache key names the reader roster.

## Addendum 2026-09-08: `google.gemma-4-e2b` takes the translator seat and the consolidation seat

Decided by the pooled-null rule the editor and translator seats of this decision were read by,
run as `producer-calibrate 40 --candidates google.gemma-4-e2b` on `5462257b4`
(launched 13:07 UTC,
standing printed 16:07 UTC,
10795 s wall clock,
one slice at a time,
every one of the ten models writing and judging every slice),
the same instrument that dropped two writers on 2026-09-01,
with the Bonferroni threshold at ten comparisons (z 2.81) rather than nine.
Log:
`~/temp/agent/producer-calibrate-e2b-20260908.log`.
The reader (the scratch `read-standing.mjs`) reproduces the 2026-09-01 record
(pooled null 13.0 percent,
z -4.53 and -4.55) before reading this one.

Standing as printed,
best first,
with the availability-adjusted share where it differs and the z against the pooled null of 12.8 percent
(308 disinterested wins over 2413 ballots):

-   `hf:Qwen/Qwen3.8-27B`:
    24.4 percent (55 of 225 disinterested ballots,
    over 35 candidates),
    adjusted 21.4,
    z +5.25
-   `gemma-4-26b-a4b-it`:
    21.3 (56 of 263,
    over 40),
    adjusted 21.3,
    z +4.14
-   `hf:moonshotai/Kimi-K3`:
    16.2 (42 of 260,
    over 40),
    z +1.64
-   `minimax-m3`:
    11.6 (30 of 258,
    over 40),
    z -0.55
-   `hf:zai-org/GLM-5.3-Flash`:
    11.1 (24 of 217,
    over 35),
    adjusted 9.7,
    z -0.75
-   `glm-5.3`:
    10.7 (17 of 159,
    over 25),
    adjusted 6.7,
    z -0.78
-   `google.gemma-4-e2b`:
    10.1 (30 of 298,
    over 39),
    adjusted 9.8,
    z -1.40
-   `deepseek-v4-pro-0813`:
    7.8 (19 of 243,
    over 39),
    adjusted 7.6,
    z -2.31
-   `deepseek-v4-flash-0731`:
    7.6 (18 of 236,
    over 40),
    z -2.36
-   `hf:openai/gpt-oss-120b`:
    6.7 (17 of 254,
    over 40),
    z -2.90

The candidate is not separated from the null (z -1.40 against a threshold of 2.81),
wrote 39 of 40 candidates,
threw none of 94 asks (`SEAT google.gemma-4-e2b asked=94 usable=94 unusable=0 threw=0`),
and its 94 completed streams ran p50 1.6 s,
p90 2.5 s,
max 5.1 s,
the fastest voice on the bench;
no warning in the log names it.
Slice-clustered (37 winner-bearing rounds over 37 slices,
top-three inclusion over 4000 resamples of whole slices):
Qwen3.8-27B 91.7 percent,
gemma-4-26b-a4b-it 83.8,
Kimi-K3 83.0,
E2B 12.9,
GLM-5.3-Flash 12.2,
the rest under 8.

Seated as `169a86173`:
`google.gemma-4-e2b` leaves `WRITER_UNMEASURED` in `run-config.ts`,
which is now empty,
and so joins `RUN_TRANSLATORS` (eight,
quorum 4) and `RUN_WRITERS` (ten),
the consolidation seat going with the translator seat as it did for every measured writer on 2026-09-01,
the two dropped translators included.
`RUN_WRITERS` now filters off `RUN_ROSTER` rather than off the catalog,
so a Bedrock-only size the roster seats writes once measured and never before.

What the same standing says about the seated writers,
recorded and not acted on:
`hf:openai/gpt-oss-120b` sits below the null again (z -2.90 against 2.81),
`deepseek-v4-flash-0731` does not this time (z -2.36),
and `deepseek-v4-pro-0813` is at z -2.31;
the 2026-09-01 drops stand on their own measurement and nothing here reseats or drops anyone else.
`glm-5.3` threw 41 of 87 asks to the calibration's windows
(cut mid-reply after 1.6 to 3.2 million delivered characters),
Qwen3.8-27B 13 of 84,
GLM-5.3-Flash 14 of 87,
Kimi-K3 4 of 88,
deepseek-v4-pro-0813 3 of 88;
eight OpenRouter streams carried an in-stream 504 from Together,
Reka or Wafer after the gateway's success status,
each retried once and answered.

A consequence for the Bedrock-alone shape:
the translators bench a Bedrock-only pass reaches is now a pair
(`gemma-4-26b-a4b-it` and `google.gemma-4-e2b`),
the floor the fifteenth class asks for,
while still short of the quorum of 4 of 8;
the editors and refiners benches stay at zero there,
so a Bedrock-only pass still stops at the lanes on those two,
and the fixtures in `run-seats-floor.unit.test.ts` and `run-seats-read.unit.test.ts` now say so.

## Addendum 2026-09-09: `inception/mercury-2.5` joins the judge seats; its writing seats wait on the calibration

Taken on the same delegation,
under the rule pre-registered on 2026-09-07,
after the owner's "Mercury 2.5 is out and approved" of 2026-09-09.
Instrument:
`judge-fidelity-probe --cap 48 --candidates inception/mercury-2.5 --candidates-alone`
over a throwaway runs dir holding the three settled artifacts on disk
(`gqt` from the fourth pass,
`hakureico` from the twenty-third,
`noname` from the first),
build `3224ff347`,
28 rows over fourteen distinct questions
(six deletions,
six insertions,
two alterations),
14 asks,
14 usable,
75 s,
record `~/temp/agent/probe-mercury-20260909/judge-fidelity-probe/`.

-   `inception/mercury-2.5`:
    14 of 14 chose the complete text,
    no damaged pick,
    no decline,
    position two on 7 of 14.
    A candidate at the maximum meets both clauses whatever the seated median reads;
    the seated roster's reading of the same fourteen is bought beside it and recorded in the planning log of
    2026-09-09 ("The owner asks where 200 USD went,
    and six levers land").
    Seated in critic,
    adjudication panel and judge in both lanes,
    the late bench and the slate (`fcc8ca197`,
    `SEATED_OPENROUTER_JUDGES` in `run-config.ts`).
    No writing seat yet:
    `WRITER_UNMEASURED` holds its translator and consolidation seats until
    `producer-calibrate 40 --candidates inception/mercury-2.5` (launched 17:06 UTC the same day) is read against
    the pooled null,
    the way `google.gemma-4-e2b` was seated on 2026-09-08.

OpenRouter alone serves it,
so it is a judge that answers while Synthetic stands at the bottom of its week and Hyper at its daily limit,
at 0.04 and 0.15 USD per million against the anchor judge's 0.58 and 1.74.

The seated roster's reading of the same fourteen landed at 17:18 UTC
(pid 875777,
`judge-fidelity-probe --cap 48` over a second copy of the same three artifacts,
28 rows,
roster verdict clean on 20 and declined on 8,
record `~/temp/agent/probe-seated-20260909/judge-fidelity-probe/`),
with Synthetic at the bottom of its week and Hyper dry,
so `glm-5.3` was lost on every ask
(`OPENROUTER_DROPPED_SEATS` holds it since `037d1f650` and no other provider served it)
and the other seats declined often.
Complete-text picks over the distinct questions each judge answered:

- `hf:moonshotai/Kimi-K3` 8 of 8,
  no damaged pick.
- `google.gemma-4-e2b` 9 of 11,
  2 damaged picks.
- `deepseek-v4-pro-0813` 6 of 9.
- `hf:zai-org/GLM-5.3-Flash` 5 of 9.
- `hf:Qwen/Qwen3.8-27B` 4 of 10.
- `deepseek-v4-flash-0731` 4 of 10.
- `minimax-m3` 3 of 6,
  1 damaged pick.
- `hf:openai/gpt-oss-120b` 2 of 6.
- `gemma-4-26b-a4b-it` 1 of 6.
- `glm-5.3` no usable ask.

The seated median lies between 4 of 10 and 5 of 9;
the most damage-prone seated judge chose the damaged text twice.
Mercury's 14 of 14 with no damaged pick clears both clauses by measurement,
not only by the maximum argument.

## Addendum 2026-09-09: `inception/mercury-2.5` takes the translator and the consolidation seat

Taken on the delegation of 2026-09-01,
by the rule of that day:
a writer leaves the translator seat when its z crosses the Bonferroni threshold below the pooled null,
and a candidate not separated from the null takes it.
Instrument:
`producer-calibrate 40 --candidates inception/mercury-2.5` on frozen `3224ff347`,
launched detached at 17:06 UTC and printing `STANDING over 40 rounds` at 20:02 UTC,
log `~/temp/agent/producer-calibrate-mercury-20260909.log`,
read with the scratch `read-standing.mjs`.

Standing as printed,
240 disinterested wins over 1229 ballots,
pooled null 19.5 percent,
Bonferroni threshold for ten comparisons z 2.81:

-   `gemma-4-26b-a4b-it`:
    38.6 percent (68 of 176 disinterested ballots,
    over 36 candidates),
    z +6.39,
    ABOVE the null.
-   `hf:zai-org/GLM-5.3-Flash`:
    23.1 (24 of 104,
    over 23),
    adjusted 14.7,
    z +0.91.
-   `deepseek-v4-flash-0731`:
    21.6 (22 of 102),
    adjusted 12.0,
    z +0.52.
-   `minimax-m3`:
    21.0 (33 of 157),
    adjusted 18.7,
    z +0.47.
-   `hf:Qwen/Qwen3.8-27B`:
    18.6 (8 of 43),
    adjusted 4.7,
    z -0.15.
-   `inception/mercury-2.5`:
    17.8 (18 of 101,
    over 23 candidates),
    adjusted 11.4,
    z -0.43.
-   `google.gemma-4-e2b`:
    17.3 (19 of 110),
    adjusted 9.6,
    z -0.60.
-   `hf:openai/gpt-oss-120b`:
    13.1 (19 of 145),
    z -1.95.
-   `hf:moonshotai/Kimi-K3`:
    10.9 (16 of 147),
    z -2.64.
-   `deepseek-v4-pro-0813`:
    9.0 (13 of 144),
    adjusted 7.3,
    z -3.18,
    BELOW the null.

The candidate is not separated from the null (z -0.43 against a threshold of 2.81),
wrote 23 of 40 candidates,
threw none of its 49 asks (`SEAT inception/mercury-2.5 asked=49 usable=49 unusable=0 threw=0`),
and its 49 completed streams ran p50 4.7 s,
p90 6.3 s,
max 10.1 s.
By the rule it takes the translator seat and,
as a measured writer,
the consolidation seat:
`WRITER_UNMEASURED` is empty again since `028432713`,
which also lowered its `COMPLETION_CAP` row from the pooled 99th to the pooled 90th,
its own p99 over 136 calls being 3,063 tokens.

WHAT THIS STANDING DOES NOT SAY.
Mercury took 0.7 of 37 round wins and appears in no resample's top three,
so it is a seat that does not lose,
not a seat that wins.
The bench that judged it sat without Hyper (its daily limit spent,
so `glm-5.3` threw 48 of 48) and without Synthetic from 17:25 UTC
(Qwen3.8-27B threw 35 of 56),
and `deepseek-v4-pro-0813` wrote and judged from NextBit without its reasoning
(the 2026-09-09 addendum).
Every candidate was compared under the same judges,
so the standing among them holds.

## Addendum 2026-09-09: `deepseek-v4-pro-0813` leaves the translator seat on its second reading below the null

Taken on the owner's authorization of 2026-09-01
("I authorize you to drop any model from any role,
as long as you have evidence"),
by the same rule.
The anchor judge has now been read below the pooled null by two 40-round calibrations:

-   2026-09-08,
    on Parasail with its reasoning:
    19 of 243 disinterested ballots over 39 candidates,
    adjusted 7.6 percent,
    z -2.31 against a 12.8 percent null,
    short of the threshold.
-   2026-09-09,
    on NextBit without it:
    13 of 144 over 29 candidates,
    adjusted 7.3 percent,
    z -3.18 against a 19.5 percent null,
    across the threshold of 2.81.

Landed as `a5e0efc7f`:
`TRANSLATOR_DROPPED` names it beside `gpt-oss-120b` and `deepseek-v4-flash-0731`,
exported so the seat test's count follows the set,
translators eight with Mercury in and the anchor judge out,
stage quorum 4.
It keeps every judge seat,
the critic and panel seats,
the consolidation seat and its place as the price anchor:
nothing here measures judging,
and the fidelity probe of the same evening read it at 11 of 14.
