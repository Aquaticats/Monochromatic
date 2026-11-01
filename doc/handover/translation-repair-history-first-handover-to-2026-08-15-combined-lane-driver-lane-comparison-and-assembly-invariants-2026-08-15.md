# Translation repair history, first handover document: combined lane driver, lane comparison and assembly invariants of 2026-08-15

One list of items about the driver that runs both lanes over one preparation without choosing between them,
the per-slice comparison of what each lane decided,
the review findings recorded as tasks,
and the assembly invariants added after them.
Its first paragraph continues the translate lane driver recorded in `translation-repair-history-first-handover-to-2026-08-15-overnight-2026-08-14-roster-guard-preparation-translate-lane-driver-cache-poisoning-and-open-questions.md`
and the footnote guard recorded in `translation-repair-history-first-handover-to-2026-08-15-morning-2026-08-15-footnote-guard-and-its-review.md`.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## One driver for both lanes that chooses between neither

AND THEN BOTH LANES GOT ONE DRIVER,
which is the last Question 5 neutral piece
of `#89`:

-   `runDocumentLanes` (`document-lanes.ts`) takes one prepared pair,
    runs both
    lanes over it,
    and returns both documents.
    It ARBITRATES NOTHING:
    no winner,
    no preferred lane,
    no merged text,
    because choosing between them is Question
    5 and a driver that chose would answer it invisibly for every later count.
-   SEQUENTIAL,
    REPAIR FIRST.
    Concurrency buys nothing:
    the quota spent is the
    same and both lanes already serialize their own slices for provider-capacity
    reasons.
    Repair goes first because its naturalness phase settles AFTER the
    slice loop and nothing persists what that phase produced,
    while the translate
    lane caches every slice as it finishes;
    under a deadline that cuts the entry,
    running the uncheckpointed phase first loses less of what was bought.
    That
    reasoning is an external reviewer's and it is recorded in the driver.
-   NO ABORT CHECK BETWEEN THE LANES,
    deliberately.
    Both drivers let a fully
    cached lane finish after an abort,
    since resuming buys nothing,
    and a gate
    there would refuse that.
-   ALIGNMENT FINDINGS ONCE,
    at the top level:
    they belong to the preparation
    both lanes shared,
    so counting them per lane would count one defect in the
    archive twice.
    The repair result still repeats them inside its own findings,
    which is that lane's existing contract and was left alone.
-   `repairPreparedDocument` NOW TAKES A PARENT LOGGER,
    defaulting to the
    pipeline root,
    so both lanes read as one entry rather than as two runs.
-   THE ABORT CASE COULD NOT PIN THE CONTRACT,
    and finding that out is the
    reason there are two failure cases rather than one.
    A driver that caught the
    repair lane's failure and ran translate anyway still passes an ABORT test,
    because the translate lane refuses on its own once the signal is aborted:
    two different mechanisms produce the same observation.
    The case that pins it
    hands the repair lane a cache whose resumed outcomes name other slices,
    which the driver refuses with nothing aborted anywhere.
    Demonstrated by
    swallowing the repair failure in a scratch build:
    that case fails at exactly
    the assertion that encodes the contract,
    and the abort case passes.
-   BOTH LANE RESULTS NOW NAME THEIR SLICES,
    not just count them:
    `shippedChunkIndices` and `withdrawnChunkIndices` on each.
    Read off the
    guard's own surviving replacements,
    because a per-slice record says what
    that slice CHOSE and the document may carry something else;
    a comparison
    built from the records would credit a lane with slices it did not change.
    The repair artifact records both,
    so the withdraw rate is countable over a
    settled directory.
    Artifacts from before 2026-08-15 lack the fields,
    and a
    reader must treat that as unknown rather than as empty.

## Per-slice lane comparison, the review findings recorded as tasks and the assembly invariants

-   BOTH LANES NOW REPORT WHAT THEY DECIDED FOR EVERY SLICE,
    beside the
    archive's own wording,
    and a pure function compares the two documents slice
    by slice.
    `LaneSliceText` (`lane-slice-text.ts`) is one entry per PREPARED
    slice:
    index,
    incumbent,
    accepted.
    `compareDocumentLanes`
    (`lane-comparison.ts`) joins two lane results on the index and names each
    slice `archive-stands`,
    `repair-only`,
    `translate-only`,
    `both-agree` or
    `both-differ`.
-   THE SHIPPED FLAG IS DELIBERATELY NOT ON THE SLICE RECORD,
    which is the whole
    design.
    Whether a slice shipped is decided by an assembly guard reading the
    WHOLE document,
    and the same slice can ship in one run and be withdrawn in
    the next when a neighbouring replacement changes.
    Membership in
    `shippedChunkIndices` is that fact.
    Putting it on a per-slice record would
    put a per-run verdict on a cacheable record,
    which is the defect class the
    last three days were spent removing;
    the translate lane's slice records are
    literally its cache values,
    so a resumed slice would have served a stale
    verdict.
    The comparison reads the index sets and derives what each document
    carries:
    accepted where it shipped,
    incumbent where it did not.
-   AND FOR THE SAME REASON THE WORDINGS ARE BUILT AT THE DOCUMENT LEVEL,
    from
    `prepared.slices`,
    rather than stored per slice.
    An incumbent belongs to a
    PREPARATION;
    a slice resumed from an earlier run would otherwise report the
    wording that preparation had then.
    Neither lane's cache schema changed,
    so
    the 150 settled repair slices on disk survive this.
-   COVERAGE IS CHECKED RATHER THAN ASSUMED.
    `buildLaneSliceTexts` throws
    `LaneSliceCoverageError` when a lane leaves a prepared slice undecided or
    names a slice the preparation never produced,
    and `compareDocumentLanes`
    throws `LaneComparisonError` on differing slice counts,
    a missing slice,
    or
    two lanes disagreeing about one slice's incumbent.
    All three mean the two
    sides came from different preparations,
    which no later reader could detect:
    the rows would line up and describe different passages.
