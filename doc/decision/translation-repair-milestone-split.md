# Milestone three splits into detection and repair safety

Ratified 2026-08-13 by the user,
answering an open question recorded in
 `doc/planning/translation-repair-milestone-replan.md`.

## The decision

Milestone three becomes two gates rather than one:

-   DETECTION precision,
    unchanged,
    at the 0.9 bar.
    That bar is the user's and
    stays.
-   REPAIR SAFETY,
    gated separately.
    Safety means an edit did not delete or
    invent content nobody complained about.

Repair QUALITY is measured but not gated.
Rendering 变故 as "misfortune" is a
 real defect and it is not a reason to refuse to ship.

## The measurement that settled it

From the round-three repair sheet,
over the 34 items the user graded as real
 defects,
the five failures separate cleanly and unevenly:

-   SAFETY,
    four:
    item 2 deleted a clause the source supports,
    item 21 deleted a
    contributor's name,
    item 37 changed "reminiscing" to "pleading",
    and item 34
    never addressed its claim at all.
-   QUALITY,
    one:
    item 1 fixed the semantics and left English nobody writes.

One bar covering both would let a deleted name and an awkward register weigh the
 same,
which is exactly what made the sheet ungradeable in the first place:
the
 user stopped,
saying "the repairs are currently too broken to grade".

## What already implements the safety half

`checkPreservation`,
wired into `applyPatchOperations`.
It is deterministic and
 model-free,
and it rejects an edit that drops content no accepted issue quoted.
 Calibrated on these same 50 repairs:
it catches the deletions and rejects none
 of the 29 repairs graded sound.

It does NOT catch item 37,
because rewording is not deletion and the sentence is
 still represented.
Safety gating is therefore partly deterministic today and
 not yet complete,
which is a known gap rather than an oversight.
