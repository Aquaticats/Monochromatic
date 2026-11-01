# The pipeline translates every slice and selects, rather than routing by ratio

Decision record for the shape of `@monochromatic-dev/module-translation-repair`.
Decision:
 every slice is translated afresh,
 the existing translation competes as one candidate,
 and judges select per slice.

Provenance,
stated exactly because it is a derivation rather than a quotation:
 on 2026-08-15 the user REJECTED the recommended shape and gave two criteria for
 rejecting it,
 and those two criteria pick this shape out of the three that were on the table.
The derivation is set out in "What was proposed and what the user said".
It was reported to the user for correction when it was taken.

It closes the translate-first re-design proposal,
 which `doc/decision/translation-repair-output-goal.md` left open when it decided
 the goal and named a re-design as likely rather than chosen.

## What was proposed and what the user said

Three shapes were put up,
ranked A over C over B,
 with A the recommendation:
 route a section to a translate lane when its target-to-source character ratio
 falls below a line the corpus supplies.

The user rejected it:

> Your recommended option is invalid.
> "ratio" is a magic number and if we go through the option we'll end up with
> output quality that isn't the best possible.

Two criteria are in that sentence,
 and together they pick one of the three shapes.

A ratio is a PROXY for the question the pipeline actually has,
 which is whether this slice's English is the best available rendering of its
 Chinese.
Every proxy has a boundary,
 and a boundary decides sections by a number rather than by their content.
The measured gap between 0.74 and 1.32 makes the line defensible;
 it does not make it the question.

Best-possible quality is a ceiling statement.
Routing by ratio leaves 248 of 254 sections on the repair path,
 where the ceiling is the input translation plus whatever defects the critics
 happen to name.
A slice whose translation is present,
fluent and mediocre is invisible to a
 ratio,
to a coverage check,
and to a critic looking for nameable defects.

## What is decided

Every slice is translated from the SOURCE by the editor ensemble.
The existing translation enters selection as one candidate among them.
The judges already built for candidate selection choose per slice.

Good human translation survives by WINNING selection rather than by never being
 touched,
 which makes preservation a measured outcome instead of an assumption.
There is no threshold anywhere in the routing,
 because there is no routing:
 absent,
partial,
nonsensical and excellent input all take one path.

Cost is not an objection to this and was not weighed as one.
The user's standing on that is on record from 2026-08-14:
 a complaint that the best option is the most expensive does not count in what
 this project is trying to do.
The measurement stands for the record rather than as an argument:
 option B raises editor calls by 1.56x and editor output volume by 3.9x,
 because the editor already fires on 64% of slices holding 75% of slice text.

## What this obliges, none of which is optional

The introduced-defect probe compares an AFTER text against a BEFORE text.
A from-scratch translation has no before text,
 so that instrument does not survive this change as it stands.
Its replacement already exists in intent:
 the probe question was re-anchored on 2026-08-12 to
 "does the AFTER text misrepresent the ORIGINAL",
 which never needed the before text except to establish that an edit caused the
 change.

Judge quality on the preserve-or-replace question has never been measured.
This shape stakes every slice on judges preferring a good human translation to a
 fluent machine one,
 and milestone three deferred the judge crosscheck.
Measuring it is now a precondition rather than a nicety,
 because a weak judge under this shape does not degrade one repair,
 it rewrites the corpus.

The transcribed-image class has no answer here.
Some sections carry content the original holds only inside an image,
 which a translator transcribed.
Selection alone has no evidence for preferring a candidate that carries
 image-borne content,
 so a fresh translation can win by being faithful to a source that is missing
 the content.

## What is NOT decided

Whether the critic stage survives.
Critics exist to find defects for an editor to repair,
 and this shape repairs nothing.
Dropping them would remove 582 calls and add 105,
 making the new shape cheaper than the one it replaces;
 keeping them would inform the judges and give a stage that reasons about the
 source,
which the transcribed-image class argues for.
This is a genuine open question and belongs to the implementation.

Roster,
ordering,
and how selection reports its reasons are implementation
 detail.

## What survives unchanged

Section alignment and slicing,
 the naturalness lane,
 run artifacts and their generation identity,
 and every reader that turns artifacts into a number.

CORRECTED 2026-08-14.
This list also named the deterministic apply gate and the preservation check,
 and both claims were wrong.
Every policy in that gate is anchored to an EDIT bounded by an envelope some
 accepted issue named:
 offsets,
base-text hashes,
no-op detection,
and a preservation rule that lets
 quoted defect text disappear and nothing else.
A whole-slice replacement has no envelope,
and faking one that spans the slice
 fails in both directions:
 with no licensed quotes the preservation rule rejects nearly every legitimate
 translation,
while licensing the whole envelope makes it vacuous.
What that gate enforced still has to be enforced,
by a validator built for whole
 candidates rather than for edits,
which is the invalid-candidate re-ask.
Semantic preservation does not survive in deterministic form at all:
 whether wording only the existing translation carries is a correct archival
 addition needs source-aware judging,
and the incumbent standing as a candidate
 is not an enforcement mechanism for it.

The accumulation running under the repair-only shape keeps its value until the
 new one lands:
 probe calibration and prober disagreement,
 are questions about judging damage,
 and this shape needs judges more than the last one did.
