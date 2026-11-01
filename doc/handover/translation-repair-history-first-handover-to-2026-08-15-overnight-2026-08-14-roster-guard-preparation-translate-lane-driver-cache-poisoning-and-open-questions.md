# Translation repair history, first handover document: overnight of 2026-08-14 into 15 with the pick-up note, the roster guard, shared preparation, the translate lane driver and alignment guard, the poisonable cache and the open questions

The pick-up note of 2026-08-15 that sent the reader to the open decisions,
and the first half of the overnight session:
the roster guard rewritten to measure available weight,
authorship merged for identical candidates,
shared document preparation,
lane-neutral splicing,
the translate lane driver,
and the discovery that an aborted run cached slices nobody examined.
It ends with the questions that blocked the next step.
The census correction that its opening summary mentions is not a passage of this file or of any other file of this record under that name;
the census of 2026-08-13 and its corrections to earlier claims are in `translation-repair-history-2026-08-12-to-15-overnight-2026-08-13-census-of-unread-signals-and-open-decisions.md`.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Immediate next steps

PICK UP HERE (2026-08-15,
after the overnight session).

READ FIRST,
if you are the user coming back to this:
`doc/planning/translation-repair-open-decisions.md`.
It holds the six questions that need your answer,
each with options,
pros,
cons,
a ranking and the measurements taken to make the question answerable.
Question 5 was raised by the bench itself and is the one that blocks wiring.
Everything below is the state those questions sit in.

## Summary of the overnight session of 2026-08-14 into 15

THE SHORT VERSION OF THE NIGHT,
if you read one paragraph:
 both lanes now run over one preparation and neither is chosen over the other,
 which is the last thing that could be built without your answer to Question 5;
 the reassembled document is checked for broken footnotes in both lanes and a
 replacement that breaks one is withdrawn;
 an aborted run can no longer cache slices nobody examined;
 and the lane's budget is measured from calls already bought,
with the audit
 saying plainly which slices the bench never sampled.
Three defects found by review were fixed and are listed under decisions 14 and
15 and in the census correction below;
 one of them,
a footnote label folded differently by the parser and by our own
 scan,
would have silently thrown away every repair in any document a model
 wrote a word-labelled footnote into.

## Roster guard, merged authorship and the roster-width bench

WHAT CHANGED OVERNIGHT,
2026-08-14 into 15:

-   THE ROSTER GUARD NOW MATCHES YOUR RULING (`285af2867`,
    `#91` guard limb).
    It still required two judges outside the producer set,
    so a roster where
    every model produces threw before any model was asked,
    and the bench below
    could not have run past four producers.
    It now refuses only rosters that could not decide a round however they
    voted,
    measured as available weight rather than as a count of seats.
-   IDENTICAL CANDIDATES MERGE THEIR AUTHORS (`8709e83aa`).
    The editor lane kept
    only the first proposer of identical text and the naturalness lane
    deduplicated nothing,
    so a model could vote at FULL weight for its own words
    whenever another model wrote them first.
    Found by an external review of the
    guard change.
-   SELECTION IS AUDITABLE (`8709e83aa`,
    `e8e5bdfbc`).
    Every ballot carries
    whether it was a self-vote,
    every candidate carries what it drew,
    a
    selection carries the position it won at,
    and the translate stage records
    the rotated slate the judges saw.
    Before this a stored ballot naming
    candidate 2 could not be joined to any text:
    the slate is rotated per slice
    and nothing recorded the order.
-   THE ROSTER-WIDTH BENCH EXISTS AND IS RUNNING (`6602831ee`).
    `mise run //package/module/translation-repair:roster-bench -- 10` (moved on 2026-10-18 to
    `//package/translation-repair/bench:roster-bench`) over ten
    stratified slices at every width from two to six,
    width inner so each width
    meets the same provider weather,
    one width run twice for a run-to-run band.
    Rows land incrementally in
    `~/temp/agent/roster-bench-2026-08-15/roster-bench/rows.json`,
    log beside it.
    It already showed the band is wide:
    the same slice at the same width judged
    and replaced on one pass and declined for indecision on the next.
