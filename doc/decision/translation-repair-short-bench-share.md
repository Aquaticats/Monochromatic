# A deciding bench short of quorum seats a winner by a share of the reachable weight

Superseded in part on 2026-10-25 by
`package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md`:
the minimum weight a winner needs (`MIN_SELECTION_WEIGHT`) and its scaling by reachable seats over quorum are gone,
and two ballots naming the winner decide whatever their weight.

Decided by the owner on 2026-09-09 ("Share of reachable weight"),
asked with three options after the second `noname` and the sixth `Mio` ran on Bedrock alone
(2026-09-09,
12:22 to 12:33 UTC;
handover,
"The thin-bench questions").

## What happened

- Bedrock alone reaches three seats of the wide bench,
    every one of them also a writer.
- `MIN_SELECTION_WEIGHT` of 2 in `candidate-select-model.ts` is absolute,
    so on that bench a winner needed every reachable judge,
    or two disinterested ones where a producer sat.
- The second `noname` shipped with its `## 简介` heading unfilled and the sixth `Mio` stopped INCOMPLETE at slice 3.

## The rule

- The seated bench is what `judgeSeatsFor` passes to the stage:
    a seat no wet provider serves stays on the bench and is lost at call time as `NoProviderForModelError`.
    The reachable bench is the seated bench less the seats lost that way.
    Quorum is `rosterQuorumSize` over the seated bench,
    as in every other stage.
- At or above quorum nothing changes:
    plurality wins,
    a tie declines,
    and the winner needs `MIN_SELECTION_WEIGHT`.
- Short of quorum the minimum weight is `MIN_SELECTION_WEIGHT` times reachable over quorum:
    the share of the bench that 2 is of a quorum bench.
    The rule scales the existing minimum instead of choosing a new number.
- Two ballots naming the winner is the floor whatever the weight (`MIN_SELECTION_BALLOTS`):
    one judge never decides,
    which is what the absolute 2 was for.
- The round's findings say `select-short-bench (reachable r of n, minimum m)`,
    so a page seated this way is told apart in its findings.

## What this is not

- Not the fifteenth class widened:
    a deciding bench short of quorum keeps deciding,
    and the writing-bench floor (`translation-repair-writing-bench-floor.md`) still stops an entry with no writer.
- Not a change on any bench at quorum:
    the archive-block review's four seats,
    the fixtures and the full production bench keep the absolute 2 and the three-halves bound
    `runCollapsedSelection` pins.

## Options rejected

- Stop INCOMPLETE,
    the fifteenth class widened (ranked first when asked):
    no thin page ships and it is the shape the owner chose for the empty bench,
    but on one provider nothing ships,
    and one provider wet is normal operation by the owner's words.
- As now:
    a page three judges made lands in `fixed/` beside one the whole bench made,
    told apart only by its findings,
    and ships with insertions unfilled.
- Widening Bedrock's judge bench with `google.gemma-4-31b`:
    measured out on 2026-09-07 (6 of 12 against a seated median of 9.5).

## What landed

- `stage-call.ts`:
    a lost voice says whether the router refused the seat (`unreachable`,
    true on `NoProviderForModelError`),
    and `stage-quorum.ts` collects those seats on the gather as `unreachable`.
- `candidate-select-minimum.ts`:
    `selectionMinimum` (the weight,
    reachable count,
    quorum and whether the bench was short),
    `MIN_SELECTION_BALLOTS` and `shortBenchFinding`.
- `candidate-select.ts` applies it after the gather,
    declines a leader named by one ballot as `winner named by one judge alone`,
    and carries the finding;
    its ballot weighing and tally moved to `candidate-select-count.ts` at the line cap.
- `repair-contract.ts`'s capacity guard is unchanged:
    it reads the seated bench at stage entry,
    where every seat is still on it.
- Guards shown to fail on the build before the change:
    `candidate-select.unit.test.ts` ("selection on a bench short of quorum":
    three of eight reachable seats a producer-plus-one agreement at three halves,
    two of eight declines a lone full ballot,
    a bench at quorum keeps the absolute 2),
    `stage-quorum.unit.test.ts` (the gather names the refused seat apart from a transport loss),
    `candidate-select-minimum.unit.test.ts`.

## Addendum 2026-09-09: the share reaches every stage gather, not only the select round

