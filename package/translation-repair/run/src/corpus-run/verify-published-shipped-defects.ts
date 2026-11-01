import {
  composeOverTiling,
  finalNaturalnessDefects,
  namedSlices,
  type PageGrammarReading,
  type PublishDefect,
  readPageGrammar,
  type WouldShipSource,
} from '@monochromatic-dev/translation-repair-page-check/ts';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { errorClassOf, } from '@monochromatic-dev/translation-repair-markup/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Verify published shipped defects
// THE DEFECTS A PAGE SHIPS WITH THAT ITS AGREEMENT WITH THE ARTIFACT CANNOT SHOW (B1231). The artifact records no publish
// defect, so `verify-published` re-derives the three the owner's rule ships a page with that the agreement check misses:
// a page the strict grammar refuses (`page-unparseable`, which agrees with its artifact), the archive kept where the
// grammar refused a wording (`page-grammar-archive-kept`, which the report printed as MISSING slices with no reason) and
// a body without naturalness approval (`final-naturalness`, which the report did not show at all). PRINTS THE ENTRY, THE
// DEFECT NAME AND SLICE INDICES, never the grammar's refusal, which names a place in corpus text.

/**
 Logger root for the report's own lines.
 */
const root = contextRoot({ tag: 'translation-repair', },);

/**
 Lines the report prints for the defects a page ships with.

 @param entryId - person entry the page belongs to

 @param naturalness - the defects the artifact's naturalness review makes, none when every slice has approval

 @param pageGrammar - what the grammar says of the page on disk

 @param composedGrammar - what the grammar says of the page the artifact's wordings compose, absent where it was not asked

 @param missing - slices the page does not carry, which the agreement check names

 @returns One line per defect; none for a page that parses, whose composition parses and whose body has approval

 @example
 ```ts
 const lines = shippedDefectLinesOf({ entryId, naturalness: [], pageGrammar, composedGrammar, missing: [], },);
 ```
 */
export function shippedDefectLinesOf(
  {
    entryId,
    naturalness,
    pageGrammar,
    composedGrammar,
    missing,
  }: {
    readonly entryId: string;
    readonly naturalness: readonly PublishDefect[];
    readonly pageGrammar: PageGrammarReading;
    readonly composedGrammar?: PageGrammarReading;
    readonly missing: readonly number[];
  },
): readonly string[] {
  return [
    ...naturalness.map(function naturalnessLine({ message, },): string {
      return `${entryId}: final-naturalness, ${message}`;
    },),
    ...((pageGrammar.kind === 'refused')
      ? [
        `${entryId}: page-unparseable, the strict grammar refuses the page on disk; the publisher ships such a page `
          + 'with this defect, and the site build will not take it',
      ]
      : []),
    ...(((pageGrammar.kind === 'parses') && (composedGrammar?.kind === 'refused')
      && (missing.length > 0))
      ? [
        `${entryId}: page-grammar-archive-kept, the strict grammar refuses the artifact's wordings composed as one page, `
          + 'so the publisher keeps the archive at the slices that break it; the slices named MISSING, '
          + `${namedSlices({ indices: missing, },)}, are among them`,
      ]
      : []),
  ];
}

/**
 The defects the naturalness review of an artifact makes, none for a review the guard cannot read.

 @param artifact - settled artifact

 @returns The defects, none where the review has approval or cannot be read

 @example
 ```ts
 const naturalness = naturalnessDefectsOf({ artifact, },);
 ```
 */
function naturalnessDefectsOf({ artifact, }: { readonly artifact: WouldShipSource; },): readonly PublishDefect[] {
  /**
   The deciding stages the artifact records.
   */
  const {
    consolidation,
    laneSelection,
  } = artifact;
  // AN ARTIFACT THAT RECORDS NEITHER STAGE AS RUN (one written before generation six, or whose contest or consolidation
  // never ran) has no review to weigh, and the guard would call it a fault of the code.
  if ((laneSelection.kind !== 'contested') || (consolidation.kind !== 'settled'))
    return [];
  try {
    return finalNaturalnessDefects({
      artifact: {
        consolidation,
        laneSelection,
      },
    },);
  }
  catch (error) {
    // A fault of the artifact, not a finding of the page: the log names its class, and the report names no defect.
    tagged({
      tag: naturalnessDefectsOf.name,
      l: root,
    },)
      .debug(`the naturalness review could not be read from the artifact (class=${errorClassOf({ error, },)})`,);
    return [];
  }
}

/**
 Lines the report prints for the defects a stored page ships with, read from the artifact and the page on disk.

 @param entryId - person entry the page belongs to

 @param artifact - settled artifact the page is judged against

 @param pageText - page as it sits on disk

 @param missing - slices the page does not carry, which the agreement check names

 @returns One line per defect

 @example
 ```ts
 const lines = shippedDefectLinesFor({ entryId, artifact, pageText, missing, },);
 ```
 */
export function shippedDefectLinesFor(
  {
    entryId,
    artifact,
    pageText,
    missing,
  }: {
    readonly entryId: string;
    readonly artifact: WouldShipSource;
    readonly pageText: string;
    readonly missing: readonly number[];
  },
): readonly string[] {
  /**
   What the grammar says of the page on disk.
   */
  const pageGrammar = readPageGrammar({ pageText, },);
  /**
   What the grammar says of the page the artifact's wordings compose, asked only where it can explain a missing slice.
   */
  const composedGrammar = ((pageGrammar.kind === 'parses') && (missing.length > 0))
    ? readPageGrammar({
      pageText: composeOverTiling({ artifact, },)
        .pageText,
    },)
    : undefined;
  /**
   The defects the naturalness review makes. A review the guard cannot read, an artifact that records its consolidation as
   never run or one that predates the review, raises something but its refusal, which is a fault of the artifact and
   no defect of the page; the report names no defect for it, which is a limit.
   */
  const naturalness = naturalnessDefectsOf({ artifact, },);
  return shippedDefectLinesOf({
    entryId,
    naturalness,
    pageGrammar,
    ...((composedGrammar === undefined) ? {} : { composedGrammar, }),
    missing,
  },);
}

//endregion Verify published shipped defects
