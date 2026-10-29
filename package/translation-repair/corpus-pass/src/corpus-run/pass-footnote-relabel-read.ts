import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { RelabelClosure, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  closeFootnoteRelabel,
  documentLabels,
} from '@monochromatic-dev/translation-repair-archive/ts';
import {
  footnoteRelabelOf,
  footnoteRelabelOfDefinitions,
  type FootnoteRelabelReading,
  widenFootnoteRelabel,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { DefinitionLabelPair, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Pass footnote relabel reading
// How the pass reads the label map and closes it over the archive's labels:
// off the definitions the roster paired by content where it paired any, off
// the paired slices where it paired none, and off both together where the
// definitions alone do not close (class ninety-five). Split out of
// `pass-footnote-relabel.ts` so that file keeps to its line budget.

/**
 Evidence the definitions the roster paired give.
 */
const DEFINITIONS_BASIS = 'the definitions the roster paired';

/**
 Evidence the paired slices give.
 */
const SLICES_BASIS = 'the paired slices';

/**
 Evidence both give together.
 */
const BOTH_BASIS = 'the definitions the roster paired and the paired slices';

/**
 A reading with evidence in it: a map, or the labels already agreeing.
 */
type EvidentReading = Exclude<FootnoteRelabelReading, { readonly kind: 'ambiguous'; }>;

/**
 What the pass read: the evidence disagreeing with itself, or a reading
 with its closure over the archive's labels.

 @example
 ```ts
 const read = readClosedRelabel({ entryId, definitionPairs, slices, sourceText, archiveText, l, },);
 if (read.kind === 'read') apply(read.closure);
 ```
 */
export type ClosedRelabelReading = {
  /**
   The evidence disagrees with itself, so the archive stands.
   */
  readonly kind: 'ambiguous';

  /**
   Which evidence, for the log and the findings.
   */
  readonly basis: string;

  /**
   Which claims disagreed.
   */
  readonly detail: string;
} | {
  /**
   The evidence agrees with itself.
   */
  readonly kind: 'read';

  /**
   What the evidence says about the labels.
   */
  readonly reading: EvidentReading;

  /**
   Which evidence, for the log and the findings.
   */
  readonly basis: string;

  /**
   The map closed over the archive's labels, or why it stays open.
   */
  readonly closure: RelabelClosure;
};

/**
 Closes a reading over both documents' labels, or names the disagreement
 where the reading is ambiguous.

 @param reading - what the evidence says

 @param basis - which evidence

 @param sourceText - original page

 @param archiveText - archive text

 @returns Reading with its closure, or the disagreement

 @example
 ```ts
 const read = closeReading({ reading, basis, sourceText, archiveText, },);
 ```
 */
function closeReading(
  {
    reading,
    basis,
    sourceText,
    archiveText,
  }: {
    readonly reading: FootnoteRelabelReading;
    readonly basis: string;
    readonly sourceText: string;
    readonly archiveText: string;
  },
): ClosedRelabelReading {
  if (reading.kind === 'ambiguous')
    return {
      kind: 'ambiguous',
      basis,
      detail: reading.detail,
    };
  /**
   The map closed over the archive's labels, completed by elimination where
   that is forced; an empty map where the labels already agree.
   */
  const closure = closeFootnoteRelabel({
    map: reading.correspondences,
    archiveLabels: documentLabels({ text: archiveText, },),
    originalLabels: documentLabels({ text: sourceText, },),
  },);
  return {
    kind: 'read',
    reading,
    basis,
    closure,
  };
}

/**
 Reads the label map off the definitions the roster paired, off the paired
 slices where it paired none, and off both together where the definitions
 alone do not close, closing each reading over the archive's labels.

 @param entryId - entry being prepared, for the log

 @param definitionPairs - definitions the roster paired, by label

 @param slices - first preparation's slices

 @param sourceText - complete original backing the slices

 @param archiveText - complete archive backing the slices

 @param l - entry logger

 @returns Reading that stands with its closure, or the disagreement

 @example
 ```ts
 const read = readClosedRelabel({ entryId, definitionPairs, slices, sourceText, archiveText, l, },);
 ```
 */
export function readClosedRelabel(
  {
    entryId,
    definitionPairs,
    slices,
    sourceText,
    archiveText,
    l,
  }: {
    readonly entryId: string;
    readonly definitionPairs: readonly DefinitionLabelPair[];
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly archiveText: string;
    readonly l: Logger;
  },
): ClosedRelabelReading {
  if (definitionPairs.length === 0)
    return closeReading({
      reading: footnoteRelabelOf({
        slices,
        sourceText,
        targetText: archiveText,
      },),
      basis: SLICES_BASIS,
      sourceText,
      archiveText,
    },);
  /**
   What the definitions say, the exact evidence, with its closure.
   */
  const ofDefinitions = closeReading({
    reading: footnoteRelabelOfDefinitions({ pairs: definitionPairs, },),
    basis: DEFINITIONS_BASIS,
    sourceText,
    archiveText,
  },);
  if (ofDefinitions.kind === 'ambiguous')
    return ofDefinitions;
  /**
   How the definitions' map closed.
   */
  const { closure, } = ofDefinitions;
  if (closure.kind === 'closed')
    return ofDefinitions;
  // CLASS NINETY-FIVE: the definitions alone leave the map open, so the
  // paired slices are read beside them before the archive is left standing.
  l.info(
    `FOOTNOTES entry=${entryId} the map read off ${DEFINITIONS_BASIS} does not close (${closure.detail}); reading ${SLICES_BASIS} beside them`,
  );
  return closeReading({
    reading: widenFootnoteRelabel({
      definitions: ofDefinitions.reading,
      slices: footnoteRelabelOf({
        slices,
        sourceText,
        targetText: archiveText,
      },),
    },),
    basis: BOTH_BASIS,
    sourceText,
    archiveText,
  },);
}

//endregion Pass footnote relabel reading