The owner's answer was built into `candidate-select-minimum.ts` first,
which sizes the WEIGHT a winner needs.
The GATHER still sized its quorum on the seats the phase seated,
and on 2026-09-09 at 20:23 UTC that cost an entry:
the seat reader read Synthetic wet at 0.04 percent of its rolling week and seated Qwen3.8-27B and Kimi-K3,
the provider refused at 20:25 UTC,
the router named those two and the dark `glm-5.3` unreachable,
and `hulicaijia`'s archive block review needed 6 voices of an 11-seat bench,
heard 5,
spent four retry rounds re-asking the three seats no provider served,
and threw `provider-unavailable` at 382 seconds.
The entry stopped INCOMPLETE on a bench that had four seats able to answer.

Since `40aba2fdb` the same share sizes every gather (`stage-reachable-quorum.ts`):
`reachableQuorum({ benchSize, unreachable, })` keeps the bench quorum while the reachable seats can meet it,
and otherwise needs half the reachable seats rounded up,
never fewer than `MIN_STAGE_VOICES` of 2.
A seat the router refused is dropped from the pending list rather than re-asked,
since nothing changes between rounds for a seat no wet provider serves.
A short gather warns `<stage>: bench short of quorum, reachable r of n; closing on q voices` and carries
`stage-short-bench (<stage> reachable r of n, quorum q)` into the artifact whatever the verdict.
A bench with one reachable seat,
or none,
still reads as an outage,
which is what the two-voice floor is for.

## Addendum 2026-09-27: a refused seat hands its round place on

