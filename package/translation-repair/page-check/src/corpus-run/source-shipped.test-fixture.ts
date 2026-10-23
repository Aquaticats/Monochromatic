import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import type { WouldShipSource, } from '../../dist/final/node/index.mjs';

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
 Builds the artifact of an entry whose rows are given, in slice order.

 EVERY ROW IS A SLICE BOTH LANES LEFT AS THE ARCHIVE HAD IT, an anchor being one
 they left empty, so the only deciders that speak are the lanes' agreement and
 nobody backs a wording at an anchor.

 @param rows - the slices, anchors among them

 @returns Artifact the publisher and the page checks read

 @example
 ```ts
 const artifact = sourceShippedArtifact({ rows: [{ incumbentText: 'A cat.\n', sourceText: '猫\n', },], },);
 ```
 */
export function sourceShippedArtifact(
  { rows, }: { readonly rows: readonly SourceShippedRow[]; },
): WouldShipSource {
  return {
    comparison: rows.map(function toComparison({ incumbentText, }, sliceIndex,) {
      return {
        sliceIndex,
        incumbentKind: (incumbentText === '') ? 'absent' : 'present',
        incumbentText,
        repairText: incumbentText,
        translateText: incumbentText,
        laneRelation: (incumbentText === '') ? 'gap-remains' : 'archive-stands',
        repairOutcome: (incumbentText === '') ? { kind: 'unfilled', } : { kind: 'decided', acceptedText: incumbentText, },
        translateOutcome: (incumbentText === '') ? { kind: 'unfilled', } : { kind: 'decided', acceptedText: incumbentText, },
        decisionComparison: { kind: 'not-comparable', undecidedLanes: ['repair', 'translate',], },
        repairDelivery: (incumbentText === '') ? { kind: 'gap-remains', } : { kind: 'incumbent-retained', },
        translateDelivery: (incumbentText === '') ? { kind: 'gap-remains', } : { kind: 'incumbent-retained', },
      };
    },),
    consolidation: { kind: 'not-run', },
    laneSelection: {
      kind: 'contested',
      slices: [],
    },
    lanes: {
      translate: {
        delivery: rows.map(function toDelivery({ incumbentText, sourceText, }, sliceIndex,) {
          return {
            sliceIndex,
            sourceText,
            incumbentKind: (incumbentText === '') ? 'absent' : 'present',
            incumbentText,
          };
        },),
      },
    },
  } as unknown as WouldShipSource;
}

/**
 Builds the slices of the archive the rows cover, in order, an anchor at the
 offset where the archive stands when it is reached.

 @param rows - the slices, anchors among them

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
  return rows.map(function toPair({ incumbentText, sourceText, }, sliceIndex,): ChunkPair {
    /**
     Where this slice's span starts.
     */
    const startOffset = cursor.at;
    cursor.at += incumbentText.length;
    return {
      source: {
        sliceIndex,
        nodes: [],
        startOffset: 0,
        endOffset: 0,
        text: sourceText,
      },
      target: {
        ...((incumbentText === '') ? { kind: 'insertion', } : {}),
        sliceIndex,
        nodes: [],
        startOffset,
        endOffset: cursor.at,
        text: incumbentText,
      },
    } as unknown as ChunkPair;
  },);
}

/**
 Whole archive the rows cover, which is their archive wordings end to end.

 @param rows - the slices, anchors among them

 @returns The archive English

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

//endregion Source shipped fixtures
