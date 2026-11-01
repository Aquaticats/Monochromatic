import { QUOTED_WORDS_REMINDER, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { DECLARED_NAME_REFERENCE_EXEMPTION, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { PAGE_APPARATUS_IS_KEPT, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Editor selection sheet
// Task and criteria the two editor selections put to the judges, apart from
// `editor-ensemble.ts` at the line cap on 2026-09-09 when the community
// glossary reached the select sheet.

/**
 Faithfulness as both editor selections test it, with the exemptions the
 translate slate reads (ledger S14): a declared name referring to a person is
 no addition, and the page's own apparatus is kept.
 */
const EDITOR_FAITHFULNESS = 'Faithfulness to the ORIGINAL: no content added, dropped, or altered in meaning. '
  // Ledger L14(b): the chunk judges selected a candidate repeating a sentence
  // 3 times over every artifact, and Carena0442 slice 14 shipped two.
  + 'Saying something twice is adding content unless the ORIGINAL says it again at that place: a candidate that '
  + 'repeats a sentence the passage already carries elsewhere, or carries what a neighbouring passage says, adds '
  + `content. ${DECLARED_NAME_REFERENCE_EXEMPTION} ${PAGE_APPARATUS_IS_KEPT}`;

/**
 What the per-envelope selection asks.
 */
export const ENVELOPE_SELECTION_TASK =
  'Each candidate replaces the SAME passage of an English translation of the Chinese ORIGINAL below.';

/**
 How the per-envelope selection decides, earlier criteria outranking later.
 */
export const ENVELOPE_SELECTION_CRITERIA: readonly string[] = [
  EDITOR_FAITHFULNESS,
  'Natural, idiomatic English that carries the ORIGINAL\'s feeling.',
  // Ledger S14: "Fits the surrounding text in register and tense" made the
  // surrounding English the tense authority, against the house tense rule.
  'Fits the surrounding text in register. Its tense follows the house tense rule (the past for the life of a person '
    + `who has died), not surrounding English that tells the life in the present. ${QUOTED_WORDS_REMINDER}`,
];

/**
 What the whole-chunk selection asks.
 */
export const CHUNK_SELECTION_TASK =
  'Each candidate is a full English translation of the Chinese ORIGINAL below, after repairs were applied.';

/**
 How the whole-chunk selection decides, earlier criteria outranking later.
 */
export const CHUNK_SELECTION_CRITERIA: readonly string[] = [
  EDITOR_FAITHFULNESS,
  'Natural, idiomatic English reading as one coherent passage, not as stitched fragments.',
  'Consistent voice, tense, and terminology across the whole passage.',
];

//endregion Editor selection sheet