The rule is unchanged:
a seat no wet provider serves stays on the seated bench,
and the quorum and the short-bench findings read it as unreachable.
What changed is the round it was asked in.
A window seat the router refused in the same millisecond had spent the round's spare place,
so the round waited for its slowest reachable voice with no grace or fell through to a retry round:
TianqiChen66620 closed 269 of 421 rounds a seat short with no grace,
2,852 s of the 4,575 s its rounds took.
Since `84a6caa02` a refused window seat hands its place in that round to the next pending seat
(`runGatherRound`'s `reserve` in `stage-round.ts`),
and the windowed stages (`stage-windowed-rounds.ts`) no longer re-ask a refused seat in later rounds,
which the gathers had stopped doing on 2026-09-09.
The windowed stages still sized their quorum over the seated bench rather than the reachable one
(audit ledger X8) until the next addendum.

The checker bench is the one bench that does not keep an unserved seat while a served one is available:
since 2026-09-27 it is the first three of the measured checker order a wet provider serves,
padded with unserved ranked seats only while fewer than three are served
(`run-seats.ts`,
audit ledger L1).

## Addendum 2026-09-27, second: the windowed stages and the naturalness review size on the reachable bench

Taken under the rule itself,
by the whole-package audit (audit ledger E3 and X8).
The six stages that read their own rounds (lane contest,
section and block pairing,
the consolidation gate,
the polish gate and the absolute naturalness review)
sized their quorum on the seated bench.
Five of them decide on two ballots,
so for them the bench quorum only set how long rounds chased seats;
the naturalness review decides on it,
and 59 of the 4,567 reviews in the 265 stored artifacts closed `quorum-not-met` with 2 to 4 usable seats.
XingZ624 shows the shape:
a confirmation asked the 5 seats its discovery asked,
over a bench of 8,
and closed 3 of 5 usable against a quorum of 4 with `glm-5.3` and Qwen3.8-27B refused out of budget.

- `runWindowedRounds` takes the bench (`quorumOver`,
  the seats asked by default)
    and re-sizes the quorum with `reachableQuorum` before every round,
    counting out of reach the seats the router refused
    and the bench seats the stage may not ask at all.
    It returns the quorum it closed on and that count,
  and warns when the bench was short.
- The naturalness review decides on that quorum and records the count as `unreachable` in its review round;
    the artifact reader recomputes the verdict from it,
    and reads a round without it,
  every round stored before this change,
  as none out of reach,
    which is exactly the exact-half quorum those rounds were decided by.
- The other five stages carry `stage-short-bench (<stage> reachable r of n, quorum q)` in their findings,
    as the gathers do.
- Counting the seats a confirmation may not ask as out of reach was taken first from the two settled rules
    (the reachable share sizes every gather;
  a confirmation asks only the discovery's seats),
    then put to the owner the same day,
  who chose "Count as out of reach"
    over counting only the seats the router refused.
    With nothing refused the confirmation's quorum is unchanged;
    it drops to the reachable share only where the seats it may ask cannot meet the bench quorum.

## Addendum 2026-10-21: a refusal the router gives is not an ask

The rule is unchanged:
a seat no wet provider serves stays on the seated bench and is lost at call time as `NoProviderForModelError`,
and a refusal is not an ask.
The seat tally did not hold to it.
`seatTallyClient` counted the router's own refusal as a failed ask of the seat,
so `zheermao20` printed `SEAT glm-5.3 asked=4 usable=0 unusable=0 threw=4` for a seat never sent a request,
and withheld it for the run after four refusals
(audit ledger B711).

The counting rule,
written on 2026-10-21 (UTC):

- A `NoProviderForModelError` thrown by the client the tally wraps is counted as `unreachable`.
    It is in neither `asked` nor `threw`,
    and it counts toward no withholding threshold.
    That holds whether or not the caller had aborted,
    and for the owner's cull,
    which gives the same refusal inside the tally.
- `withheld({ minAsks })` stays "asked at least `minAsks` times and every ask threw",
    over real asks only.
    A seat that only the router refuses is never withheld by the tally.
    The router refuses it cheaply at every call,
    the stages read the refusal as an unreachable seat and size their quorum on the reachable bench,
    and the seat comes back by itself when a provider is wet again.
- `dark()` stays "asked at least once and never once produced a usable answer",
    so a seat only the router refused is not dark.
    Its evidence is the `unreachable` column of its `SEAT` line
    and the `unserved=` field of the `JUDGE SEATS` lines.
- A failure of any other kind,
    including a rejected credential where no other provider is left to try,
    is counted and withheld as before.

The seats that sit on a bench and that no wet provider serves are now named in the log
(`unserved=` on every `JUDGE SEATS` reading line,
`unserved-checkers=` on the repair lane's `chunk re-seated under a hold` line),
so the checker bench padded with unserved ranked seats (addendum of 2026-09-27) is visible where it is seated.

## Addendum 2026-10-22: a bench left with exactly its quorum closes one voice below it

Taken by the owner,
who answered "Let the grace open" to the question of whether a short bench that needs every live seat should wait on its
last seat at all
(audit ledger B809 and B816;
the decision is recorded in `doc/decision/translation-repair-straggler-grace.md`,
section "Decision 2026-10-22").

The addendum of 2026-09-09 says `reachableQuorum` keeps the bench quorum while the reachable seats can meet it.
That held for a reachable bench equal to its quorum too,
so a consolidation bench of 7 with 3 seats unreachable needed 4 of 4 live seats,
and one stream that held without content held the round while the straggler grace,
which opens only once a quorum stands,
could not open.

The rule now:

- Where the reachable seats equal the bench quorum and that quorum is above `MIN_STAGE_VOICES` (2),
  `reachableQuorum` needs one voice fewer than the bench quorum and marks the bench short,
  so the findings carry `stage-short-bench (<stage> reachable r of n, quorum q)` as for any short bench.
  Seven seats with three dark close on three of four;
  eleven seats with five dark close on five of six.
- Every other shape keeps the rule of 2026-09-09.
  A reachable bench above the bench quorum needs the bench quorum.
  A reachable bench below it needs half the reachable seats rounded up,
  never under two.
- A bench of 3 with 1 unreachable has a bench quorum of 2 and keeps both live seats:
  the floor of two voices holds,
  so one judge never decides.
  Whether the floor should give way on a bench of three was not asked.
- The last live seat still gets the grace after the reduced quorum stands.
  A verdict is decided with one voice fewer only when that seat does not answer inside it.
- A refusal inside a round,
  a slot handing its place to a reserve seat included,
  sizes the need again (`heardNeededAfterRefusals` in `runGatherRound`),
  so round 0,
  which is sized before the router names any seat,
  opens its grace on the quorum the refusals leave.
  The gather and the windowed stages both pass it.

What this does not change:
the weight a deciding bench needs (`selectionMinimum` in `candidate-select-minimum.ts`) is a separate rule,
and the commits of this rule did not touch it.

Records stored under the rule of 2026-09-09:
the naturalness review recomputes its verdict from the seats a stored round recorded,
so a stored round whose live seats equal its quorum reads acceptable under the new rule and short of quorum under the old.
Artifact generation 20 gates the rule (`artifactGenerationLetsGraceOpenOnLastSeat`),
`reachableQuorum` takes `lastSeatGrace: false` to read an older record by its own rule,
and a build of 20 refuses to resume a run directory stamped 19 or earlier.
The cache versions of every stage that caches a gather moved with it (audit ledger B816).
