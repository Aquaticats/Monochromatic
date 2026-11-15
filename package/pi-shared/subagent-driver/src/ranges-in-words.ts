/**
 What a review's ranges are, in words a reader who does not know git's three dots understands, and as the list a review copies into
 its own file.

 A brief that names `main...fix/a` alone leaves a reviewer to read the change as the last commit, because the three dots are a
 form only someone who already knows git reads: a review of that day read one commit of a twelve-file range and said so. So every
 range is written here as the set of commits it holds, and the brief and the review's own statement both read the same words.

 @module
 */

import { codeSpanOf, } from './markdown-code-spans.ts';

/**
 How many characters the three dots of a three-dot range are.
 */
const THREE_DOTS_LENGTH = '...'.length;

/**
 How many characters the two dots of a two-dot range are.
 */
const TWO_DOTS_LENGTH = '..'.length;

/**
 Says what one range holds, in words that name both the range and its left side.

 A three-dot range `A...B` is everything `B` holds that `A` does not, which is the merge base of the two to `B`; a two-dot range
 `A..B` is the difference between the two commits themselves; anything else is taken as `git diff` takes it.

 @param range - the range, as `git diff` takes one

 @returns the range as a code span, followed by what it holds

 @example
 ```ts
 const words = rangeInWords({ range: 'main...fix/a', },);
 ```
 */
export function rangeInWords({ range, }: { readonly range: string },): string {
  /**
   Where the three dots are, or minus one when the range has none.
   */
  const threeDots = range.indexOf('...',);
  if (threeDots > 0) {
    /**
     The range's left side, which the reviewed commit is read against.
     */
    const from = range.slice(
      0,
      threeDots,
    );
    if (range.length > (threeDots + THREE_DOTS_LENGTH)) {
      return `${codeSpanOf({ text: range, },)}, which is everything the commit you start on holds that `
        + `${codeSpanOf({ text: from, },)} does not`;
    }
  }
  /**
   Where the two dots are, or minus one when the range has none.
   */
  const twoDots = range.indexOf('..',);
  if (twoDots > 0) {
    /**
     The range's left side.
     */
    const from = range.slice(
      0,
      twoDots,
    );
    /**
     The range's right side.
     */
    const to = range.slice(twoDots + TWO_DOTS_LENGTH,);
    if (to !== '') {
      return `${codeSpanOf({ text: range, },)}, the difference between ${codeSpanOf({ text: from, },)} `
        + `and ${codeSpanOf({ text: to, },)}`;
    }
  }
  return `${codeSpanOf({ text: range, },)}, as \`git diff\` takes it`;
}

/**
 Joins what several ranges hold into the phrase a brief reads after "The change is".

 @param ranges - what changed, each as `git diff` takes one

 @returns the ranges, each in words, and nothing but the word for a change that names no range

 @example
 ```ts
 const words = rangesInWords({ ranges: ['main...fix/a',], },);
 ```
 */
export function rangesInWords({ ranges, }: { readonly ranges: readonly string[] },): string {
  if (ranges.length === 0) {
    return 'no range';
  }
  /**
   Each range in its own words.
   */
  const words = ranges.map(function inWords(range,) {
    return rangeInWords({ range, },);
  },);
  /**
   Every word but the last, as one list.
   */
  const head = words.slice(
    0,
    -1,
  )
    .join(', ',);
  /**
   The words as one list, with the last joined by "and".
   */
  const list = (words.length === 1)
    ? words.join('',)
    : `${head}, and ${words.at(-1,) ?? ''}`;
  return `${(words.length === 1) ? 'the range' : 'the ranges'} ${list}`;
}

/**
 Writes the ranges as the list a review copies into its own file: each as a code span, separated by commas.

 @param ranges - what changed, each as `git diff` takes one

 @returns the ranges, ready to follow a label

 @example
 ```ts
 const line = rangesInALine({ ranges: ['main...fix/a', 'main...fix/b',], },);
 ```
 */
export function rangesInALine({ ranges, }: { readonly ranges: readonly string[] },): string {
  return ranges.map(function asACodeSpan(range,) {
    return codeSpanOf({ text: range, },);
  },)
    .join(', ',);
}
