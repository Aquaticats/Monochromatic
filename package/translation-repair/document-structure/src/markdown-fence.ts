import { longestRunOf, } from './character-run.ts';

//region Markdown fence
// Choosing a code fence no enclosed text can close.
//
// Grading sheets interpolate corpus prose and model output into Markdown, which
// is a destination grammar, not a display surface. A replacement is arbitrary
// text: it can contain backticks, a line starting with `###`, or a literal
// `- repair grade: [ ]`. Interpolated raw, the first breaks the block, the
// second invents a heading, and the third puts a grade box on the sheet that
// nobody wrote. Curly quotation marks around it change nothing, because quotes
// are not Markdown syntax.
//
// The fence is chosen against the content rather than fixed, for the same
// reason `candidate-select-wire.ts` chooses its prompt fence against candidate
// text: a fixed fence lets the content close its own block and have the rest
// read as sheet.

/**
 Shortest fence used when nothing enclosed competes with it, which is also
 Markdown's own minimum for a fenced block.
 */
const MARKDOWN_FENCE_MIN = 3;

/**
 Fence character; backticks are what Markdown fences with.
 */
const FENCE_CHARACTER = '`';

/**
 Longest unbroken run of the fence character anywhere in one text.

 Single linear pass, because the input is unbounded corpus prose.

 @param text - content that will be fenced

 @returns Longest run length, zero when the character never appears

 @example
 ```ts
 const longest = longestBacktickRun('a ``` b',);
 ```
 */
export function longestBacktickRun(text: string,): number {
  return longestRunOf({
    text,
    character: FENCE_CHARACTER,
  },);
}

/**
 Wraps text in a fenced block no enclosed content can close.

 @param text - content to enclose

 @returns Fenced block, opening and closing fences on their own lines

 @example
 ```ts
 const block = fenceForMarkdown({ text: '### not a heading here', },);
 ```
 */
export function fenceForMarkdown({ text, }: { readonly text: string; },): string {
  /**
   Fence strictly longer than any run inside the content.
   */
  const fence = FENCE_CHARACTER.repeat(Math.max(
    MARKDOWN_FENCE_MIN,
    longestBacktickRun(text,) + 1,
  ),);
  return `${fence}text\n${text}\n${fence}`;
}

//endregion Markdown fence
