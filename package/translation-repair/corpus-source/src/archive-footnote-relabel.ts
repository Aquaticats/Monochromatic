import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { DefinitionLabelPair, } from '@monochromatic-dev/translation-repair-stage/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  activeFootnoteMarkers,
  isInsertionChunk,
  sliceFootnoteLabels,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type FootnoteRewriteError,
  normalizeFootnoteIdentifier,
  relabelsFootnote,
} from '@monochromatic-dev/translation-repair-document-structure/ts';


//region Archive footnote relabel
// THE ARCHIVE'S FOOTNOTE LABELS FOLLOW THE ORIGINAL'S, decided after the
// yuki418330012 page of 2026-09-08 shipped its two markers pointing at each
// other's notes. The original writes 洲洲[^2] and 真理[^1]; the archive had
// renumbered them by first appearance, Zhouzhou[^1] and Zhenli[^2], with its
// definitions numbered to match. Every lane judged the body against the
// original, so the body followed the original's labels, while the archive's
// two definitions were left standing as they were (the roster paired neither,
// and the original's definitions, rendered as an insertion, were withdrawn as
// duplicates). The page carried `Zhenli[^1]` above `[^1]: Yuki's substitute
// parent`. THE NINETEENTH CLASS: a footnote is a relation between slices,
// and two slices that each followed a different numbering agree with
// themselves and not with each other.
//
// THE FIX IS UPSTREAM OF EVERY LANE. Once the archive's labels are rewritten
// to the original's, label equality IS the correspondence: a definition
// pairs with the definition of the same label, the body's markers agree with
// the incumbent's, and a definition that renders the wrong note is a fidelity
// defect the lanes see and repair like any other.
//
// READ OFF THE PAIRED SLICES, positionally. Within one slice the original and
// the archive carry the same passage, so the k-th distinct marker on one side
// is the k-th on the other. A slice whose two sides carry different counts of
// distinct markers says nothing about the labels (the archive dropped or added
// a note there, which the lanes see as the fidelity defect it is; hakureico's
// archive carries no [^2] at all) and is left out of the reading, named. Two
// slices that disagree about one label leave the archive as it is with a
// warning: a wrong relabel would be the defect this exists to stop.

/**
 One label the archive carries and the original's label for the same note.

 @example
 ```ts
 const relabel: FootnoteRelabel = { from: '1', to: '2', };
 ```
 */
export type FootnoteRelabel = {
  /**
   Label as the archive spells it.
   */
  readonly from: string;

  /**
   Label the original gives the same note.
   */
  readonly to: string;
};

/**
 What the paired slices say about the archive's labels.

 @example
 ```ts
 const reading: FootnoteRelabelReading = { kind: 'unchanged', correspondences: [], skipped: [], };
 ```
 */
export type FootnoteRelabelReading = {
  /**
   The archive's labels differ from the original's and every one maps.
   */
  readonly kind: 'relabel';

  /**
   Complete supplied relations, including identities that establish original-label coverage.
   */
  readonly correspondences: readonly FootnoteRelabel[];

  /**
   Labels to rewrite, only the ones that change.
   */
  readonly map: readonly FootnoteRelabel[];

  /**
   Slices left out of the reading, each with why.
   */
  readonly skipped: readonly string[];
} | {
  /**
   The labels already agree, or no slice carries a marker on both sides.
   */
  readonly kind: 'unchanged';

  /**
   Positive relations, distinct from the absence of any correspondence evidence.
   */
  readonly correspondences: readonly FootnoteRelabel[];

  /**
   Slices left out of the reading, each with why.
   */
  readonly skipped: readonly string[];
} | {
  /**
   The slices disagree, so the archive stands as it is.
   */
  readonly kind: 'ambiguous';

  /**
   Which slice, and what each side carried.
   */
  readonly detail: string;
};

/**
 One claim that an archive label is an original label, and where it was
 read.
 */
