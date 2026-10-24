import type { ArtifactComparisonRow, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import type {
  PageGrammarReading,
  WouldShipSource,
} from '../../dist/final/node/index.mjs';

//region Source shipped fixtures
// AN ENTRY WHOSE ARCHIVE STOPS BEFORE THE ORIGINAL DOES: the slices the archive
// covers are content spans it renders, and the slices past its end are anchors
// nobody filled, as the corpus pass XingZ6020 left slices 88 to 117 of a page
// whose archive ended at the ninth section.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Built as a literal rather than through the
// builder, because what is under test is the reading of an artifact and a
// fixture assembled by the code being read could hide a field it reads wrongly.

/**
 One slice of a source-shipped fixture: the archive wording it covers (empty
 for an anchor) and the original text it was cut from.

 @example
 ```ts
 const row: SourceShippedRow = { incumbentText: '', sourceText: '## 简介\n', };
 ```
 */
export type SourceShippedRow = {
  /**
   Archive wording the slice covers, empty at an anchor.
   */
  readonly incumbentText: string;

  /**
   Original text of the slice.
   */
  readonly sourceText: string;
};

/**
 Comparison row of one slice.

 @param row - slice

 @param sliceIndex - its index

 @returns Row the artifact records for it

 @example
 ```ts
 const comparison = comparisonRowOf({ row, sliceIndex: 0, },);
 ```
 */
function comparisonRowOf(
  {
    row,
    sliceIndex,
  }: {
    readonly row: SourceShippedRow;
    readonly sliceIndex: number;
  },
): ArtifactComparisonRow {
  if (row.incumbentText === '')
    return {
      sliceIndex,
      incumbentKind: 'absent',
      incumbentText: '',
      repairText: '',
      translateText: '',
      laneRelation: 'gap-remains',
      repairOutcome: { kind: 'unfilled', },
      translateOutcome: { kind: 'unfilled', },
      decisionComparison: {
        kind: 'comparable',
        verdict: 'same',
      },
      repairDelivery: { kind: 'gap-remains', },
      translateDelivery: { kind: 'gap-remains', },
    };
  return {
    sliceIndex,
    incumbentKind: 'present',
    incumbentText: row.incumbentText,
    repairText: row.incumbentText,
    translateText: row.incumbentText,
    laneRelation: 'archive-stands',
    repairOutcome: {
      kind: 'decided',
      acceptedText: row.incumbentText,
    },
    translateOutcome: {
      kind: 'decided',
      acceptedText: row.incumbentText,
    },
    decisionComparison: {
      kind: 'comparable',
      verdict: 'same',
    },
    repairDelivery: { kind: 'incumbent-retained', },
    translateDelivery: { kind: 'incumbent-retained', },
  };
}

/**
 Builds the artifact of an entry whose rows are given, in slice order.

 EVERY ROW IS A SLICE BOTH LANES LEFT AS THE ARCHIVE HAD IT, an anchor being one
 they left empty, so the only deciders that speak are the lanes' agreement and
 nobody backs a wording at an anchor.

 @param rows - slices, anchors among them

 @returns Artifact the publisher and the page checks read

 @example
 ```ts
 const artifact = sourceShippedArtifact({ rows: [{ incumbentText: 'A cat.\\n', sourceText: '猫\\n', },], },);
 ```
 */
export function sourceShippedArtifact(
  { rows, }: { readonly rows: readonly SourceShippedRow[]; },
): WouldShipSource {
  return {
    comparison: rows.map(function toComparison(
      row,
      sliceIndex,
    ) {
      return comparisonRowOf({
        row,
        sliceIndex,
      },);
    },),
    consolidation: { kind: 'not-run', },
    laneSelection: {
      kind: 'contested',
      slices: [],
    },
    lanes: {
      translate: {
        delivery: rows.map(function toDelivery(
          { sourceText, },
          sliceIndex,
        ) {
          return {
            sliceIndex,
            sourceText,
          };
        },),
      },
    },
  };
}

/**
 Builds the slices of the archive the rows cover, in order, an anchor at the
 offset where the archive stands when it is reached.

 @param rows - slices, anchors among them

 @returns Pairs covering the archive end to end, whose source side carries each row's original

 @example
 ```ts
 const slices = sourceShippedSlices({ rows, },);
 ```
 */
export function sourceShippedSlices(
  { rows, }: { readonly rows: readonly SourceShippedRow[]; },
): readonly ChunkPair[] {
  /**
   Offset in the archive the next slice starts at, advanced by every content span.
   */
  const cursor = { at: 0, };
  return rows.map(function toPair(
    {
      incumbentText,
      sourceText,
    },
    sliceIndex,
  ): ChunkPair {
    /**
     Where this slice's span starts.
     */
    const startOffset = cursor.at;
    cursor.at += incumbentText.length;
    return {
      source: {
        kind: 'content',
        sliceIndex,
        nodes: [],
        startOffset: 0,
        endOffset: 0,
        text: sourceText,
      },
      target: (incumbentText === '')
        ? makeInsertionChunk({
          sliceIndex,
          offset: startOffset,
        },)
        : {
          kind: 'content',
          sliceIndex,
          nodes: [],
          startOffset,
          endOffset: cursor.at,
          text: incumbentText,
        },
    };
  },);
}

/**
 Whole archive the rows cover, which is their archive wordings end to end.

 @param rows - slices, anchors among them

 @returns Archive English

 @example
 ```ts
 const archive = sourceShippedArchive({ rows, },);
 ```
 */
export function sourceShippedArchive(
  { rows, }: { readonly rows: readonly SourceShippedRow[]; },
): string {
  return rows
    .map(function archiveOf({ incumbentText, },): string {
      return incumbentText;
    },)
    .join('',);
}

/**
 A grammar that refuses a text, whether a page or a fragment of its own, that holds the word NOPE.

 STANDS FOR A RUN THAT NO WRITING MAKES PARSE, which the real grammar has for nesting past a bound.

 @param pageText - page or fragment

 @returns Refusal for a text holding the word, a parse otherwise

 @example
 ```ts
 const reading = refusingNope({ pageText: 'NOPE', },);
 ```
 */
export function refusingNope({ pageText, }: { readonly pageText: string; },): PageGrammarReading {
  return pageText.includes('NOPE',)
    ? {
      kind: 'refused',
      refusal: 'at 1:1 (test/nope)',
    }
    : { kind: 'parses', };
}

//endregion Source shipped fixtures
