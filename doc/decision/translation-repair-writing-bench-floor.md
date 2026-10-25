# Translation repair: a pass with no reachable writer stops INCOMPLETE

Superseded in part on 2026-10-25 by
`package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md`:
the contest no longer asks a minimum ballot weight,
and two answering judges decide whatever their weight.

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
