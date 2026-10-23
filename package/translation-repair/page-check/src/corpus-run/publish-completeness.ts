import type { UnfilledSlice, } from '@monochromatic-dev/translation-repair-translate-lane/ts';

//region Publication completeness
// A corpus artifact may retain evidence about a missing passage, and a published
// memorial page never omits such a passage silently: it carries the passage's
// original text under a note, and the finding records it.
//
// THE SILENT GAP IS GONE FOR A SLICE WITH NO ARCHIVE TEXT AND NO BACKED WORDING.
// The owner, 2026-10-23, asked what a shipped page carries for such a slice,
// chose "Source text + note": the page carries the source text of the passage,
// preceded by a note saying it has not been translated yet and the original
// follows, footnote definitions included; a later run replaces it once a
// wording is backed (`source-ships.ts`). Options offered and not chosen: the
// note alone; keeping the silent gap; and unbacked wording marked as
// unreviewed, which would break the standing rule that one judge alone never
// decides. The earlier rulings stand: a run always ships, and a slice with
// archive text and no valid wording keeps the archive.
//
// HOW THE GAP CAME TO BE, kept because it is why the entry still ships. The
// page shipped without the passage since the no-loop design of 2026-09-01
// (doc/planning/translation-repair-no-loop-design.md, "Insertion placement,
// single round"): an insertion is recovered supplementary content whose
// absence is a recorded gap, not a missing required page. A refusal that
// predated that design outlived it: on 2026-09-02 it dropped XIEPT2 after 35
// minutes over one passage the judges could not back in two rounds, and the
// publish test records an earlier XIEPT2 attempt lost the same way after four
// hours forty-eight minutes. The pass has recorded the finding and published
// since; the refusal, kept for callers that might want to fail closed, had
// none, and went on 2026-09-29 with its error class (ledger B30). The gap
// itself went on 2026-10-23, after the corpus pass XingZ6020 shipped 88 of 118
// slices and left the other 30, 31.5 percent of the original and every
// footnote definition, off the page with nothing on it to say so (ledger B827).

/**
 Findings that record each unfilled source passage a page ships without.

 @param unfilled - source passages with no shipped rendering

 @returns One finding per passage, naming the slice and why it is unfilled, and what the page carries instead

 @example
 ```ts
 unfilledPageFindings({ unfilled: [{ sliceIndex: 15, reason: 'no-candidate-backed', findings: [], }], },);
 // => ['source-passage-unfilled (slice 15, no-candidate-backed): the page carries the source text of this passage under a note that it has not been translated yet']
 ```
 */
export function unfilledPageFindings(
  { unfilled, }: { readonly unfilled: readonly UnfilledSlice[]; },
): readonly string[] {
  return unfilled.map(function toFinding(passage,): string {
    return `source-passage-unfilled (slice ${String(passage.sliceIndex,)}, ${passage.reason}): `
      + 'the page carries the source text of this passage under a note that it has not been translated yet';
  },);
}

//endregion Publication completeness