export type LabelCorrespondence = {
  /**
   Label as the archive spells it.
   */
  readonly from: string;

  /**
   Label the original gives the same note.
   */
  readonly to: string;

  /**
   Where the claim was read, for the detail: a slice or a definition pair,
   named so a later claim contradicting this one names it too.
   */
  readonly where: string;
};

/**
 Folds correspondences into one map, refusing as ambiguous the first that
 contradicts an earlier one on either side.

 THE DETAIL NAMES THE EARLIER CLAIM AS IT WAS MADE (ledger T8, eighteenth
 batch): its place, which side the label it names stands on, and that label
 as the document spells it. It once ended "where an earlier slice mapped
 [^X]": X was an original label when one archive label met two, an archive
 label when two met one, and in either case the parser's case-folded key
 rather than the document's spelling, and "slice" was the later claim's kind
 of place, so a slice contradicting a definition pair blamed a slice.

 @param correspondences - claims in reading order

 @param skipped - places that said nothing, carried into the reading

 @returns Reading

 @example
 ```ts
 mapLabels({ correspondences: [ { from: '1', to: '2', where: 'slice 3', }, ], skipped: [], },);
 ```
 */
export function mapLabels(
  {
    correspondences,
    skipped,
  }: {
    readonly correspondences: readonly LabelCorrespondence[];
    readonly skipped: readonly string[];
  },
): FootnoteRelabelReading {
  /**
   The claim that first mapped each archive label, keyed by its
   parser-equivalent form.
   */
  const forward = new Map<string, LabelCorrespondence>();

  /**
   The claim that first mapped onto each original label, keyed the same way,
   so two archive labels cannot claim one.
   */
  const backward = new Map<string, LabelCorrespondence>();
  /**
   First supplied spelling of each distinct logical relation, including identities.
   */
  const distinct: FootnoteRelabel[] = [];
  for (const claim of correspondences) {
    /**
     Parser-equivalent archive identifier.
     */
    const from = normalizeFootnoteIdentifier({ identifier: claim.from, },);
    /**
     Parser-equivalent original identifier.
     */
    const to = normalizeFootnoteIdentifier({ identifier: claim.to, },);
    /**
     The earlier claim on this archive label, when one was made.
     */
    const forwardSeen = forward.get(from,);
    /**
     The earlier claim onto this original label, when one was made.
     */
    const backwardSeen = backward.get(to,);
    /**
     The claim as the detail opens with it.
     */
    const contradicting = `${claim.where} maps archive [^${claim.from}] to original [^${claim.to}]`;
    if ((forwardSeen !== undefined)
      && (normalizeFootnoteIdentifier({ identifier: forwardSeen.to, },) !== to))
      return {
        kind: 'ambiguous',
        detail: `${contradicting}, where ${forwardSeen.where} mapped that archive label to original [^${
          forwardSeen.to
        }]`,
      };
    if ((backwardSeen !== undefined)
      && (normalizeFootnoteIdentifier({ identifier: backwardSeen.from, },) !== from))
      return {
        kind: 'ambiguous',
        detail: `${contradicting}, where ${backwardSeen.where} mapped archive [^${
          backwardSeen.from
        }] to that original label`,
      };
    // A REPEAT of a relation already read adds nothing: its first claim stays
    // the one a later contradiction names.
    if (forwardSeen !== undefined)
      continue;
    distinct.push({
      from: claim.from,
      to: claim.to,
    },);
    forward.set(
      from,
      claim,
    );
    backward.set(
      to,
      claim,
    );
  }
  /**
   Only changes of logical identity need rewriting; positive identity evidence remains separate.
   */
  const map = distinct.filter(function changes(relation,): boolean {
    return relabelsFootnote({ relation, },);
  },);
  if (map.length === 0)
    return {
      kind: 'unchanged',
      correspondences: distinct,
      skipped,
    };
  return {
    kind: 'relabel',
    map,
    correspondences: distinct,
    skipped,
  };
}

