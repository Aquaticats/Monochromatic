# Introduced-defect probe gating

Decision record for whether `runIntroducedDefectProbe` may block a repair.
Decision:
 no,
 taken on 2026-08-07.
The probe stays in shadow mode,
 recording into the outcome and the artifacts while candidate selection ignores it,
until its false-positive rate has been graded against a human repair sheet.

This closes task #53 in the session task tracker,
 which is a different numbering from GitHub issues;
GitHub issue 53 is an unrelated port-registry item.

## Background

The probe asks whether a repair broke something nobody raised.
That question is invisible to `regressedKnownIssues`,
 which reads verdicts keyed by issues a critic already filed,
so a defect the repair introduced itself has no filer and no verdict.

Shipping it in shadow mode was deliberate from the start.
The open question was the next step:
 whether to give it authority over candidate selection,
 and if so what the pipeline should do at the moment it corroborates damage.

## Options considered

Ranked best first,
 with the reason deciding each adjacent pair.

-   **Stay in shadow mode.**
    Chosen.
    Preserves every correct fix,
     and the measurement that would settle the question is already scheduled at no extra quota.
    Costs the single corroborated region across 83 continuing to ship.
-   **Fall back to the runner-up editor candidate.**
    Preserves a repair at no new model calls,
     since every candidate was already produced,
     judged,
     and checked in that round.
    Rejected because it still acts on an unmeasured error rate,
     and the measurement is close enough that deciding now buys nothing waiting does not.
-   **Salvage the surviving subset.**
    Drops the confirmed-defective operations,
     reapplies the rest from the original target,
     revalidates.
    Ranked below the fallback because it costs a full extra round of judging,
     checking,
     probing,
     and selection per trigger,
    to buy the one region's worth of preserved repair seen across nine entries.
-   **Reject the whole chunk.**
    Simplest to implement and never ships a flagged repair.
    Ranked last because observed chunks resolve 9/9 to 17/17 accepted issues,
     so it discards many good repairs to remove one suspect region.

## Why the measured evidence points at deferral

THE PROBE IS A BIASED INSTRUMENT BY CONSTRUCTION.
Every region it inspects contains a defect,
 that being why the region was edited,
so a model asked whether anything is wrong will find something.
This is not a flaw to fix;
 it is the reason a raw probe rate cannot be read as a damage rate.

THE CORROBORATION COLUMNS SEPARATE CLEANLY.
`majorityIntroduced` held at exactly 1 across 1,
 5,
 6,
 7,
 8,
 9,
 and 15 settled entries while the region count went 13,
 67,
 68,
 72,
 79,
 83,
 210,
then reached 2 at 18 entries and 246 regions,
 and 7 at 28 entries and 412 regions.
`minorityIntroduced` went from 1 to 45 over the same span,
 tracking region count almost exactly.

A CORRECTION BELONGS HERE.
At 2 events this document said the rate was "roughly 1 in 120 and has stayed
 there rather than climbing."
That was not supportable and should not have been written.
Two events carry almost no information about a rate:
 the interval around 2 in 246 comfortably contains the 7 in 412 measured
 afterwards,
so the honest reading is that the early numbers were too sparse to say anything
 about stability,
not that stability was observed and later broken.

What 7 events do support is a firmer estimate:
 roughly 1.7% of distinct shipped regions draw a majority,
 about 1 in 59.
The 7 are spread one apiece across 7 different entries
 (`AmbeR_the_anpa`,
`Kotori`,
`LCG_Akiball`,
`MTF_0615`,
`MeowBot233`,
`Mio`,
 `Mizuki_Yuuki`),
so this is a low per-entry base rate rather than one pathological document
 inflating a total.

The direction of the decision is unchanged,
but the ARGUMENT has to change with
 the evidence.
It no longer rests on the majority column barely moving.
It rests on the rate being low and,
more importantly,
on nobody yet knowing
 whether those 7 regions were damaged at all.
Probers keep making claims;
 what stays rare is a majority agreeing on one region.
That is the shape of a sensitive but noisy instrument,
 and it is the opposite of the silently-always-negative failure the sensitivity check was built to rule out.

THE DETERMINISTIC SCREEN IS HOLDING,
 and it has now fired.
By 246 regions it had thrown out 1 contradicted claim and 2 unanchored ones,
 where at 83 regions it had thrown out none,
so the screen is demonstrably capable of rejecting a claim rather than merely
 never having met one it disliked.
`unprobedRecords` stays at zero over 1214 shipped records
 and `degradedRosterRegions` at zero,
so the probe is reaching everything it should rather than skipping quietly.

ONE SCOPE LIMIT MATTERS FOR THE GRADING,
NOT FOR THE GATING.
The probe runs inside the accuracy stage and the naturalness lane runs after it,
 so on a slice the lane rewrote,
the probe judged wording that did not ship.
For the GATING question this is harmless:
 a gate would act during candidate selection,
 which is also before the lane runs,
so the probe judges exactly the text such a gate would judge.
For VALIDATING the probe against human repair grades it is not harmless,
 because the repair sheet shows the human the returned wording and asks them to
 grade that.
Those positions compare two texts.
`score-probe` now reports them as `refinedJoined`,
 10 of the 50 in the round-three sample,
so the comparison this decision waits on can exclude them rather than average
 them in.
A quote must be new in the replacement,
 or gone from it for dropped content,
so an impossible claim is dismissed without anyone having to prove a possible one.

WHAT IS STILL MISSING IS THE DENOMINATOR THAT MATTERS.
Nothing here says how often a corroborated region is genuinely damaged
 rather than a plausible-sounding claim about a region that was already defective.
Only a human repair grade answers that,
 and gating on the answer before having it would discard correct fixes on unmeasured evidence.

## Revisit condition

Reopen this decision when the round-three repair sheet has been graded
 and the probe's corroborated regions can be compared against those grades.
That grading is tracked as task #48;
 the comparison itself is task #60,
so this deferral cannot quietly become an omission.

Two outcomes are worth naming in advance:

-   If corroborated regions are usually genuinely damaged,
     the fallback option becomes the cheap next step,
     because the candidates it needs already exist in the round.
-   If they are usually not,
     shadow mode is the permanent answer
     and the probe's value is telemetry about editor behavior rather than a gate.

## What this decision does not change

The probe keeps running,
 keeps writing its report into the outcome and the artifacts,
 and keeps costing its share of the run.
Nothing about it is disabled.
`compareCandidates` remains untouched,
 which is what makes this a recorded default rather than a code change.
