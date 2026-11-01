# Roster seating of 2026-09-01: addenda of 2026-09-11 to 2026-09-24

The addenda dated 2026-09-11 to 2026-09-24,
each recording a seat a model took or left and the evidence for it.

Part of [Roster seating of 2026-09-01, on the two calibrations under the owner blocklist](translation-repair-roster-seating-2026-09-01.md),
which lists every file of this record.

## Addendum 2026-09-11: DeepSeek V4.1 Flash joins judging, not writing or image reading

Taken under the owner's delegated calibration-then-seat rule.
Serving approval alone did not release a role hold,
and no V4 Flash 0731 rating or exclusion was inherited.

The verified instrument used frozen `993583ad5`,
source-reviewed references for `gqt`,
`MTF_0615` and locally corrected `Y1Ran`,
and the same fourteen distinct message-plus-schema questions for the candidate and all nine peers.
Twenty-eight logical position/direction arrangements were deduplicated before scoring.
The source/reference and every deliberate damage were independently read before calls;
all 140 actual ballot reasons were read afterward.
No corpus file changed.

The paid run completed from 14:13:40.708 to 14:19:10.310 UTC:
140 stage calls,
140 model POSTs,
155 total transport operations including budget GETs,
and 140 schema-valid responses.
Every question heard all nine peers against the pre-registered quorum of five.
No missing ballot,
transport retry,
schema warning or abandoned-call record occurred.

Results on the fixed question set:

- V4.1 Flash selected the reviewed reference fourteen times,
  with no damaged pick or decline.
- Peer median clean count was fourteen.
- Maximum peer damaged count was one.
- Qwen,
  Kimi,
  MiniMax,
  V4 Pro 0813,
  V4 Flash 0731 and Mercury each selected fourteen references.
- GPT-OSS and Gemma 26B each selected thirteen references and declined once.
- Gemma E2B selected thirteen references and one damaged candidate.

V4.1 therefore meets both pre-registered clauses:
its clean count equals the peer median,
and its damaged count is below the peer maximum.
It identified the actual omitted content,
unsupported additions and changed year in both ballot positions.
Ancillary rationale errors,
such as calling one inserted sentence two sentences,
were not promoted into factual evidence or a claim of perfect reasoning.
An independent review confirmed judge admission under the existing rule.

`83e632127` removes only `deepseek-v4.1-flash` from `UNMEASURED_UNTIL_SEATED`.
It joins the general text/preparation roster,
wide judge/critic/panel bench and late judging bench once under one identity.
Hyper and OpenRouter remain routes for that identity,
not separate votes.
`WRITER_UNMEASURED` and `READER_UNMEASURED` remain unchanged.
The 40-round writer calibration and image-reading verification are separate tasks.

The post-change compiled boundary check compares every exported role array,
nested repair configuration and all sixteen provider-budget states with frozen `993583ad5`.
Existing seat order,
writer/reader arrays,
provider order,
completion caps and quorum formulas are unchanged.
The declared roster retains an unreachable identity under existing policy;
the router has no route when both Hyper and OpenRouter are dry.
The final check is `~/temp/agent/v41-judge-seat-final-boundary-20260911.json`,
using frozen post-admission `45e64e411`.
Build,
types and zero-warning lint pass;
`v41-judge-seat-final-unit-20260911.out` ends `unit exit 0` at line 9164.

All fourteen V4.1 calls were served by Hyper and returned the V4.1 identifier.
The dated predecessor returned its distinct 0731 identifier;
this is response-identity evidence,
not proof of an undisclosed backend implementation.

Costs remain separate by evidence and unit:
OpenRouter reported USD 0.00050599 across fourteen calls,
thirteen of which explicitly reported zero.
Bedrock's twenty-eight calls logged USD 0.00770297 from its pricing calculation.
Hyper's fifty-six calls estimate 2.674203982384 credits at the dated 2026-09-11 rates,
including 122213 explicitly reported cache-read tokens;
that is not a wire-reported debit,
and absent cache fields remain unreported.
Synthetic's forty-two calls remain subscription usage without an assigned per-call dollar price.
The spend and daily helpers were run after completion.

Evidence:
`~/temp/agent/v41-reviewed-run-20260911-vbOoMQ`,
`v41-reviewed-judge-results-20260911.json`,
`v41-reviewed-judge-ballots-20260911.out`,
`v41-reviewed-hyper-usage-20260911.json`
and the registered plan in
`~/temp/agent/v41-reviewed-plan-20260911-9eo8in/plan.json`.
The plan,
fixture and driver verification records remain under `doc/planning/`.

