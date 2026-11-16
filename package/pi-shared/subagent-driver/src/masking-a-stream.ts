/**
 One matcher for every stream the driver writes: it masks what it Is sure of and holds back what could still become a match.

 A stream is cut where the next read begins, so a value can be split across two writes. Masking each write on its own would let the
 two halves through, which together are the whole value. This holds back the longest suffix that is still the beginning of an armed
 form, and refuses to write anything else it cannot decide: what it holds is bounded by the longest form, so the run cannot wait on
 it for long.

 @module
 */

import type { TextMasking, } from './secret-value-masking.ts';

//region Types

/**
 What one stream's writes go through.
 */
export type StreamMatcher = {
  /**
   The text to write for what arrived, which is nothing while what arrived is still the beginning of a possible match.

   @param chunk - what arrived
   */
  readonly push: (chunk: string) => string;

  /**
   The text to write for what was held back, once nothing more will arrive.
   */
  readonly flush: () => string;

  /**
   How many characters are held back right now, which is the bound on what this matcher keeps.
   */
  readonly heldLength: () => number;
};

//endregion Types

//region The matcher

/**
 Starts one stream's matcher.

 @param mask - the masking the stream is written through

 @returns the matcher, which holds back what could still become a match

 @example
 ```ts
 const matcher = createStreamMatcher({ mask, },);
 const first = matcher.push('half a val',);
 ```
 */
export function createStreamMatcher({ mask, }: { readonly mask: TextMasking },): StreamMatcher {
  /**
   The text held back, which is the beginning of a form this matcher has not decided about yet.
   */
  const held: { text: string } = { text: '', };

  /**
   Masks one piece of text, saying so when the masking refuses it rather than writing the text it refused.

   @param text - the text to write

   @returns what to write
   */
  function maskedOrRefused({ text, }: { readonly text: string },): string {
    try {
      return mask.maskText(text,);
    }
    catch (error) {
      if (!Error.isError(error,)) {
        throw error;
      }
      return '[the rest of this stream was withheld: its masking failed]\n';
    }
  }

  return {
    push: function push(chunk,) {
      held.text += chunk;
      /**
       How many of its last characters could still become a match, which must not be written yet.
       */
      const keep = mask.heldBackSuffixLength(held.text,);
      /**
       What may be written now.
       */
      const writable = (keep === 0) ? held.text : held.text
        .slice(
        0,
        held.text
          .length
          - keep,
        );
      held.text = (keep === 0) ? '' : held.text
        .slice(-keep,);
      return (writable === '') ? '' : maskedOrRefused({ text: writable, },);
    },
    flush: function flush() {
      /**
       What was held back when nothing more will arrive.
       */
      const rest = held.text;
      held.text = '';
      return (rest === '') ? '' : maskedOrRefused({ text: rest, },);
    },
    heldLength: function heldLength() {
      return held.text
        .length;
    },
  };
}

//endregion The matcher
