# Translation repair: a pass with no reachable writer stops INCOMPLETE

Superseded in part on 2026-10-25 by
`package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md`:
the contest no longer asks a minimum ballot weight,
and two answering judges decide whatever their weight.
Extended on 2026-11-01 by the section "Addendum of 2026-11-01:
a short writing bench backfills from the role order,
and Sonnet refines":
the floor stays,
and the benches it counts are filled from the owner's role order before it stops an entry.

Decision of 2026-09-08,
the owner's,
asked after the eighth hakureico pass settled on Bedrock alone.

## The question

On the night of 2026-09-07 to 08 Synthetic's week stood at 0 percent,
Hyper's balance at 0,
OpenRouter's at 0.01 USD,
and Bedrock alone read wet.
The owner's standing words make any number of dry providers normal operation.
The seventh hakureico launch stopped INCOMPLETE at the pictures in 20 seconds,
since no picture reader had a Bedrock seat;
two Gemma sizes were measured and seated as readers
([`translation-repair-roster-seating-2026-09-01.md`](translation-repair-roster-seating-2026-09-01.md),
addendum 2026-09-08),
and the eighth launch passed its pictures and settled in 4.4 minutes for 0.22 USD.

What it settled:
a page with five slices rewritten by the one reachable translator (`gemma-4-26b-a4b-it`),
each chosen at the lane contest by three judges of nine
(the contest asks a minimum ballot weight,
not a majority of its bench),
zero repairs
(no editor or refiner has a Bedrock seat;
`editor round: 0/3 heard` 40 times,
`refiner round: 0/3 heard` 52 times),
every consolidation slate declined under `quorum-not-met`,
and the footnote passage the archive lacks left unfilled.
Every line the pass printed said so
(`repairStatus=unchanged translateStatus=unfilled` on the tally,
`short of quorum` before every phase since `c4a9682fe`),
and the page went to `fixed/` as `SETTLED`,
told apart from the sixth pass's whole-bench page
(fourteen changed slices,
three consolidated,
both footnotes)
only in the findings.
Read in
[`translation-repair-openrouter-2026-09-03.md`](../planning/translation-repair-openrouter-2026-09-03.md)
under "The eighth pass settles in 4.4 minutes".

## The options put to the owner

-   Stop INCOMPLETE when a writing bench (editors,
    refiners,
    translators) has no reachable seat and no provider has named its return.
    No one-writer page ships;
    the entry stays pending for a pass that has the benches,
    as INCOMPLETE entries already do;
    the floor is the thirteenth class's shape.
    A single-provider night ships nothing.
-   Ship what one provider buys,
    the behaviour as built:
    every line honest,
    something ships,
    and a page one writer and three judges made is `SETTLED` like any other.
-   Ship and mark the page partial for a later whole-bench pass to redo:
    a third terminal status through the artifact,
    the tally,
    the publisher and `verify-published`.

## The decision

Stop INCOMPLETE.
The owner chose the first option on 2026-09-08.

## What it means for the code

-   `BENCHES_BY_PHASE` in `run-seats-wait.ts` names the writing benches the lanes lean on
    (editors and refiners beside the translators),
    so the shortfall reading sees them.
-   A phase whose writing bench cannot reach its floor waits out a named hold as the thirteenth class does,
    and when no provider has named its return it stops the entry INCOMPLETE with the shortfall as the reason,
    instead of running on what is reachable.
    The judge benches keep the reading they have:
    a short judge bench with no hold runs on what is reachable and says so,
    since the settlement records `quorum-not-met` on every review and declines every slate under it.
-   The floor for a writing bench is one reachable seat short of what the lane needs to write at all
    (an editor bench of none writes nothing;
    a translator bench of one writes without a slate),
    measured on the lane drivers' own rounds rather than a fresh number;
    the exact floor is written with the fix and recorded in the planning log.
-   The entry's INCOMPLETE stop is the existing one
    (`stage-local work remains; whole entry will not restart in this invocation`),
    so a later pass with the benches picks the entry up unchanged.

## What this decision does not settle

-   Whether OpenRouter's meter at 0.01 USD should read dry before the first 402
    (the owner's rule against magic numbers keeps a threshold out;
    the 402 body names what the balance can afford).
-   Whether a pass that loses its writing benches inside a phase
    (the sixth pass lost Hyper two minutes into consolidation)
    stops at the next chunk or finishes the phase;
    the sixth pass finished its consolidation on the judges that remained,
    and that reading is recorded under the thirteenth class.

## Addendum of 2026-11-01: a short writing bench backfills from the role order, and Sonnet refines

