# Translation repair history, first handover document: morning of 2026-08-15 with the footnote guard on the reassembled document and its review

The guard that checks the reassembled document for broken footnotes in both lanes,
the measured lane budget,
and the review that found a withdrawn rewrite still recorded as shipped and footnote labels folded differently by the parser.
Its first paragraph continues the night recorded in `translation-repair-history-first-handover-to-2026-08-15-overnight-2026-08-14-roster-guard-preparation-translate-lane-driver-cache-poisoning-and-open-questions.md`.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Footnote guard on the reassembled document and the lane budget audit

AND THEN THE FIRST HALF OF `#92`,
the part that needs no quota:

-   THE REASSEMBLED DOCUMENT IS NOW CHECKED (`b77cff67b`,
    `48e20c20d`).
    A
    footnote is a relation BETWEEN slices,
    and selection settles each slice
    alone,
    so a candidate that drops or renumbers a marker validates perfectly
    inside its own slice and breaks the document.
    The lane now splices,
    parses
    the result,
    and diffs its footnote findings against the archive's,
    so a
    defect the archive already carried is neither blamed on the lane nor
    repaired by it.
    Replacements that broke the graph are WITHDRAWN.
-   IT ITERATES rather than checking once,
    and the case is pinned:
    one slice
    renumbers `[^1]` to `[^2]` while another supplies the `[^2]` definition,
    so
    withdrawing the first is what orphans the second.
    Demonstrated by capping
    the loop at one round and watching that case fail.
-   ATTRIBUTION IS BY ROLE,
    after an external review found the first key too
    coarse:
    a slice that turns `[^1]: the note` into prose saying `see[^1]`
    mentions the identifier exactly as often as before.
-   A DEFECT NOBODY CAN BE BLAMED FOR WITHDRAWS EVERYTHING.
    A stray comment
    opener masks markers document-wide,
    since masking runs over the whole body
    before parsing,
    and no slice's own mention count moves.
    The archive text is
    the one thing certain to parse as it did before.
-   THE LANE'S BUDGET IS MEASURED,
    from calls the bench had already bought,
    in
    `doc/audit/translation-repair-lane-budget.md`.
    Headline:
    594 of 602
    exchanges returned ok and the only failures were the straggler cut,
    both on
    the GLM models;
    a slice costs 10.2 calls and 34567 tokens at width 4;
    one
    corpus pass is 43.6M tokens at that width;
    and against the three-hour entry
    cap every entry fits at the median slice time,
    with the two largest falling
    out at p90.
    So the cap is not the binding constraint it was for repair.
-   MEASURED,
    over all 184 corpus documents at the pinned commit:
    209 GFM
    footnote markers across 45 files,
    and ZERO reference-style link definitions,
    ZERO reference-style link uses,
    ZERO heading-anchor links.
    So footnotes are
    the only cross-slice relation this corpus actually has,
    and the other graphs
    an external review proposed would be built against nothing.
    The
    `〔N〕` convention the parser also supports appears nowhere in this corpus
    either;
    it belongs to the other archive named in `footnote-model.ts`.
-   BOTH LANES ASSEMBLE THROUGH THE GUARD.
    In the repair lane the withdrawal
    also reaches the ISSUE RECORDS:
    an issue whose slice was taken back is
    recorded `withdrawn` and unresolved,
    the same disposition a non-translation
    block already used.
    Crediting it would have overstated precision by exactly
    the repairs no reader saw.
    Both lane guards were demonstrated by disabling
    the guard and watching the cases fail.
-   `repairPreparedDocument` (`b2fba072a`) takes a prepared pair,
    with
    `repairTranslation` the thin entry point that prepares and delegates.
    That
    is what a combined driver needs:
    ONE preparation handed to both lanes.
    Assembly moved to `repair-assemble.ts` and the result types to
    `repair-result.ts` at the line budget;
    `repair-translation.ts` re-exports
    them,
    so callers and the barrel are unchanged.

## Review of the footnote guard

THEN A REVIEW OF THAT GUARD FOUND TWO THINGS IT BROKE OR LEFT BROKEN,
2026-08-15
morning:

-   A WITHDRAWN REWRITE WAS STILL RECORDED AS WHAT SHIPPED (`48dcce7ba`).
    `finalSliceText` was written whenever the naturalness lane rewrote a slice,
    which was correct until the guard could take a rewritten slice back.
    It is
    now written only where the document carries the rewrite.
    The artifact reader
    required that field of every rewritten record,
    so it had to learn the same
    rule in the same commit or refuse to read the run;
    it now requires it exactly
    where a repair SHIPPED and reads it wherever else it was written.
    The sheet
    says the rewrite was taken back instead of fencing an empty block under "the
    slice as actually returned",
    which is what it did before.
-   SHIPMENT IS NOW ASKED OF THE STEP THAT DECIDES IT (`74dafeb3b`).
    A record
    asks whether this slice's replacement is in the returned document,
    which
    dominance,
    the guard,
    and an unchanged selection can each answer no to.
    The
    third term restates `repairReplacements`,
    which emits nothing for an
    unchanged outcome;
    today that term changes no record,
    because
    `accuracyPatchSelected` is set FROM `changed` and the refine lane only ever
    sets `changed` true,
    but the record no longer depends on another file holding
    that invariant.
    `judgeDisposition` takes `repairReachedReader` rather than
    `blocked`,
    which stopped naming what it receives once the guard existed.
-   FOOTNOTE LABELS ARE FOLDED THE WAY THE PARSER FOLDS THEM (`9322cdaba`).
    mdast keys `[^Note]` and `[^note]` alike and hands back the folded spelling;
    the raw scans this guard attributes with saw the source spelling,
    so a
    finding about `note` was looked up in mentions keyed `Note`.
    Measured before
    the fix on a two-replacement fixture:
    no slice could be blamed and BOTH were
    withdrawn,
    including one that touched no footnote.
    Measured after:
    the guilty
    slice alone is reverted and the innocent edit ships.
    All 209 corpus markers
    are numeric,
    where folding is identity,
    so nothing settled is affected.
    `normalizeFootnoteIdentifier` reproduces `normalizeIdentifier(label)
    .toLowerCase()`,
    and its test compares against a real parse rather than
    against a restatement of the rule.