-   TWO MEASUREMENT ERRORS OF MINE WERE STRUCK,
    both in the questions document:
    the transcription class is enumerated now rather than estimated,
    and the
    claim that no image marker exists in the markdown was a bad search.

## Guard corrections, shared preparation and calibrated mismatch threshold

CONTINUED THROUGH THE NIGHT,
2026-08-15 early hours:

-   THE ROSTER GUARD WAS CORRECTED A SECOND TIME (`9e43d5afc`).
    My first weight
    rule measured what a candidate would draw if EVERY producer had a stake in
    it,
    and refused three authors judging only each other.
    That bench decides
    comfortably:
    a candidate one of them wrote draws half a vote from its author
    and a full one from each of the other two.
    Capacity is now measured over the
    most favourable candidate,
    which is the question a guard refusing rosters
    that could not decide HOWEVER they voted has to ask.
    The narrow case it was
    built for,
    one producer judged by itself and one other model at 1.5,
    still
    refuses.
    Found by an external review.
-   A SELECTION ROUND REFUSES A REPEATED JUDGE before spending a call
    (`c5444423b`).
    The stage guard already refused that,
    but `selectPerEnvelope`
    and `selectChunkPatch` are reachable without the stage,
    and two exchanges to
    one model are two ballots from one opinion,
    which reaches the minimum weight
    alone.
-   DOCUMENT PREPARATION IS SHARED (`610ea11b9` splice,
    preparation commit
    before it).
    Parsing,
    identity,
    alignment,
    subdivision and governance now
    live in `prepareDocumentPair`,
    taking no client,
    roster,
    config,
    signal or
    cache.
    Two lanes slicing separately would drift the moment either changed a
    budget,
    and each would still report slices that looked right on its own.
-   SPLICING IS LANE-NEUTRAL AND CAN FILL A GAP (`610ea11b9`).
    It consumes
    replacements rather than repair outcomes,
    inserts into zero-length spans,
    resolves indexes before sorting,
    refuses duplicates,
    and orders several
    insertions at one offset by slice index.
    Three defects went with it,
    each of
    which produced wrong text rather than an error.
-   THE MISMATCH THRESHOLD IS CALIBRATED (`#90`).
    Over all 1260 two-sided corpus
    slices the incumbent-to-source character ratio runs p50 2.95,
    p90 4.10,
    p95
    5.36,
    p99 23.78,
    max 521.9.
    So 3:1 is normal and a cut around 10 flags 25
    slices,
    including every known damage case.
    The bench case that started this,
    `windward0032#10` at 3 characters against 226,
    sits in the worst six.
-   THE BENCH DRAW AND WIDTH SWEEP ARE TESTED (`bench-draw.ts`).
    Both decide
    what a width comparison measures and both failed silently before.

## Translate lane driver, alignment guard and the roster-width bench result

THEN THE LANE GOT ITS DRIVER,
2026-08-15 pre-dawn:

-   ASSEMBLY REFUSES TWO SLICES CARRYING ONE INDEX (`e66a18749`),
    which is a
    wrong-text failure rather than an error.
-   THE ALIGNMENT GUARD LANDED,
    CALIBRATED (`c5e781956`,
    `d319f329e`).
    A
    replacement is refused when the incumbent is at least 128 code points and
    more than 16 times its source.
    Translation still runs and the judges'
    evidence is kept,
    so the record distinguishes "the judges kept the
    incumbent" from "the judges wanted a replacement and the guard refused".
    Calibrated over all 1260 two-sided corpus slices;
    it refuses 16,
    and every
    one of those 16 was read.
-   THE SLICE CACHE IS GENERIC OVER ITS LANE'S VALUE (`003e09f9d`) AND EACH LANE
    OWNS A NAMESPACE (`108329dc2`):
    a file prefix plus its own generation
    marker.
    One shared marker with a directory-wide delete,
    which is what this
    replaced,
    meant a translate change threw away every settled repair slice in
    the corpus and nothing reported the loss.