Corpus pass XingZ6024 (build `e9b966af55`) stopped both attempts with
`JUDGE SEATS phase=lanes writing bench unreachable: refiners 1 of 3 reachable, floor 2`
and shipped only an interim page (88 archive slices,
30 source,
0 settled).
NeuralWatt was dry that day,
and Hyper and OpenRouter had never been topped up,
so GLM-5.3-Flash (served by NeuralWatt,
Hyper and OpenRouter,
all dry)
and minimax-m3 (served by Hyper and OpenRouter,
both dry) had no wet provider,
and DeepSeek V4.1 Flash alone was reachable on the refiner bench.
Kimi-K3 and Qwen3.8-27B,
both measured refiners and both served by the wet Synthetic,
sat unused,
because the refiner bench was the static list.
The checker bench already filled its dark seats from a ranked order;
the writing benches did not.

### The question and the answer

The owner was asked,
on 2026-11-01,
what a run should do when a writing bench (editors or refiners) has fewer than 2 reachable seats.
The options offered were
"Backfill from your order (Recommended)",
which fills dark seats with the next reachable models in the owner's role order,
and "Add Sonnet as a refiner",
which seats claude-sonnet-5-5 on the refiner bench as well as the editor bench.
The owner's answer:
"Backfill,
and add sonnet".
Both options are wanted.

The role order is the owner's "Order by role standing (Recommended)" of 2026-10-31 (ledger B1144),
held once in `shared-contract/src/seat-role-standing.ts`
and applied by `alternateModelsFor` in `provider/src/reask-alternate-models.ts`.

### What it means for the code

-   A reading derives each writing bench as the checker bench is derived:
    the static seats a wet provider serves,
    then the next models of the role order that a wet provider serves
    (not held out of the role by a card,
    not withheld by the run on the provider that would serve them,
    not already on the bench),
    until the bench reaches its static width,
    then the static seats nobody serves,
    which a stage reads as unreachable as before (`run-writing-backfill.ts`,
    `run-seats.ts`).
-   The floor of two reachable seats is unchanged
    (owner,
    2026-09-08,
    the decision this addendum sits in),
    and it counts the derived bench:
    a bench that the role order cannot lift to two still stops the entry with the clause,
    and "Never wait,
    relaunch" (owner,
    2026-10-29) still holds:
    nothing waits for NeuralWatt's day.
-   Claude Sonnet 5.5 is the second static refiner,
    as it is the second static editor.
    It runs on the owner's Claude key with no effort sent,
    as the editor seat does (the owner declined to pick an effort:
    "Why don't you bench?"),
    and is unmeasured as a refiner until the next `editor-calibrate`,
    whose refiner chair seats it by name.
-   The `JUDGE SEATS` line counts the editors and the refiners and names the seats taken from the role order
    (`backfilled-by-role-order=`),
    and the stop line says which seats the role order gave
    (`backfilled by role order:`).
-   Qwen3.8-27B was removed from the wide seats only (owner,
    2026-10-29),
    so it may write;
    the run still withholds it while Hyper would serve it,
    so the backfill skips it under that reading and takes the next model of the order.

### What changed for the reading that founded this decision

Bedrock alone,
the eighth hakureico pass,
stopped the lanes with no reachable editor and no reachable refiner.
Bedrock serves `gemma-4-26b-a4b-it` and `google.gemma-4-e2b`,
both in the pool of both roles,
so Bedrock alone now reaches a pair of editors and a pair of refiners from the role order and the lanes go on.
The editor calibration measured `gemma-4-26b-a4b-it` last of the six standings it recorded as an editor
(7.0 percent,
z -6.44) and as a refiner (3.4 percent),
and `google.gemma-4-e2b` was measured in neither role.
That is a consequence of "Backfill",
not a choice made in the code,
and it is recorded here for the owner to veto:
a reading that leaves only these two Gemma sizes to write the repair lane will ship a page they wrote.
The readings that still stop are those in which fewer than two seats of the static bench and the role order are served,
such as DeepSeek alone or the Claude key alone.

### A candidate that also checks

A model of the measured checker order (Kimi-K3 is the case) may be a candidate for the writing benches.
The checker contract refuses a model that edits or refines and also checks unless the run permits it
(`checkerSelfCertificationPermitted`,
owner,
2026-08-24:
"enable the discount and let every model do both"),
and a checker's ballot is weighed by the seat that cast it,
so a re-ask of a seat that ran away,
recorded under that seat,
would weigh in full for text its answerer wrote.
The backfill therefore seats such a candidate only when the checker bench of the same reading holds it,
where the discount on its own ballot applies,
and only while the run permits the overlap.
