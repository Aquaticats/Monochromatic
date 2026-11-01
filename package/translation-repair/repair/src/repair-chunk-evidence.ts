import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Chunk evidence
// SPLIT OUT OF `repair-chunk.ts` for the file-length cap when class one
// hundred nine added the checker stage's re-seat. Nothing here decides
// anything: it is the window and the parsed pair every stage of one chunk
// reads, built once so the critic, the panel and the editor see the same
// evidence.

/**
 Neighbouring passages as the stages receive them: absent where the chunk
 has no neighbour on that side.
 */
export type NeighbourWindow = {
  readonly neighbouringSourceText?: string;
  readonly neighbouringIncumbentText?: string;
};

/**
 Evidence every stage of one chunk reads.
 */
export type ChunkEvidence = {
  /**
   Neighbouring evidence, spread into every stage that has to reason about it.

   BUILT ONCE RATHER THAN PASSED THREE TIMES. The critic, the panel and the
   editor must see the SAME window or they contradict each other: a critic that
   can see next door raises a relocation claim, and a panel that cannot see it
   rejects that claim as unfounded. Three call sites spreading one value cannot
   drift the way three separate arguments can.
   */
  readonly windowFragment: NeighbourWindow;
  /**
   Parsed chunk pair claims anchor against.
   */
  readonly documents: {
    readonly source: ReturnType<typeof parseDocument>;
    readonly target: ReturnType<typeof parseDocument>;
  };
};

/**
 Builds the evidence one chunk's stages share.

 @param sourceText - original of this chunk

 @param targetText - archive English of this chunk

 @param neighbouringSourceText - original of the passages either side, absent
 at a document's edge

 @param neighbouringIncumbentText - archive English of those passages, absent
 at a document's edge

 @returns Window fragment and parsed pair

 @example
 ```ts
 const { windowFragment, documents, } = chunkEvidence({ sourceText, targetText, },);
 ```
 */
export function chunkEvidence(
  {
    sourceText,
    targetText,
    neighbouringSourceText,
    neighbouringIncumbentText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly neighbouringSourceText?: string;
    readonly neighbouringIncumbentText?: string;
  },
): ChunkEvidence {
  return {
    windowFragment: {
      ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
      ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
    },
    documents: {
      source: parseDocument({ text: sourceText, },),
      target: parseDocument({ text: targetText, },),
    },
  };
}

//endregion Chunk evidence