-   THE TRANSLATE DOCUMENT DRIVER EXISTS (`e2deabf4d`).
    One prepared pair in,
    every slice visited unconditionally,
    one settled record per slice,
    the
    document reassembled from per-slice decisions,
    its own schema version and
    its own cache namespace.
-   THE BENCH FINISHED AND RAISED QUESTION 5 (`7fe82a159`).
    Width does not
    measurably change agreement at n=10,
    but the lane replaced the archive's
    English in 44 of 60 rounds,
    and that is a decision rather than a finding.

## Slice cache poisoned by aborted runs

AND THEN THE CACHE TURNED OUT TO BE POISONABLE,
which is why no pass should
start before these landed:

-   AN ABORTED RUN WAS CACHING SLICES IT NEVER BOUGHT (`1918c67a1`,
    `d89550076`).
    An abort reaches every exchange as a torn-down stream,
    the
    round records each as a LOST VOICE by design,
    and a stage that heard nothing
    keeps the incumbent and returns an ordinary settled record.
    Both drivers
    wrote that to the cache,
    so an entry stopped at its deadline recorded its
    unexamined tail as decided,
    and every later attempt RESUMED it.
    Both drivers
    now check the signal before buying a slice and again before persisting one,
    and report the abort by the caller's own reason rather than by whichever
    exchange happened to surface.
-   A SLICE NOBODY EXAMINED IS NO LONGER CACHED (same commits).
    Zero translators
    heard,
    or zero critics heard,
    settles in memory for this run and is left out
    of the cache,
    so the next attempt asks again instead of resuming an outage
    as a verdict.
-   A ROUND NOW RAISES A CALLER ABORT THAT LANDS AFTER QUORUM (`182280185`),
    which its own comment already promised.
    Measured on the driver test:
    an
    abort inside the second slice previously let the whole judge roster fan out
    afterwards,
    12 judge calls attempted where 6 were owed.
-   THE REPAIR DRIVER NOW REFUSES A CACHED OUTCOME NAMING ANOTHER SLICE
    (`d89550076`),
    which the translate driver already did.
-   MEASURED,
    NOT ASSUMED:
    the 150 repair slices currently cached carry
    `heardCritics` 3,
    4,
    5 and 6 (1,
    3,
    92 and 54 slices),
    so NONE of them is
    poisoned and nothing has to be invalidated.
    A cache written before these
    fixes could only be resumed under its own pipeline digest anyway,
    and that
    digest has since moved.

## Questions that blocked the next step

WHAT NEEDS YOU,
in the order it blocks work:

1.   HOW WIDE the producing roles should be,
     and how to WRITE that width.
    You decided to widen them and separately ruled that provider counts must
    not be hardcoded,
     since the offering changes often.
    No structural bound survives to derive a number from:
     selection works with
    every model producing,
     because the discount applies to a judge's ballot for
    its OWN candidate only,
     and checker disjointness is being replaced by a
    weighting under `#91`.
    What moves with width is cost,
     ballot dilution and coverage,
     which is a
    tradeoff rather than an arithmetic the code can settle,
     so the number needs
    to come from you or from a measurement nobody has taken.
2.   A policy answer for the TRANSCRIBED-IMAGE class,
     unchanged and now urgent.
    Chinese pages hold letters as images;
     English pages transcribe and translate
    them.
    The class is now ENUMERATED,
     replacing the older "roughly 31 thousand
    characters,
     6 entries verified",
     which reproduces from no measurement I can
    take:
     8 target-only blockquotes over 1000 characters,
     across 6 entries,
    15299 characters,
     sitting inside a wider target-only population of 132
    blocks and 44731 characters that also holds translator apparatus and
    alignment slop.
    Exactly one transcription is invisible to that structural test,
     `shihai4h`
    at 102 source characters against 1665,
     because it was transcribed INTO a
    quote the Chinese also carries.
    A source-only translator has no source for that text and a source-only judge
    cannot tell dropping it from correctly omitting it.
    Your standing ruling,
     keep accurate translator additions,
     says it must
    survive;
     nothing in the lane yet makes it survive.
3.   `#66` and `#68`,
     human grading,
     unchanged and still the gate on probe
    calibration.