/**
 Reads, off the paired slices, how the archive's labels map to the
 original's.

 @param slices - preparation's slices, both sides' canonical texts included

 @param sourceText - complete original backing the prepared ranges

 @param targetText - complete archive backing the prepared ranges

 @returns Map, that nothing changes, or why the archive must stand

 @throws {@link FootnoteRewriteError} when document syntax or slice scope cannot establish current evidence

 @example
 ```ts
 const reading = footnoteRelabelOf(prepared);
 ```
 */
export function footnoteRelabelOf(
  {
    slices,
    sourceText,
    targetText,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly targetText: string;
  },
): FootnoteRelabelReading {
  /**
   Complete-document source syntax, never an unmatched prepared container half.
   */
  const sourceReferences = activeFootnoteMarkers({ text: sourceText, },)
    .filter(function reference(marker,): boolean {
      return marker.kind === 'reference';
    },);
  /**
   Archive syntax uses the same actual document boundary.
   */
  const targetReferences = activeFootnoteMarkers({ text: targetText, },)
    .filter(function reference(marker,): boolean {
      return marker.kind === 'reference';
    },);
  /**
   Correspondences the slices give, positionally.
   */
  const correspondences: LabelCorrespondence[] = [];

  /**
   Slices that said nothing about the labels, each with why.
   */
  const skipped: string[] = [];
  for (const slice of slices) {
    /**
     Current original references projected into an exact prepared range.
     */
    const original = sliceFootnoteLabels({
      chunk: slice.source,
      documentText: sourceText,
      markers: sourceReferences,
    },);
    /**
     Current archive references, with stale or truncated ranges refused before counting.
     */
    const archive = sliceFootnoteLabels({
      chunk: slice.target,
      documentText: targetText,
      markers: targetReferences,
    },);
    if (isInsertionChunk(slice.target,))
      continue;
    /**
     Which slice, for the detail.
     */
    const sliceIndex = String(
      slice.target
        .sliceIndex,
    );
    if ((original.length === 0) && (archive.length === 0))
      continue;
    if (original.length !== archive.length) {
      skipped.push(
        `slice ${sliceIndex} references ${String(original.length,)} distinct ${
          wordForCount({
            count: original.length,
            one: 'note',
            many: 'notes',
          },)
        } in the original and ${
          String(archive.length,)
        } in the archive`,
      );
      continue;
    }
    for (const [at, from,] of archive.entries()) {
      // The two lists have one length, checked just before, so the original
      // carries a label at every position the archive does.
      correspondences.push({
        from,
        to: nonNullishOrThrow(original[at],),
        where: `slice ${sliceIndex}`,
      },);
    }
  }
  return mapLabels({
    correspondences,
    skipped,
  },);
}

/**
 Reads the map off the definitions the roster paired by content, the exact
 evidence: the archive's definition of a note carries the archive's label
 for it, and the original's carries the original's.

 @param pairs - definition pairs, by label

 @returns Map, that nothing changes, or why the archive must stand

 @example
 ```ts
 footnoteRelabelOfDefinitions({ pairs: [ { sourceLabel: '2', targetLabel: '1', }, { sourceLabel: '1', targetLabel: '2', }, ], },);
 // Positive relations remain in `correspondences`, including identities omitted from `map`.
 ```
 */
export function footnoteRelabelOfDefinitions(
  { pairs, }: { readonly pairs: readonly DefinitionLabelPair[]; },
): FootnoteRelabelReading {
  return mapLabels({
    correspondences: pairs.map(function toCorrespondence(
      pair,
      at,
    ): LabelCorrespondence {
      return {
        from: pair.targetLabel,
        to: pair.sourceLabel,
        where: `definition pair ${String(at,)}`,
      };
    },),
    skipped: [],
  },);
}

//endregion Archive footnote relabel
