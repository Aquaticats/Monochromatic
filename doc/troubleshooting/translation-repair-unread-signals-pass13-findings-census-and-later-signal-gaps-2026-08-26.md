# Unread signals: the pass13 findings census and two later signal gaps, to 2026-08-26

A census of the findings recorded in the first settled entries of pass13,
including the panel parser that degrades gracefully
and the alignment finding that contradicts its own message,
and the 2026-08-26 pair:
a coverage sentence that misread its seat,
and the link check that did not exist.

Part of [Signals the pipeline emits and nobody reads](translation-repair-unread-signals.md),
which lists every file of this record.

## Census of the findings nobody reads, pass13

Every settled artifact carries a `findings` array.
`score-probe` reads it,
but
 only to summarize the editor and refine rosters,
so every other kind recorded
 there has never been looked at.
Taken over pass13's first 13 settled entries:
 748 findings.

NONE OF THEM CONTAIN CORPUS TEXT.
Checked rather than assumed:
zero of 748
 contain any CJK character,
and the rare kinds were read with quoted spans
 redacted before anything was pasted anywhere.
That makes a findings census safe
 to share in a way the artifacts themselves are not.

The bulk are ordinary stage bookkeeping:

-   `refine-skip` 214,
    `refine-skipped` 100
-   `editor-candidates` 88,
    `editor-envelope-select` 87,
    `editor-chunk-select` 87
-   `quote-not-found` 69,
    which is the anchoring loss the quote-anchoring finding already measured
-   `refine-candidates` 28,
    `empty-quote` 24,
    `refine-selected` 13,
    `refine-declined` 13,
    `refine-recheck-passed` 8

The tail is where the unread signal lives:

-   `group-index-out-of-range` 5
-   `ambiguous-quote` 3,
    `quote-outside-blocks` 3,
    `non-translation` 2
-   `alignment` 1,
    `duplicate-verdict` 1,
    `unknown-regrade-severity` 1,
    `missing-verdict` 1

### The panel parser degrades gracefully, which is worth stating

`group-index-out-of-range`,
`duplicate-verdict`,
`missing-verdict` and
 `unknown-regrade-severity` all come from `adjudicate-wire.ts`,
and all four are
 the parser correctly refusing a malformed judge answer rather than a defect of
 its own.
The important one is `unknown-regrade-severity`
 (`adjudicate-wire.ts:279`):
it keeps the VOTE and drops only the unusable
 severity,
so a judge that mis-typed a severity still counts toward the verdict.
 A first reading of the name suggests a lost voice,
and it is not one.

At 8 occurrences in 748 findings across 13 entries,
the panel is receiving
 well-formed answers nearly always.
This is a healthy signal that had never been
 read either.

### The one alignment finding contradicts its own message

```text
alignment structure-mismatch (pair 0: section structures differ
  (source 1 chunks, target 1 chunks); aligned proportionally by character fraction)
```

The structures are declared different while both sides have exactly ONE chunk.
 That is not a contradiction in the data,
it is the mismatch test being about
 something other than counts,
and the code says exactly which:

```ts
// package/translation-repair/slice-reading/src/chunk-document.ts, the shape test as it stood;
// today it is `equalShape` and reads the same two things
const mirrored = (sourceChunks.length === targetChunks.length)
  && sourceChunks.every(function leadingKindMatches(chunk, index,) {
    return chunk.nodes[0]?.kind === targetChunks[index]?.nodes[0]?.kind;
  },);
```

With one chunk on each side,
the count test passes by construction,
so
 the shape test can be false for exactly one reason:
the leading node kinds differ.
 One document opens with a heading and the other with a paragraph.
That is the
 asymmetric preamble the aligner-scoring finding named,
confirmed from the source rather than
 inferred from a sample.

The fallback did no harm HERE,
because pairing one chunk to one chunk
 proportionally gave the same answer as pairing them directly.
But it was the
 same code path that shifted every section of `XingZ60` by two,
and it fired on
 an entry where nothing was wrong with the document at all.
A test that reports
 a mismatch on a forced pairing is reporting on the preamble,
not the structure.
 The proportional path has since been deleted (`chunk-document.ts` says so at
 its head);
an unequal shape now goes to `alignHeadingsForced`,
and the finding
 names the sections it could not pair rather than a fraction it aligned by.

## A signal that was read and said the wrong thing, and one that did not exist (2026-08-26)

THE COVERAGE SENTENCE MISREAD ITS OWN SEAT.
`WROTE NOTHING AT ALL: minimax-m3` at the end of arm A's REFINER
table told the reader to re-run a seat whose `SEAT` line,
printed by the same command,
said `asked=31 usable=31`;
arm A2 repeated it for `minimax-m3` and `deepseek-v4-flash-0731`.
The line was derived from slates,
and a
rewriter that answers every ask and leaves the paragraph as it stands reaches no slate.
The answered-but-unslated fix (landed in source,
awaiting the build) carries the refine stage's heard ids out to the coverage split,
which gains an
answered-but-unslated state;
the editor and translate seats still carry only a count out and say so.
The general shape is the one this document keeps finding:
a summary line derived from one record while the
contradicting record sits three lines below it,
and nothing compares them.

NOTHING WATCHED THE LINKS.
The naturalness lane protects link destinations as ordered atoms,
but a slice that
ships through the repair lane's archive sentence or the contest's incumbent can drop a source hyperlink with no
finding anywhere;
the output reading found a page that had.
The destinations check (landed in source as `fd4f7546f`) adds the document-level
check at publish:
`DESTINATIONS <id> source=N page=M dropped=K` on stdout beside the `TALLY` line,
the
addresses at info in the run log,
a warn line with the count.
It is a finding and never a refusal,
because the
page is what both deciders approved.
`grep '^DESTINATIONS ' <log> | grep -v 'dropped=0'` is the read.