-   THE BLOCKED REPAIR EXIT NOW CARRIES WORDINGS TOO,
    which closes a consumer
    trap a review had flagged:
    that exit returns both index sets empty while
    every issue record reads `withdrawn`.
    Read with the wordings it now states
    "this lane had repairs and the document carries none of them",
    which two
    empty sets alone could not say.
-   AND THE FIRST VERSION OF IT WAS WRONG,
    caught by an external review before
    it was committed.
    `repairPreparedDocument` runs the dominance check INSIDE
    the slice loop and returns at the earliest crossing,
    so the blocked exit
    holds FEWER outcomes than prepared slices.
    A builder that demanded a
    decision per slice threw there,
    which would have turned a documented
    blocked result into a crash.
    `acceptedText` is now `string | null`,
    null
    meaning the lane never reached that slice,
    and the builder takes an
    explicit policy:
    `refuse` where the lane visits everything,
    `not-evaluated`
    only where it stops early by design.
    The tests did not catch it because no
    test drives the blocked exit with a partial outcome list,
    which is itself
    worth fixing.
-   THREE SOL REVIEWS LANDED ON 2026-08-15 MORNING and their findings are
    recorded as tasks rather than left in the transcript:
    `#93` (empty-roster
    placement,
    now answered:
    the check belongs in `repairPreparedDocument`
    because `runDocumentLanes` bypasses `repairTranslation` entirely),
    `#94`
    (index contracts claim sortedness,
    uniqueness,
    disjointness and range and
    enforce none),
    `#95` (a cached slice can claim a change it did not make),
    `#96` (the artifact is repair-only,
    unversioned,
    and cannot express
    unknown),
    `#97` (a checker verdict may describe pre-refinement text),
    `#98`
    (equal section counts skip the aligner entirely),
    `#99` (`chunkIndex` means
    three different things),
    `#100` (one-sided slicing:
    the design answers),
    `#101` (splice ordering and separator ownership),
    `#102` (what remains of
    the delivery ledger).
    Read `#98`,
    `#99` and `#100` together:
    they are one
    change to how a slice gets its identity and its span.
-   AND BOTH LANES NOW REFUSE A CHANGE THE DOCUMENT DOES NOT CARRY
    (`assembly-invariant.ts`).
    `assertReplacementsChange` runs before the
    footnote guard and refuses a replacement that repeats its slice's incumbent,
    or names a slice the preparation never produced.
    `assertDocumentChangeAgrees` runs after assembly and refuses a returned
    document that disagrees with its own change set in either direction.
    And
    `orderedChangeSets` checks both index sets against each other,
    integers,
    in
    range,
    no repeats,
    disjoint,
    and returns BOTH ascending:
    the withdrawn one
    never was sorted,
    so two lanes compared slice by slice were being read from
    lists ordered by different rules.
    `RepairTranslationResult` also carries
    `sliceCount` now,
    which the translate side always had and which is what a
    standalone consumer needs to range-check an index at all.
-   THE REACHABLE WAY IN WAS THE SLICE CACHE,
    which is why these are assertions
    rather than comments.
    A cached record is trusted on its chunk index alone,
    so one claiming a change while holding the archive's own wording reached the
    guard,
    survived it untouched,
    and landed in the shipped set beside a
    document nobody changed.
    A truncated write that still parses,
    or a slicing
    that moved while the pipeline digest did not,
    both produce that record.
-   THEY THROW,
    and that is not obviously right:
    both run after model calls
    costing minutes and quota,
    inside a pass that settles one entry at a time.
    A throw loses the entry's unpersisted work;
    a finding lets a wrong count
    settle into an artifact.
    Nothing is at risk today because no pass is
    running.
    `#95` records the open question and the measurement that would
    settle it,
    which is whether `ChunkRepairOutcome.changed` can be true while
    the repaired text equals the incumbent:
    the cached outcomes on disk do not
    carry the incumbent,
    so answering it needs a re-preparation of each entry,
    which costs no quota.
-   AND THE FIRST OF THE REVIEW FINDINGS WAS MEASURED RATHER THAN QUEUED.
    `#98`
    says the aligner's mirrored fast path skips alignment whenever the section
    counts match,
    which is `#71`'s defect arriving by an uncovered path.
    Over
    the pinned corpus:
    85 of 92 pairs take that fast path,
    and the forced
    aligner would pair NONE of them differently.
    The positive control ran first
    and shows the probe can see the defect,
    on invented headings with one
    section dropped and one added.
    But the same control also shows the forced
    aligner pairing three WHOLLY UNRELATED headings by position without a single
    refusal,
    and on this corpus every source heading is Chinese against an
    English target,
    so it has no signal to work from and degrades to exactly the
    positional pairing the fast path already does.
    THE ZERO IS NOT EVIDENCE THE
    85 PAIRINGS ARE RIGHT;
    it says running the aligner on them would change
    nothing,
    because the aligner is blind here.
    That is the same blindness `#71`
    named and the instrument weakness `#74` is about,
    so `#98` alone would land
    a change that provably alters no pairing while looking like a fix.
-   WHAT IT STILL DOES NOT DO.
    Nothing CALLS `compareDocumentLanes` yet:
    the
    corpus pass writes a repair-only artifact,
    and wiring it for two lanes is
    the part Question 5 shapes.
    The settled artifact also records no per-slice
    wording,
    so a grader reading a settled directory still cannot see what a
    lane decided for a slice it did not ship.
    That is the remaining `#89` item
    and the same work as the `sliceSelections` artifact field;
    the contract it
    needed now exists.
