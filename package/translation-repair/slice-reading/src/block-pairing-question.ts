import { blockPairingQuestionKey, } from './block-pairing-question-key.ts';
import type { ChunkPair, } from './chunk-document.ts';
import {
  definitionIndexes,
  type FreeOrderBlocks,
  type NumberedBlock,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Current parent question identity
// A cold section and a resumed one must number and key exactly the same blocks.

/**
 The pairing question for one aligned section, with the key its agreed answer is cached under.
 A cache hit reuses the relations a past round agreed on; it is not a fresh vote.

 @example
 ```ts
 const question = blockPairingQuestion({ pair, modelIds, });
 ```
 */
export type BlockPairingQuestion = {
  /**
   Source indexes local to this complete parent.
   */
  readonly sourceBlocks: readonly NumberedBlock[];
  /**
   Target indexes under the same local convention.
   */
  readonly targetBlocks: readonly NumberedBlock[];
  /**
   Definition indexes retaining the existing ordinary-order exemption.
   */
  readonly freeOrder: FreeOrderBlocks;
  /**
   Versioned pairing-cache key, naming the question rather than vouching for its answer.
   */
  readonly key: string;
};

/**
 Constructs the question `prepareBlockPairing` asks the roster and looks the cache up by.
 It buys no calls and does not alter singleton or empty-side dispatch.

 @param pair - complete current parent whose parsed nodes define local indexes

 @param pictureContext - transcripts the sheet is shown, part of the key when non-empty

 @param modelIds - roster that answers the question, part of the key (ledger X13)

 @returns Numbered text, definition exemptions and existing cache identity

 @example
 ```ts
 const { sourceBlocks, targetBlocks, freeOrder, key, } = blockPairingQuestion({ pair, modelIds, });
 ```
 */
export function blockPairingQuestion(
  {
    pair,
    pictureContext,
    modelIds,
  }: {
    readonly pair: ChunkPair;
    readonly pictureContext?: string;
    readonly modelIds: readonly RosterModelId[];
  },
): BlockPairingQuestion {
  /**
   Original blocks under the production question's local numbering.
   */
  const sourceBlocks = pair.source
    .nodes
    .map(function sourceBlock(
      node,
      index,
    ): NumberedBlock {
      return {
        index,
        text: node.text,
      };
    },);
  /**
   Incumbent blocks under the same numbering rule.
   */
  const targetBlocks = pair.target
    .nodes
    .map(function targetBlock(
      node,
      index,
    ): NumberedBlock {
      return {
        index,
        text: node.text,
      };
    },);
  /**
   Key naming this question, one per question (ledger X15).
   */
  const key = blockPairingQuestionKey({
    sourceBlocks,
    targetBlocks,
    ...((pictureContext === undefined) ? {} : { pictureContext, }),
    modelIds,
  });
  return {
    sourceBlocks,
    targetBlocks,
    freeOrder: {
      source: definitionIndexes({ nodes: pair.source
        .nodes, },),
      target: definitionIndexes({ nodes: pair.target
        .nodes, },),
    },
    key,
  };
}

//endregion Current parent question identity
