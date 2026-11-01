import type { NumberedBlock, } from '@monochromatic-dev/translation-repair-stage/ts';
import { pairingQuestionKey, } from './pairing-question-key.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Block-pairing cache key

/**
 Computes the versioned key one aligned section's block pairing is cached
 under: one key per question, since `pairingQuestionKey` serializes its
 material injectively (ledger X15).

 @param sourceBlocks - native original block texts in emitted order

 @param targetBlocks - native incumbent block texts in their separate emitted order

 @param pictureContext - transcripts the sheet was shown, empty when none were
 (class thirty-four, 2026-09-16: a pairing bought without the pictures must not
 answer the question asked with them)

 @param modelIds - roster that answers the question, so a pairing one bench
 settled is never resumed for another (ledger X13, 2026-09-28: a section a dry
 reading paired on a subset resumed on the full bench)

 @returns Lowercase hex SHA-256 cache key

 @example
 ```ts
 const key = blockPairingQuestionKey({ sourceBlocks, targetBlocks, modelIds, });
 ```
 */
export function blockPairingQuestionKey({
  sourceBlocks,
  targetBlocks,
  pictureContext = '',
  modelIds,
}: {
  readonly sourceBlocks: readonly NumberedBlock[];
  readonly targetBlocks: readonly NumberedBlock[];
  readonly pictureContext?: string;
  readonly modelIds: readonly RosterModelId[];
}): string {
  return pairingQuestionKey({
    question: 'block',
    sourceTexts: sourceBlocks.map(function sourceContent(block,): string {
      return block.text;
    },),
    targetTexts: targetBlocks.map(function targetContent(block,): string {
      return block.text;
    },),
    pictureContext,
    modelIds,
  },);
}

//endregion Block-pairing cache key
