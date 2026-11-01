/**
 The quotes a scripted judge gives for the cat page of
 `consistency-page.test-fixture.ts`: the repeated sentence's two places and the
 refrain's four. Cat-themed invention; no corpus content appears here.

 @module
 */

/**
 Quotes of the repeated sentence's two places that stand once in the archive's
 English: the first with its closing full stop, the second without one.
 */
export const SENTENCE_QUOTES: readonly Record<string, string>[] = [
  {
    label: 'OCCURRENCE 1',
    quote: 'The sunny window put the kitten to sleep.',
  },
  {
    label: 'OCCURRENCE 2',
    quote: 'the sunlight on the sill lulled the kitten to sleep',
  },
];

/**
 Quotes of the refrain's four places that stand once in the archive's English,
 each with its slot.
 */
export const REFRAIN_QUOTES: readonly Record<string, string>[] = [
  {
    label: 'OCCURRENCE 1',
    quote: 'So this cat must be a chubby little treasure.',
    slot: 'chubby',
  },
  {
    label: 'OCCURRENCE 2',
    quote: 'So this cat is surely a lazy little treasure.',
    slot: 'lazy',
  },
  {
    label: 'OCCURRENCE 3',
    quote: 'So this cat has to be a sweet little darling.',
    slot: 'sweet',
  },
  {
    label: 'OCCURRENCE 4',
    quote: 'So this cat is certainly a warm little dumpling.',
    slot: 'warm',
  },
];
