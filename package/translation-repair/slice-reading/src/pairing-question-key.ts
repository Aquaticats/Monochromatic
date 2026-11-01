import { createHash, } from 'node:crypto';

import { PAIRING_CACHE_VERSION, } from './pairing-cache-version.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Pairing question key
// ONE ENCODER FOR BOTH PAIRING KEYS (ledger X15, 2026-09-28). Each key once
// joined its texts with NUL and marked the side boundary with one more NUL, so
// an empty text beside the boundary, or a NUL inside a text, moved a text
// across it unseen, and UTF-8 folded every lone surrogate into U+FFFD. The
// material is now one JSON value of fixed shape: JSON escapes every quote,
// backslash, control character and lone surrogate, so two different questions
// can never serialize alike, and the question kind keeps a section key and a
// block key apart even over the same texts.

/**
 Which pairing round a key names: the whole-document section round or one
 aligned section's block round.

 @example
 ```ts
 const question: PairingQuestion = 'block';
 ```
 */
export type PairingQuestion = 'section' | 'block';

/**
 Computes the versioned key a pairing round's answer is cached under, one key
 per question.

 @param question - round the key names, so a section key and a block key
 never meet

 @param sourceTexts - original texts in document order

 @param targetTexts - translation texts in document order

 @param pictureContext - transcripts the sheet was shown, empty when it was
 shown none (class thirty-four)

 @param modelIds - roster that answers, so a pairing one bench settled is
 never resumed for another (ledger X13)

 @returns Lowercase hex SHA-256 of the question's material

 @example
 ```ts
 const key = pairingQuestionKey({ question: 'block', sourceTexts: ['猫'], targetTexts: ['Cat'], pictureContext: '', modelIds, },);
 ```
 */
export function pairingQuestionKey(
  {
    question,
    sourceTexts,
    targetTexts,
    pictureContext,
    modelIds,
  }: {
    readonly question: PairingQuestion;
    readonly sourceTexts: readonly string[];
    readonly targetTexts: readonly string[];
    readonly pictureContext: string;
    readonly modelIds: readonly RosterModelId[];
  },
): string {
  return createHash('sha256',)
    .update(
    JSON.stringify({
      version: PAIRING_CACHE_VERSION,
      question,
      source: sourceTexts,
      target: targetTexts,
      pictures: pictureContext,
      roster: modelIds,
    },),
    'utf8',
    )
    .digest('hex',);
}

//endregion Pairing question key
