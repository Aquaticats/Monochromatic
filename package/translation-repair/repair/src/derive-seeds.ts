import type { SeededErrorSpec, } from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Seed derivation
// Derives omission seeds from the text itself at runtime, so benchmarks over the
// UNLICENSED corpus never require quoting its content inside this repository.
// Deleting whole sentences models the load-bearing-omission class observed in the
// real error bank.

/**
 Characters that terminate a sentence in either language of the corpus.
 */
export const SENTENCE_TERMINATORS = '。．.!?！？';

/**
 Minimum sentence length worth deleting;
 shorter sentences are weak omission tests and often ambiguous needles.
 */
const MIN_SENTENCE_LENGTH = 40;

/**
 Characters opening or closing MDX expressions and JSX tags.
 A sentence carrying one may hold half of a paired construct;
 deleting it would leave the seeded document unparseable as MDX,
 so such sentences never become needles.
 */
const MDX_DELIMITERS = [
  '{',
  '}',
  '<',
  '>',
] as const;

/**
 Splits text into sentences by terminator scan.
 Single linear pass; a sentence is the slice from the previous terminator
 (exclusive) through its own terminator.

 @param text - body text to segment

 @returns Trimmed sentences in source order

 @example
 ```ts
 const sentences = splitSentences({ text: body, },);
 ```
 */
export function splitSentences({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Collected sentences in source order.
   */
  const sentences: string[] = [];

  /**
   Start of the sentence currently being scanned.
   */
  let sentenceStart = 0;

  for (
    let cursor = 0;
    cursor < text.length;
    cursor += 1
  ) {
    if (SENTENCE_TERMINATORS.includes(text.charAt(cursor,),)) {
      /**
       Candidate sentence including its terminator, trimmed.
       */
      const sentence = text
        .slice(
        sentenceStart,
        cursor + 1,
        )
        .trim();
      if (sentence !== '')
        sentences.push(sentence,);
      sentenceStart = cursor + 1;
    }
  }

  return sentences;
}

/**
 Derives deletion seeds from the longest sentences of one target body.
 Sentences that occur more than once are skipped (ambiguous needles),
 as are sentences carrying MDX expression or JSX delimiters
 (deleting half of a paired construct breaks the seeded parse);
 results are deterministic for a given text.

 @param text - target body text (front matter excluded by the caller)

 @param maxSeeds - ceiling on derived seeds

 @returns Omission seeds in descending sentence length order

 @example
 ```ts
 const seeds = deriveOmissionSeeds({ text: body, maxSeeds: 3, },);
 ```
 */
export function deriveOmissionSeeds(
  {
    text,
    maxSeeds,
  }: {
    readonly text: string;
    readonly maxSeeds: number;
  },
): readonly SeededErrorSpec[] {
  /**
   Long-enough sentences, longest first;
   ties keep source order because sort is stable.
   */
  const candidates = splitSentences({ text, },)
    .filter(function longEnough(sentence,) {
      return sentence.length >= MIN_SENTENCE_LENGTH;
    },)
    .filter(function freeOfMdxDelimiters(sentence,) {
      return MDX_DELIMITERS.every(function absent(delimiter,) {
        return !sentence.includes(delimiter,);
      },);
    },)
    .toSorted(function byLengthDescending(
      a,
      b,
    ) {
      return b.length - a.length;
    },);

  /**
   Unambiguous candidates capped at the ceiling.
   */
  const chosen = candidates
    .filter(function occursOnce(sentence,) {
      /**
       First occurrence, guaranteed present.
       */
      const first = text.indexOf(sentence,);
      return !text.includes(
        sentence,
        first + 1,
      );
    },)
    .slice(
    0,
    maxSeeds,
    );

  return chosen.map(function toSpec(
    sentence,
    index,
  ) {
    return {
      id: `seed/omission-${String(index,)}`,
      category: 'accuracy/omission',
      kind: 'deletion',
      needle: sentence,
      replacement: '',
    };
  },);
}

//endregion Seed derivation
