# Translation repair history 2026-08-12 to 2026-08-15: artifact draws that mixed pipeline generations, the readers that name their generation and the readonly rule findings left

The discovery that artifact draws silently mixed pipeline generations,
the readers that now name the generation they read,
the pool of the current generation,
and the readonly lint findings that remained.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Artifact draws that mixed pipeline generations, the readers that now name their generation, and the readonly rule findings left

### `#426`: every artifact draw silently mixes pipeline generations

Artifacts record the repo commit they ran under,
as `tip`.
NOTHING reads it back.
Six readers glob the artifacts directory with no generation filter.

This is not hypothetical.
`pass13` holds 21 settled entries across three tips,
and tested with `git merge-base --is-ancestor` ALL THREE lack both behaviour
fixes that landed on 08-13,
`fc7912929` at 18:41 and `69b81eeec` at 19:40.
The
last artifact settled at 18:28.
So the pool of entries settled under the current
pipeline is ZERO,
not 21.

`#60`,
`#66` and `#68` all name "entries settled under the current pipeline" as
their input and none can be satisfied from that directory today.
**`#426` must
land before the next draw,
or the draw mixes generations.**
The spread is still
growing:
`pass13` loaded its source at 19:41 and keeps stamping that tip while
`SLICE_CACHE_VERSION` has since gone to 23.

### `#427`: what remains of the readonly rule findings

`prefer-readonly-parameter-types` went 10 to 3 in this package.
The seven that
cleared all had a workspace-owned producer and the `#424` origin naming pointed
at the right edit every time.

The three that remain bottom out in types this workspace does not own,
and
`ForeignBorrowed` does not reach them:
I marked the boundary at both placements,
on a parameter and on a local,
and the diagnostic was byte-identical each time.
Reported rather than worked around,
because three bespoke projections over
`@types/mdast` and `Intl.SegmentData` would satisfy the linter by making the code
worse.

Also recorded there:
with the three extracted effect rules temporarily set to
`error`,
this one package goes from 3 errors to 177,
effectively all
`no-opaque-parameter-effects`,
with `JSON.stringify` a large share,
plus 65
`SemanticBridgeError` warnings carrying bundled stack traces.

### `#426` landed: readers now name the generation they read

`censusByTip` partitions an artifacts directory by the commit each run recorded.
`selectEligible` turns that into what one draw may pool and REFUSES when the
directory spans generations and the caller named neither a required commit nor
deliberate pooling.
`resolvePool` reads the policy from the environment and
prints the census,
so a rate cannot be printed without the lines saying which
pipeline produced the entries under it.

        TRANSLATION_REPAIR_REQUIRED_COMMIT   commit an eligible pipeline must contain
        TRANSLATION_REPAIR_POOL_ALL=yes      opt into a deliberately mixed pool

Wired into the four readers that produce numbers:
`score-probe`,
`attribution-read`,
`draw-sample`,
`damage-sample`.
Deliberately NOT wired into
`corpus-pass`,
whose directory reads answer "which entries already settled" and
"how many exist now";
filtering those would make a pass re-run settled work.

Verified on the live directory,
both directions.
Unfiltered,
`score-probe`
refuses.
Requiring `fc7912929` it runs over 1 of 22 and names the other 21.

Two failure kinds are handled OPPOSITELY,
which an existing test forced and was
right to:

-   MALFORMED,
    would not parse:
    kept in the pool.
    `attribution-read` guarantees
    one corrupt file costs its own row and not the run,
    because a pass killed at
    its hard cap leaves truncated artifacts.
    Filtering it here would take the
    file from the reader whose job is to report it.
-   UNTAGGED,
    parsed but no commit:
    excluded and named.
    A real artifact of
    unknown generation is exactly what must not be pooled.

Neither throws.
The first version threw on both,
which would have let one
truncated file destroy every report over the directory.

### The current-generation pool is 1

`MTF_0615` settled 21:12 and is the first entry carrying both 08-13 behaviour
fixes.
Against `fc7912929` the eligible pool is 1 of 22.
`#60`,
`#66` and `#68`
need many more before any rate over them means anything;
the filter now makes
that visible instead of letting 22 read as the denominator.
