# Translation repair history 2026-08-24 to 2026-08-25: what the test suite reaches, measured

The census of source modules directly exercised,
reached only through an importer and reached by nothing,
the three unreached modules that were the finding,
and the counts of work markers and lint suppressions.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## What the suite actually reaches, measured (2026-08-24)

Run after `#204` closed,
to answer the package-completeness rule with a number instead of a feeling.

Over all 507 source modules of the package:

-   386 are DIRECTLY exercised:
    they have a sibling `.unit.test.ts`,
    or one of their exports is named somewhere in the suite.
-   53 are reached only through an exercised importer.
-   40 are reached by NOTHING.

Of those 40,
37 are `corpus-run/` operator CLIs and probes.
Each ends in a top-level entry call and is exercised by being run,
which is a different kind of evidence and not one the suite can give.

The other three were the finding:

-   `repair-blocked-exit.ts`,
    dead since `#110`,
    deleted as `#207`.
-   `producer-standing-report.ts`,
    live but reachable only through calibration CLIs no test drives.
    It renders the share of disinterested ballots each model won,
    which is what `#199` seated the writers on.
    Covered now,
    and the ordering rule is GFP-proven:
    treating an UNJUDGED model as a zero share fails the case,
    because a model with no evidence and a model measured at zero are different findings.
-   `coverage-candidates.ts`,
    same shape.
    Covered now at both scales,
    and GFP-proven:
    dropping the block scale from the list fails the case.

### Two other layers, while looking

ZERO real TODO,
FIXME,
HACK or deprecation markers in the package.
The two apparent hits are a `U+XXXX` doc example
and a case-insensitive match inside the identifier `toDocumentNode`.

66 lint suppressions,
of which ZERO are bare:
every one carries a ` -- ` justification,
which is what `LN5` asks for.
