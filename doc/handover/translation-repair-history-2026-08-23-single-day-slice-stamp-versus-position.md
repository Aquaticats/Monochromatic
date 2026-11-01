# Translation repair history 2026-08-23: the slice stamp against position claim

The stale claim that nothing asserts slice stamping,
the one probe path that really did stamp wrongly and its fix,
and a process note on a file overwritten before it was read.

Part of [Translation repair history: 2026-08-23, the work recorded for that day alone](translation-repair-history-2026-08-23-single-day.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## `#94`: its stamp-versus-position claim was stale, and the one real instance is fixed

`#94`'s 2026-08-23 addition says NOTHING ASSERTS THE INVARIANT ANYWHERE and that it holds
by the coincidence that every production caller passes a running counter.
That is wrong.
`slice-indexing.ts` already exists,
exports `assertSliceIndexing` and
`reindexSlicePair`,
and is called at `document-preparation.ts:373` and `splice-slices.ts:358`.
It has its own unit test file,
and `slice-pictures.ts` documents relying on it in three places.
So the invariant is enforced at preparation,
which is exactly where the task proposes putting it.

A consequence worth stating:
a corpus census of stamp against position over all 92 pairs reads
zero mismatches,
but that null is GUARANTEED by the assertion rather than independent of it,
since `prepareDocumentPair` would have thrown.
It is not extra evidence.

### The one path that really did stamp wrongly

`corpus-run/probe-relabel-case.ts` carved slices with `baseIndex: index`,
where `index` is the PAIR index rather than a running count of slices already carved.
Every section therefore restamped from its own number:
pair 1's first slice claimed 1,
which pair 0's second slice already held.

Measured over the first twelve corpus entries,
before and after:

-   BEFORE:
    4 of 12 entries carry duplicate stamps and a stamp that is not its position.
    `Chinatsu_Suzuki` is the worst,
    18 slices with 6 stamps duplicated and 16 off position.
-   AFTER:
    0 duplicate,
    0 non-positional.
-   Slice TEXT is byte-identical between the two on every entry,
    which is what `baseIndex` touching only `chunkIndex` predicts.

It was inert rather than wrong:
this probe finds its slice by text (`.includes(before)`)
and returns only `holder.source.text` and `holder.target.text`,
so no stamp was ever read.
That answers the task's open question about whether it reports against shifted slices.
It does not.

FIXED by stamping from the finished order through `reindexSlicePair`,
which exists for precisely this and whose own TSDoc says the preparation
must never trust the arithmetic it handed out,
and by calling `assertSliceIndexing` on the result.
This probe deliberately bypasses `prepareDocumentPair` in order to re-carve exactly what the run
carved,
so the bypass was also skipping the invariant every other slicing is held to.

GFP:
`assertSliceIndexing` was run against the pre-fix stamping of `Chinatsu_Suzuki`
and refuses it with `SliceIndexingError`,
so the guard catches the defect it now prevents.

A first attempt used a `reduce` with a spread accumulator and was rejected by
`oxc(no-accumulating-spread)`.
The rule was right:
carve,
then stamp by final position,
is both linear and the shape the codebase already had a function for.

### Process note: a file was overwritten before it was read

While building what turned out to be a duplicate of `slice-indexing.ts`,
that existing file was overwritten with `cat >` without being read first.
Recovered whole with `git checkout --` because it was committed and the worktree was clean.
The rule this breaks is to look at the target before overwriting;
the reason it cost nothing is that the tree was clean,
which is not a defence,
only luck.
Check `git status` and read the path before writing to a name that might exist.
