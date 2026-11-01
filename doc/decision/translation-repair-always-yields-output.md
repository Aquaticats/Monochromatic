# The pipeline always yields output, whatever the input

Restated 2026-08-13 after the agent offered "block the entry" as an option for
 the aligner floor.
That option should never have been on the list,
and the
 reason it was is worth recording alongside the rule.

## The rule

The user's standing principle:
the pipeline should yield a good translation at
 the end even when the translation fed in makes no sense.
No stage may refuse to
 produce output for an entry.

That is not a preference about ergonomics.
A stage that declines an entry
 removes it from every measurement silently,
and the entry looks settled while
 nothing happened,
which is the failure shape this project has been caught by
 repeatedly.

## What it decides for the aligner

`doc/decision/translation-repair-unpairable-section.md` already ratified REPORT:
 an unpairable section gets no critic work and the refusal is recorded.
Those
 two decisions together fully determine the floor,
with nothing left to ask:

-   A section the aligner refuses passes through UNREPAIRED.
    Its text reaches
    the output unchanged.
-   A document where the aligner forces no pairing at all still settles,
    with
    its original translation as the output and a finding per refused section.
-   Blocking the entry is not available.
    Neither is aborting the run.

The asymmetry that makes this right:
leaving text alone cannot damage it,
while
 a guessed pairing feeds critics the wrong original,
manufactures issues,
and
 the repairs that follow damage text that was correct.
Doing nothing is a
 weaker outcome than repairing well and a far better one than repairing wrongly.

## What it does NOT settle

An unrepaired section is a floor,
not a goal.
The better answer for a partial
 translation like `XIEPT2`,
whose target bodies are empty,
is to TRANSLATE the
 missing content rather than to pass it through.
That is the translate stage
 the translate-first re-design is about,
and it is the reason ROUTE outranked REPORT in the original
 ranking.
REPORT is what is available before that stage exists.

## How the question got asked anyway

The agent had the principle recorded and the REPORT decision ratified,
and still
 offered blocking as one of three options.
Both premises were in front of it;
 the conclusion simply was not drawn.
Recorded because the failure is not
 ignorance of a rule but not applying a rule already known,
which no amount of
 further documentation fixes on its own.
