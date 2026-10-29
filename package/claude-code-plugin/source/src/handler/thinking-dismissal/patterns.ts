/**
 Vocabularies and sentinels for thinking-dismissal detection.

 Kept apart from the matcher so the phrase lists can grow
 without touching scanning or recording code.

 @module
 */

//region Vocabularies

/**
 Cost vocabulary: words that put algorithmic cost in scope.

 Calibrated against the two known dismissal blocks of session `73f0639e`
 and an 11-transcript negative sample,
 see `doc/handover/superlinear-cost-dismissal-gate.md`,
 heading `Local calibration, 2026-10-29`.
 Stored lowercase; matching lowercases the sentence first.
 */
const COST_TERMS: readonly string[] = [
  'o(n',
  'o(1',
  'quadratic',
  'superlinear',
  'exponential',
  'complexity',
  'nested loop',
  'nested scan',
  'per entry',
  'per row',
  'per item',
  'per call',
  'n^2',
  'n²',
  'linear pass',
  'one pass',
  'accumulator',
  'expensive',
  'costly',
  'slowdown',
  'blow up',
  'blows up',
];

/**
 Dismissal vocabulary: words that wave cost away through an input-size claim
 or through a verdict of acceptability.
 */
const DISMISSAL_TERMS: readonly string[] = [
  'short',
  'small',
  'tiny',
  'few',
  'little',
  'negligible',
  'acceptable',
  'fine',
  'okay',
  'realistic',
  'in practice',
  'typical',
  'bounded',
  'limited',
  'unlikely',
  'won\'t matter',
  'does not matter',
  'doesn\'t matter',
  'not a problem',
  'no problem',
  'good enough',
  'harmless',
];

//endregion

//region Sentence boundaries

/**
 Sentence-ending punctuation that closes a sentence for matching.
 */
const SENTENCE_END: ReadonlySet<string> = new Set([
  '.',
  '!',
  '?',
],);

/**
 Whitespace characters that separate sentences after the ending punctuation.
 */
const WHITESPACE: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
  '\f',
  '\v',
],);

//endregion

//region Pattern ids

/**
 Pattern id recorded for a sentence that carries cost and dismissal together.
 */
const SIZE_DISMISSAL = 'size-dismissal' as const;

//endregion

export {
  COST_TERMS,
  DISMISSAL_TERMS,
  SENTENCE_END,
  SIZE_DISMISSAL,
  WHITESPACE,
};