## Addendum 2026-09-16: `deepseek-v4-pro-0813` and `deepseek-v4-flash-0731` leave the roster

Taken on the owner's instruction of 2026-09-16,
verbatim:
"DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash,
so please remove V4 Pro and V4 Flash."
The owner's standing authorization
("drop any model from any role,
as long as you have evidence")
is not needed here:
the removal is an instruction,
not a measurement.

What changed:

- Both ids left `HYPER_ORIGIN_ROSTER_IDS`,
  the Hyper catalog,
  the OpenRouter catalog,
  the completion-cap table and the abandoned-spend ratio table.
  The Hyper price page keeps their rates,
  since that table is the provider's whole page rather than the roster.
- Both ids under both providers' spellings joined `ROSTER_BLOCKLIST`
  with the owner's sentence as the reason,
  so a catalog refresh labels them BLOCKED instead of proposing them again.
- `TRANSLATOR_DROPPED` lists only `hf:openai/gpt-oss-120b` now;
  the two DeepSeek readings below the pooled null stay in that constant's note
  as the method's worked examples.
- The third editor seat and the second refiner seat,
  both held by `deepseek-v4-pro-0813` on the 2026-09-01 measurement,
  passed to `deepseek-v4.1-flash`.
  The owner's sentence is the evidence for the successor taking the seat;
  no editor calibration has measured V4.1 Flash in either role,
  and the next `editor-calibrate` re-reads both seats.
  Keeping the seats at three was the reading that changed the least:
  the width of the editor bench was measured on 2026-09-01,
  and leaving a seat empty would have re-decided it.
- `WRITER_UNMEASURED` and `READER_UNMEASURED` are unchanged:
  V4.1 Flash still waits on its producer calibration for the translator seat
  and on a transcription measurement for the reader seat.

Unit tests followed the roster:
fixtures that used either id as a seat name now use `deepseek-v4.1-flash`
or `google.gemma-4-31b`,
the OpenRouter abandoned-spend example moved to MiniMax M3
(ratio 137 over 4,015 streams),
the repair slice key golden literal moved a third time
(the fixture's refiner id changed),
and the roster count is eleven.
Recorded evidence that names the departed ids
(captured streams,
false-start samples,
the class twenty-four endpoint measurement)
stays as written.

Build,
types,
zero-warning lint and the unit suite pass on the commit that records this addendum.
The pass running when the instruction arrived
(`Mio18`,
frozen `e964d2f3a`)
was killed and relaunched on the new tip under the always-kill-and-relaunch rule.

## Addendum 2026-09-18: `typesafe/jev-1.13` joins the select judges as a decision-only seat

Taken under the owner's delegated calibration-then-seat rule after the owner's approval of the model on OpenRouter
(2026-09-18,
00:29 UTC).
Jev 1.13 answers typed questions over a state on OpenRouter's decisions endpoint
and has no chat completion,
so it is the first roster model with a `decisions` side and no provider side:
`OPENROUTER_DECISION_IDS` in `src/roster-id.ts`,
`DECISION_ONLY_ROSTER_IDS` in `src/model-card-derive.ts`,
off `RUN_ROSTER` and every chat bench,
on `RUN_SELECT_JUDGES` through `RUN_DECISION_JUDGES` once measured.

The instrument was the production judge fidelity probe on the compiled package at `bc6fdc9b9`
(`mise run judge-fidelity-probe -- --candidates typesafe/jev-1.13`,
01:40:41 to 01:47:09 UTC),
the source-reviewed references for `gqt` and `MTF_0615`,
16 reviewed rows
(deletion and insertion,
preserve and replace,
both candidate orders),
the candidate beside the ten seated judges,
the same select question every peer answered as chat asked of Jev as one `choice` over the candidate numbers.
The probe warned the matrix partial,
16 of 28,
because the locally corrected `Y1Ran` reference was not on disk in this worktree;
a scratch measurement at 01:05 UTC over all 28 rows,
outside the package,
had chosen the reference 28 of 28 times
(the pass log's "Jev 1.13 approved" heading).

Results on the 16 rows:

- Jev was asked 12 times
  (the fan-out window of quorum plus one spare left it out of the four `gqt` insertion rounds)
  and chose the reviewed reference 12 times,
  no damaged pick,
  no decline,
  probabilities 0.75 to 0.96 on the reference.
- DeepSeek V4.1 Flash and Mercury each chose the reference on all 12 of their rows.
- Qwen and GLM-5.3-Flash chose it on all 10 of theirs;
  Kimi,
  GPT-OSS and MiniMax on all 8.
- Gemma 26B chose it 8 times and declined twice of 10.
- Gemma E2B chose it 8 times and a damaged candidate twice of 10.
- glm-5.3 was unreachable on every row
  (Hyper dry).

Jev meets both clauses of the admission rule:
its clean count equals the peer median on the rows it was asked,
and its damaged count is below the peer maximum.
Every call went through the decisions route
(`SPEND provider=openrouter model=typesafe/jev-1.13`,
1173 to 1211 prompt tokens,
38 output tokens,
338 to 685 ms,
0.000049 to 0.000051 USD each);
`SEAT typesafe/jev-1.13 asked=12 usable=12 unusable=0 threw=0`.

`c14c7f176` removes only `judge-unmeasured` from the card
(guards red first at `1e2c18de4`:
`decision-seat.unit.test.ts`,
`roster-fixture.unit.test.ts`;
`run-seats.unit.test.ts` now expects the select bench one seat wider than the wide bench).
It votes only where a typed question is threaded,
which today is the candidate-select stage of the repair and translate lanes
(`selectDecision` in `src/candidate-select-decision.ts`);
consolidation slates,
lane contests,
panels and gates call chat and leave it a lost voice.
`writer-unmeasured` and `reader-unmeasured` stay on the card as a matter of kind,
not of measurement:
a typed decision writes and reads nothing.
Build,
types,
zero-warning lint and the unit suite pass on `c14c7f176`
(`~/temp/agent/jev-seat-check2.log`).
XingZ604 launched at 01:56 UTC on the frozen build and its JUDGE SEATS line reads `wide=8 select=9`.

Evidence:
`~/temp/agent/jev-probe.log`,
`~/temp/agent/jev-probe/judge-fidelity-PzAXzN/judge-fidelity-probe/2026-09-18T01-40-41.110Z-7d877967.json`.

## Addendum 2026-09-19: `deepseek-v4.1-flash` takes the writer seats and `inception/mercury-2.5` leaves the translator seat

Taken on the delegation of 2026-09-01,
by the rule of that day:
a writer leaves the translator seat when its z crosses the Bonferroni threshold below the pooled null,
and a candidate not separated from the null takes it.
Instrument:
`producer-calibrate 40 --candidates deepseek-v4.1-flash` on frozen `6eada935a`,
launched detached at 14:53 UTC and printing `STANDING over 40 rounds` at 16:45 UTC,
log `~/temp/agent/producer-calibrate-v41flash-20260919.log`,
read with the scratch `read-standing.mjs`.
About 0.24 USD by the OpenRouter spend lines,
1h52m.

Standing as printed,
170 disinterested wins over 808 ballots,
pooled null 21.0 percent,
Bonferroni threshold for nine comparisons z 2.77:

-   `gemma-4-26b-a4b-it`:
    35.0 percent (36 of 103 disinterested ballots,
    over 25 candidates),
    adjusted 31.2,
    z +3.46,
    ABOVE the null.
-   `hf:zai-org/GLM-5.3-Flash`:
    32.5 (26 of 80,
    over 22),
    adjusted 25.5,
    z +2.51.
-   `deepseek-v4.1-flash`:
    24.0 (29 of 121,
    over 27),
    adjusted 23.1,
    z +0.79.
-   `hf:Qwen/Qwen3.8-27B`:
    22.1 (15 of 68,
    over 18),
    adjusted 14.2,
    z +0.21.
-   `hf:moonshotai/Kimi-K3`:
    19.6 (10 of 51,
    over 15),
    adjusted 10.5,
    z -0.25.
-   `minimax-m3`:
    18.2 (18 of 99,
    over 27),
    adjusted 17.5,
    z -0.70.
-   `google.gemma-4-e2b`:
    16.7 (14 of 84,
    over 19),
    adjusted 11.3,
    z -0.98.
-   `hf:openai/gpt-oss-120b`:
    12.8 (12 of 94,
    over 26),
    adjusted 11.9,
    z -1.97.
-   `inception/mercury-2.5`:
    9.3 (10 of 108,
    over 28),
    adjusted 9.3,
    z -3.00,
    BELOW the null.

The candidate is not separated from the null (z +0.79 against a threshold of 2.77),
wrote 27 of 40 candidates,
threw none of its 55 asks (`SEAT deepseek-v4.1-flash asked=55 usable=55 unusable=0 threw=0`),
took the most round wins of the bench (8.3 of 35 winner-bearing slices,
`gemma-4-26b-a4b-it` next at 6.8)
and sat in the resampled top three 90.7 percent of the time.
By the rule it takes the translator seat and,
as a measured writer,
the consolidation seat:
`writer-unmeasured` leaves its card;
`reader-unmeasured` stays,
since no transcription has been measured.

Mercury crossed the threshold below the null on this reading
(z -3.00 against 2.77;
its own calibration of 2026-09-09 read it at z -0.43,
"a seat that does not lose,
not a seat that wins").
By the rule it leaves the translator seat and keeps the consolidation seat,
the shape of `deepseek-v4-pro-0813`'s exit of 2026-09-09:
`translator-dropped` goes on its card.
It keeps every judge seat
(14 of 14 on the fidelity probe of 2026-09-09;
nothing here measures judging).
Translators stay eight
(V4.1 Flash in,
Mercury out),
consolidation writers eleven,
stage quorum 4.

Landed as `9f8dcccd2`,
tests red first at `abcc4368c`
(`roster-fixture.unit.test.ts`,
`run-seats.unit.test.ts`,
`deepseek-v41-admission.unit.test.ts`).
XingZ617 (frozen `c26648e2a`,
launched 16:45 UTC by the waiter on the calibration's exit)
was killed at 16:54 UTC under always-kill-and-relaunch;
XingZ618 runs on frozen `9f8dcccd2`.

WHAT THIS STANDING DOES NOT SAY.
The bench that judged it sat without Hyper
(its balance 0,
so `glm-5.3` threw 54 of 54 and no candidate of its reached any slate)
and without Synthetic from 16:45 UTC
(`Qwen3.8-27B` threw 17 of 46,
`Kimi-K3` 25 of 59).
Every candidate was compared under the same judges,
so the standing among them holds.
Mercury's ten wins over 108 ballots are a reading of one calibration;
the v4-pro exit waited for a second reading because its first fell short of the threshold,
and this one does not.

## Addendum 2026-09-24: `hf:openai/gpt-oss-120b` leaves every role at the owner's instruction

The owner,
2026-09-24,
on reading the CuspariaKLSY10 artifact:
"That particular model got cause and effect wrong.
Cull it."
and then "cull it from every role".

The evidence.
On CuspariaKLSY10 slice 1 the critic `hf:openai/gpt-oss-120b` filed,
as the sole proposer,
the claim "Translation omits the detail 服用激素 (taking hormones) as the reason for transitioning"
against the original's 服用激素，成为跨性别的原因：,
which lists taking hormones beside the reasons rather than as one of them;
the panel accepted it as a major omission.
On zheermao8 slice 9 the same seat declined every consolidation candidate
because "all candidates render the quoted lines in present tense"
on a slice that quotes no line.

What changed.
The card gains the hold `owner-culled`
("out of every seat by the owner's decision"),
`RUN_ROSTER` and `RUN_READER_MODELS` filter it,
so every derived bench (wide,
late,
select,
slate,
translators,
writers,
readers) loses the seat,
and the static checker bench passes its third seat to `gemma-4-26b-a4b-it`,
the substitute checker since 2026-09-03 and the third checker on every Synthetic-dry reading since;
`google.gemma-4-e2b` becomes the substitute on the same ground
(no editor or refiner seat;
a judge by the Bedrock fidelity probe of 2026-09-07;
a writer by the producer calibration of 2026-09-08;
Bedrock alone serves it,
the provider the withheld Synthetic checkers never sit on).
Both checker seatings stay PROVISIONAL:
no checker-side measurement exists for any model.

Why the card stays.
`hf:openai/gpt-oss-120b` is the one identity every provider serves,
and the unit fixture seats it as `SEAT_SYNTHETIC_TEXT_EVERYWHERE` in that role;
deleting the card would re-point or delete that seat across the reach,
router and catalog suites for no gain in production,
where the hold already empties every bench
(`src/corpus-run/owner-cull.unit.test.ts`).
The blocklist is not touched:
it labels ids the catalogs do not carry,
and its guard refuses a compiled catalog that seats a blocked id,
which a kept card would trip.
The roster is eleven chat seats plus the decision-only seat.
