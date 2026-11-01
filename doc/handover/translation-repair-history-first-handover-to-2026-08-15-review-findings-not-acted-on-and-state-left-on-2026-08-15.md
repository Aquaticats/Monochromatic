# Translation repair history, first handover document: review findings not acted on, and the state left on 2026-08-15

The review findings set aside,
each with the measurement that says why it can wait,
the passes stopped on purpose,
and what the translate driver landed and left undone.
The reviews its first paragraph refers to are the three reviews of the morning of 2026-08-15 recorded in `translation-repair-history-first-handover-to-2026-08-15-combined-lane-driver-lane-comparison-and-assembly-invariants-2026-08-15.md`.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Review findings deliberately not acted on

WHAT THE SAME REVIEWS RAISED AND I DID NOT ACT ON,
each with the measurement
that says why it can wait.
None is a judgement call left open;
each is real and
currently unreachable on this corpus,
so acting would be building against
nothing:

-   THE RAW SCANNER AND THE PARSER STILL DISAGREE ABOUT ESCAPES.
    `\[^1]` is not
    a footnote to the parser and is a hit to the scanner,
    which stops on `[` and
    `^` without asking whether either was escaped.
    Measured over all 279 corpus
    markdown files:
    ZERO escaped `\[^` sequences.
    A false hit inflates a mention
    count on both sides of a comparison,
    so it moves attribution only when a
    replacement adds or removes one.
-   AND ABOUT WHITESPACE IN A LABEL.
    The parser accepts it and collapses it;
    `GFM_IDENTIFIER_STOPPERS` rejects the marker outright,
    so such a footnote
    yields no mention key and the guard would withdraw everything.
    Zero corpus
    identifiers carry whitespace;
    all 209 are digits.
    Fixing it means teaching
    the scanner the parser's label rules,
    which changes what
    `buildFootnoteGraph` reports as an unresolved reference,
    so it is a change to
    the graph rather than to a key.
-   AND ABOUT WHAT MAKES A FULL-WIDTH MARKER A DEFINITION.
    `buildFootnoteGraph`
    calls it a definition when it opens a block;
    `footnoteIdentifiers` requires
    the `：` after it.
    Measured:
    ZERO `〔N〕` markers in any of the 279 files,
    so
    this convention has no corpus instances at all.
    One shared classifier is the
    fix if that ever changes.
-   `spliceSlices` DOES NOT VALIDATE THAT SPANS DO NOT OVERLAP.
    It cannot today:
    every replacement it receives is keyed to a slice from one
    `prepareDocumentPair`,
    and those spans partition the document by
    construction.
    Assembly does refuse two replacements naming one index
    (`e66a18749`).
    Recorded because the construction argument is the only thing
    holding it,
    and it lives in another file.
-   DEFINITION HITS CARRY NO OFFSET,
    while reference hits do.
    Nothing needs one
    yet;
    a future finding that wanted to point at a definition would.
-   `PreparedDocumentPair` COULD CARRY THE PARSED INCUMBENT rather than having
    the guard reparse it.
    A performance note,
    not a correctness one,
    and the
    guard reparses per round anyway.
-   A SLICE KEY COULD IN PRINCIPLE LAND IN ANOTHER LANE'S NAMESPACE.
    `sliceFileName` writes `${prefix}${key}.json` and `belongsToNamespace`
    defines the repair lane as everything NOT starting with a claimed prefix,
    so
    a repair key beginning `translate.` would be written by one lane and adopted
    by the other.
    Measured on what is actually on disk:
    every slice file in
    every pass directory is a 64-character hex digest,
    so no key can carry a dot
    at all,
    and the key derivation is what holds it.
    Recorded rather than
    guarded for the same reason as the footnote-escape items:
    the population is
    empty,
    and a guard here would be built against nothing.
    A key scheme that
    stopped being a hex digest is what makes it real.
-   AN EMPTY CRITIC ROSTER SETTLES A DOCUMENT INSTEAD OF REFUSING IT,
    found by
    fault injection while proving the guard tests fail (`#93`).
    Configuring zero
    critic models runs the repair lane end to end and returns an UNCHANGED
    document with `status` settled and zero exchanges bought:
    no throw,
    no
    finding,
    nothing a later reader can tell apart from a page that needed no
    repair.
    The quiet path is deliberate for OUTAGES,
    where a stage with no
    usable voices must settle rather than poison the slice cache with an answer
    it did not get.
    A deterministic empty roster is a CONFIGURATION error,
    and
    the two are indistinguishable downstream.
    A corpus pass under that
    misconfiguration writes a directory of vacuous settled artifacts and looks
    like a clean run.
    NOT BUILT ON PURPOSE:
    where the refusal belongs is a
    design choice (lane entry,
    `runDocumentLanes`,
    or the corpus-pass boundary
    that builds the roster),
    and so is whether an empty ADJUDICATOR,
    EDITOR,
    CHECKER or REFINER roster deserves the same treatment,
    since silence means
    different things at those stages.
    `#93` carries the probe as evidence.

## Passes stopped on purpose and the wiring still to do

STATE:
NO PASS IS RUNNING,
deliberately.
`pass16` was stopped on 2026-08-14 with zero artifacts settled,
on the user's
ruling that there is no cost to stopping a to-be-discarded entry mid-flight:
the pipeline shape is decided and anything accumulating under the repair-only
shape is output the new shape replaces.
The driver EXISTS and is tested;
nothing calls it from the corpus pass.
A long run is blocked on two things rather than one:
Question 5,
which decides
what the pass does with a replacement,
and the wiring itself,
which is shaped
differently under each of that question's answers.
Do not restart accumulation before both,
or the same budget buys the same
discardable entries again.
The stopped pass left its `pass.lock` behind in
`node_modules/.monochromatic/translation-repair-runs-pass16`;
the next pass takes it over as stale and says so with a `LOCK taking over` line,
which is the first live exercise of that path.

## Translate stage and driver landed and what remains of the combined driver

WHAT LANDED,
and what it does not yet do:
`runTranslateStage` (`8e27504f1`,
tests `9411e833d`) renders one slice from its
original through several translators,
stands the archive's own translation among
them as one candidate,
and lets judges choose.
`translateDocument` (`e2deabf4d`) now drives it over a whole prepared pair,
and
that is the part `#89` was blocked on.
It could never simply replace `runEditorStage` at the old call site,
because
`repairChunk` returns before reaching the editor on exactly the slices
translation is meant to recover:
non-translation votes standing,
critics raising
no claims,
the panel cutting no envelopes.
The driver visits every slice instead.
WHAT REMAINS OF `#89` (updated 2026-08-15,
the combined driver now exists as
`runDocumentLanes`):
`corpus-pass.ts` opens no translate cache,
writes no
translate fields into the artifact,
and has no deadline accounting that keeps a
capped run from writing a settled artifact.
The preparation half is done:
`repairPreparedDocument` takes a prepared pair,
which is what a combined driver needs from this side.
The combined driver is Question 5 neutral only if it returns both lanes' outputs
without arbitrating between them,
so build it that way;
the `corpus-pass.ts` wiring is not neutral and waits on that answer.
