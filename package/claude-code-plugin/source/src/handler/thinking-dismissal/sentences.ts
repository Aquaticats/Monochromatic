/**
 Sentence splitting and vocabulary matching over prose.

 Every helper makes one left-to-right pass over its input with O(1) stack,
 which is what unbounded thinking text needs.

 @module
 */

import {
  SENTENCE_END,
  WHITESPACE,
} from './patterns.ts';

/**
 Whether the character at one offset is whitespace.

 False when the offset sits past the end of the text,
 so callers never read outside the string.

 @param text - Text to inspect

 @param offset - Character offset to test

 @returns True when a whitespace character sits at the offset

 @example
 ```ts
 whitespaceAt({ text: 'a b', offset: 1 }); // => true
 ```
 */
function whitespaceAt(
  {
    text,
    offset,
  }: {
    readonly text: string;
    readonly offset: number;
  },
): boolean {
  /**
   True when the offset sits past the end of the text.
   */
  const past = offset >= text.length;
  if (past) return false;
  return WHITESPACE.has(text.charAt(offset,),);
}

/**
 Splits prose into sentences in one left-to-right pass.

 A sentence ends at sentence-ending punctuation followed by whitespace,
 or at the end of the text.
 Character iteration keeps the split O(n) with O(1) stack
 over unbounded thinking text,
 which a regex split cannot show as clearly.

 @param prose - Text to split

 @returns Sentences in order, without their trailing separators

 @example
 ```ts
 sentencesOf('One. Two.'); // => ['One.', 'Two.']
 ```
 */
function sentencesOf(prose: string,): readonly string[] {
  /**
   Sentences collected so far.
   */
  const sentences: string[] = [];
  /**
   Start offset of the sentence being accumulated.
   */
  let start = 0;
  for (let index = 0; index < prose.length; index += 1) {
    if (!SENTENCE_END.has(prose.charAt(index,))) continue;
    /**
     Offset just past this punctuation character.
     */
    const after = index + 1;
    /**
     True when the punctuation closes the text.
     */
    const atEnd = after >= prose.length;
    /**
     True when whitespace follows the punctuation.
     */
    const atWhitespace = whitespaceAt({
      text: prose,
      offset: after,
    },);
    /**
     True when the punctuation ends the sentence.
     */
    const closed = atEnd || atWhitespace;
    if (!closed) continue;
    /**
     Sentence just closed, punctuation included.
     */
    const sentence = prose.slice(
      start,
      after,
    );
    sentences.push(sentence,);
    /**
     Offset of the next sentence's first character,
     after the separator whitespace.
     */
    let next = after;
    while (whitespaceAt({
      text: prose,
      offset: next,
    },)) {
      next += 1;
    }
    index = next - 1;
    start = next;
  }
  /**
   True when text remains after the last sentence-ending punctuation.
   */
  const trailing = start < prose.length;
  if (trailing) {
    /**
     Closing sentence without trailing punctuation.
     */
    const sentence = prose.slice(start,);
    sentences.push(sentence,);
  }
  return sentences;
}

/**
 Whether a sentence carries any term of one vocabulary.

 @param sentence - Lowercased sentence under test

 @param vocabulary - Lowercase terms to look for

 @returns True when any term occurs in the sentence

 @example
 ```ts
 hasTerm({ sentence: 'the lines are short', vocabulary: ['short'] }); // => true
 ```
 */
function hasTerm(
  {
    sentence,
    vocabulary,
  }: {
    readonly sentence: string;
    readonly vocabulary: readonly string[];
  },
): boolean {
  for (const term of vocabulary) {
    if (sentence.includes(term)) return true;
  }
  return false;
}

export {
  hasTerm,
  sentencesOf,
  whitespaceAt,
};
