import { longestRunOf, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Prompt fence
// Choosing a fence no enclosed text can reproduce.
//
// Every stage that interpolates corpus prose or model output into a prompt is
// writing into a destination grammar, not onto a display surface. The enclosed
// text is arbitrary: a setext heading underline is an ordinary row of equals
// signs, and a translation can legitimately contain one. Fenced with a fixed
// delimiter, that text closes its own block and everything after it reads as
// instructions to the model.
//
// Choosing the fence against the content removes the possibility rather than
// making it unlikely. `candidate-select-wire.ts` did this first and privately;
// it lives here now because the introduced-defect probe needs the same
// guarantee, and two copies of a security boundary is one copy too many.

/**
 Shortest fence used when nothing enclosed competes with it.
 */
export const PROMPT_FENCE_MIN = 5;

/**
 Fence character.
 */
export const FENCE_CHARACTER = '=';

/**
 Longest unbroken run of the fence character anywhere in one text.

 Single linear pass, because the input is unbounded corpus prose.

 @param text - content that will be fenced

 @returns Longest run length, zero when the character never appears

 @example
 ```ts
 const longest = longestFenceRun('a ==== b',);
 ```
 */
export function longestFenceRun(text: string,): number {
  return longestRunOf({
    text,
    character: FENCE_CHARACTER,
  },);
}

/**
 Chooses a fence no enclosed text can reproduce.

 @param texts - every text this prompt will fence

 @returns Fence strictly longer than any run inside them

 @example
 ```ts
 const fence = selectFence({ texts: [sourceText, ...rendered,], },);
 ```
 */
export function selectFence({ texts, }: { readonly texts: readonly string[]; },): string {
  /**
   Longest fence-character run anywhere in the enclosed content.
   */
  const longest = texts.reduce(
    function longerRun(
      best: number,
      text,
    ): number {
      return Math.max(
        best,
        longestFenceRun(text,),
      );
    },
    0,
  );
  return FENCE_CHARACTER.repeat(Math.max(
    PROMPT_FENCE_MIN,
    longest + 1,
  ),);
}

//endregion Prompt fence
