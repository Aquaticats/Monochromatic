/**
 Size-dismissal detection over one thinking block.

 Detection returns hashes and indexes,
 never reasoning text.

 @module
 */

import { createHash, } from 'node:crypto';

import {
  COST_TERMS,
  DISMISSAL_TERMS,
  SIZE_DISMISSAL,
} from './patterns.ts';
import {
  hasTerm,
  sentencesOf,
} from './sentences.ts';

/**
 One detection inside one thinking block.

 Carries a hash of the block and the sentence index instead of any text,
 since reasoning text stays on this machine.
 */
type CostDismissalHit = {
  /**
   Pattern the sentence matches.
   */
  readonly pattern: typeof SIZE_DISMISSAL;

  /**
   First 16 hex characters of the SHA-256 of the whole block.
   */
  readonly blockHash: string;

  /**
   Zero-based index of the matching sentence inside the block.
   */
  readonly sentenceIndex: number;
};

/**
 Finds size-dismissal sentences in one thinking block.

 One left-to-right pass per block: sentences are walked in order,
 and each is checked against both vocabularies once.

 @param block - Thinking text of one block

 @returns Hits in sentence order, empty when the block dismisses nothing

 @example
 ```ts
 findSizeDismissal('Quadratic, but the lines are short.');
 // => [{ pattern: 'size-dismissal', blockHash: '...', sentenceIndex: 0 }]
 ```
 */
function findSizeDismissal(block: string,): readonly CostDismissalHit[] {
  /**
   SHA-256 of the block, truncated, so records can locate it without text.
   */
  const digest = createHash('sha256',)
    .update(block,)
    .digest('hex',);
  /**
   Short block identifier recorded instead of any text.
   */
  const blockHash = digest.slice(
    0,
    16,
  );
  /**
   Hits found so far.
   */
  const hits: CostDismissalHit[] = [];
  /**
   Sentences of the block, matched one at a time.
   */
  const sentences = sentencesOf(block,);
  for (const [sentenceIndex, sentence,] of sentences.entries()) {
    /**
     Lowercased sentence, matched case-insensitively.
     */
    const lowered = sentence.toLowerCase();
    /**
     True when the sentence puts algorithmic cost in scope.
     */
    const cost = hasTerm({
      sentence: lowered,
      vocabulary: COST_TERMS,
    },);
    /**
     True when the sentence waves the cost away.
     */
    const dismissal = hasTerm({
      sentence: lowered,
      vocabulary: DISMISSAL_TERMS,
    },);
    if (cost && dismissal) {
      hits.push({
        pattern: SIZE_DISMISSAL,
        blockHash,
        sentenceIndex,
      },);
    }
  }
  return hits;
}

export type { CostDismissalHit, };

export { findSizeDismissal, };
