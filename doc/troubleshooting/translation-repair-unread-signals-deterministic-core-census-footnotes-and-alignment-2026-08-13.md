# Unread signals: census one of the deterministic core, footnotes and alignment, 2026-08-13

What the deterministic core emitted over the 92-entry corpus,
and the two defects the census found:
a footnote graph wrong on 10 of 15 findings,
and an alignment mismatch finding that was a false alarm 6 times in 7,
plus the check that alignment never drops content and the four settled repairs that shipped broken footnotes.

Part of [Signals the pipeline emits and nobody reads](translation-repair-unread-signals.md),
which lists every file of this record.

## Census one: what the deterministic core emits over the 92-entry corpus

```text
PARSE findings, both sides of 92 entries
    95  html-comment-skipped
     3  invisible-line-masked

FOOTNOTE graph findings
    15  unresolved-reference        across 2 files

ALIGNMENT findings
     7  structure-mismatch          across 7 entries
     3  sections-merged
```

## The footnote graph was wrong, and nothing was reading it to notice

10 of those 15 unresolved references were FALSE.
`shihai4h/page.en.md` carries ten references and ten definitions,
and a raw
 scan of the text finds every one of them,
yet the graph reported
 `definitions: []` and called all ten references unresolved.

Cause:
`parseDocument` built the node list from FLATTENED children and the
 footnote graph from the RAW ones.
Every definition in that file sits inside a disclosure container,
so
 `flattenContainers` promoted them for the node list while the graph,
walking
 the unflattened tree,
saw a container where a definition should be.

The same disagreement corrupted identifiers.
`buildFootnoteGraph` names blocks `block/N` by position in the list it walks,
 so every id it emitted after a container named a different block than the one
 the document exposes under that name.

Fixed by flattening once and sharing the result.
After the fix:
5 findings across 1 file,
all real.
`XingZ60/page.en.md` carries 5 references and 0 definitions while its Chinese
 side resolves 9 of 9,
which is consistent with that translation being
 incomplete.
Corpus totals are 107 references against 102 definitions,
and no graph
 identifier names a block that does not exist.

Nothing reads `footnoteGraph` outside `parse-document.ts`,
which is why a
 deterministic detector could be wrong about 10 of its 15 outputs without
 anyone noticing.

## The alignment mismatch finding is a false alarm 6 times out of 7

Seven entries emitted `structure-mismatch` and,
at the time,
took a proportional fallback.
THAT FALLBACK IS GONE:
`chunk-document.ts` now computes `equalShape` (equal chunk counts and matching
leading node kinds),
pairs by index with no finding only when the shape is equal and no roster pairing
was supplied (the equal-count blind spot the code documents),
and otherwise runs `alignHeadingsForced`,
which
emits a pairing only where the affinity grid gives a unique partner in both directions;
every unpaired
section becomes a `structure-mismatch` finding and a source-only section is placed as an insertion or
refused.
The seven-entry table in this section is the measurement as it was taken,
kept for the blast-radius argument.
Only ONE of the seven actually mispaired:

```text
  Aniloviraw    chunks  1/1   equal counts, leading kinds differ   pairs by index
  Hangmster     chunks  1/1   equal counts, leading kinds differ   pairs by index
  interrgned    chunks  5/5   equal counts, leading kinds differ   pairs by index
  noname        chunks  4/4   equal counts, leading kinds differ   pairs by index
  yingying      chunks  1/1   equal counts, leading kinds differ   pairs by index
  XIEPT2        chunks  8/9   counts differ                        pairs by index
  XingZ60       chunks 15/13  counts differ                        SLIDES
```

Five of the seven had EQUAL chunk counts and differed only in the leading node
 kind of the first chunk,
which is what the shape test (then `mirrored`,
now
 `equalShape`) also checks;
the proportional fallback then paired them by index
 anyway and no harm was done.
`XIEPT2` had unequal counts and still paired by index.
Only `XingZ60` slid,
and sliding is no longer possible:
a count mismatch goes to
 the forced aligner rather than to a proportional merge.

So the blast radius for genuine mispairing is one entry,
and this is the right
 way to have established it.
An earlier check used HEADING counts as the proxy and got the right answer for
 the wrong reason:
chunk counts are what the aligner compares,
and they differ
 from heading counts because content before the first heading forms a chunk.

Anyone gating on `structure-mismatch` would discard six good entries to catch
 one bad one.

## Alignment never drops content

Checked separately,
since a merge could in principle leave blocks in no pair:
 across all 92 entries,
every block on both sides belongs to some pair.
`XIEPT2`'s extra target chunk is merged rather than lost.

## Four settled repairs shipped broken footnotes, and the detector was right there

The point of the fixed graph is that it can now be trusted,
so it was pointed
 at the pipeline's own output:
parse each settled entry's input translation and
 its `repairedText`,
and compare the two graphs.

56 entries examined,
4 broken,
0 healed:

```text
Dethelly       refs [1]     -> [1]      defs [1]     -> [1 2]    orphan-definition 2
Futajuhuacha   refs [1 2]   -> [1 2]    defs [1 2]   -> [1 2 2]  duplicate-definition 2
Y1Ran          refs [1 2 3] -> [2 3]    defs [1 2 3] -> [1 2]    unresolved-reference 3
                                                                 orphan-definition 1
gqt            refs []      -> [1]      defs []      -> []       unresolved-reference 1
```

Four different corruptions:

-   `Dethelly` gained a definition nothing references.
-   `Futajuhuacha` had a definition duplicated.
-   `Y1Ran` lost a reference and a definition,
    from different footnotes,
    so one
    reference now points nowhere and one definition is orphaned.
-   `gqt` had a footnote reference INVENTED in a document that carried no
    footnotes at all,
    pointing at a definition that has never existed.

Three further entries changed footnote counts while staying internally
 consistent:
`Huasheng` lost a matched pair,
`XIEPT2` gained one,
`hakureico`
 gained two.
Those are not corruption on this measure,
and whether a repair
 should be inventing or removing footnotes at all is a separate question.

Every one of the four passed the integrity check and shipped,
because integrity
 is `downgradeCount`,
which counts only MDX grammar downgrades.
Breaking a
 footnote leaves the grammar perfectly valid.

This is the whole thesis in one measurement:
the pipeline computes a
 deterministic detector for exactly this damage,
on every document,
and never
 consults it.

### What was done about it

`footnoteBreakCount` now joins `downgradeCount` in the candidate integrity
 gate:
a patched chunk may carry no more footnote findings than the chunk it
 replaced.
Comparison rather than an absolute count,
so a chunk holding a dangling
 reference the translation arrived with is still repairable.

Two limits,
both deliberate and neither hidden:

-   The gate is CHUNK-scoped,
    like every other measurement beside it,
    so it
    sees damage a patch does within one chunk.
    A definition deleted in one
    chunk whose reference lives in another passes,
    because neither chunk's own
    count rises.
    `Y1Ran` may be exactly that shape.
    Catching it needs a
    document-scoped check,
    which is not built.
-   The measurement that found this compared WHOLE documents,
    so it does not
    prove each of the four would have been refused by a chunk-scoped gate.
    It
    proves the damage is detectable by a detector already running.

Both rest on the graph being right,
which until 2026-08-13 it was not:
it would
 have reported ten false breaks on `shihai4h` and could not see a definition
 inside a container at all.
