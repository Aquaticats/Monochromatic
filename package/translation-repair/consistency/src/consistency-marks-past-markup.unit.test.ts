/**
 Tests for naming the apostrophe-like marks of an English that stand past markup: the ones that end a run of closing
 delimiters of emphasis (by the parse) or a link's closing `](…)` after a Latin letter or a digit, on the trailing side alone,
 read once for a whole text whatever the length of a run or of a link destination.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { marksPastMarkupOf, } from '../dist/final/node/index.mjs';

/**
 An English with each mark `marksPastMarkupOf` names shown between white square brackets, so a case reads as the text it is
 about.

 @param english - English

 @returns English with the named marks bracketed; the English itself when none is named

 @example
 ```ts
 const shown = markedPastMarkup({ english: 'The *kittens*’ bowl', },); // 'The *kittens*⟦’⟧ bowl'
 ```
 */
function markedPastMarkup({ english, }: { readonly english: string; },): string {
  const { trailing, } = marksPastMarkupOf({ english, },);
  return Array.from(
    { length: english.length, },
    function unitShown(_, at,): string {
      return trailing.has(at,) ? `⟦${english.charAt(at,)}⟧` : english.charAt(at,);
    },
  )
    .join('',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: marksPastMarkupOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE APOSTROPHE-LIKE MARK THAT ENDS A RUN OF CLOSING DELIMITERS OF EMPHASIS OR A LINK\'S CLOSING `](…)` AFTER A '
            + 'LATIN LETTER OR DIGIT, over emphasis, strong emphasis, strikethrough and a link with its destination, and names one '
            + 'a letter follows as well',
          fn: async () => {
            expect([
              markedPastMarkup({ english: 'The *kittens*’ bowl', },),
              markedPastMarkup({ english: 'The **kittens**\' bowl', },),
              markedPastMarkup({ english: 'The [kittens](https://cats.example/k/)’ bowl', },),
              markedPastMarkup({ english: 'The ~~kittens~~\u{02BC} bowl', },),
              markedPastMarkup({ english: 'The 5 *cats 5*’', },),
              markedPastMarkup({ english: 'The *kittens*’s bowl', },),
              markedPastMarkup({ english: 'The **[kittens](x)**’ bowl', },),
            ],).toEqual([
              'The *kittens*⟦’⟧ bowl',
              'The **kittens**⟦\'⟧ bowl',
              'The [kittens](https://cats.example/k/)⟦’⟧ bowl',
              'The ~~kittens~~⟦\u{02BC}⟧ bowl',
              'The 5 *cats 5*⟦’⟧',
              'The *kittens*⟦’⟧s bowl',
              'The **[kittens](x)**⟦’⟧ bowl',
            ],);
          },
        },),
        it({
          name: 'READS NO CLOSING BRACKET, DOUBLE QUOTATION MARK, CORNER BRACKET OR INLINE CODE AS PART OF THE RUN, since after them '
            + 'a single mark is most often a closing quotation mark and is read where it stands',
          fn: async () => {
            expect([
              markedPastMarkup({ english: 'The "kittens"’ bowl', },),
              markedPastMarkup({ english: 'The “kittens”’ bowl', },),
              markedPastMarkup({ english: 'The (kittens)’ bowl', },),
              markedPastMarkup({ english: 'The `kittens`’ bowl', },),
              markedPastMarkup({ english: 'The 「kittens」’ bowl', },),
              markedPastMarkup({ english: 'He said cats are "ok"\' now.', },),
              markedPastMarkup({ english: 'Come (here)’ now', },),
            ],).toEqual([
              'The "kittens"’ bowl',
              'The “kittens”’ bowl',
              'The (kittens)’ bowl',
              'The `kittens`’ bowl',
              'The 「kittens」’ bowl',
              'He said cats are "ok"\' now.',
              'Come (here)’ now',
            ],);
          },
        },),
        it({
          name: 'NAMES NO MARK WHEN NO LATIN LETTER OR DIGIT STANDS BEFORE THE RUN, when the run is empty or broken by a space, '
            + 'when a link\'s destination holds a space or is never closed, or when the mark is the opening curly single mark',
          fn: async () => {
            expect([
              markedPastMarkup({ english: 'The kittens’ bowl', },),
              markedPastMarkup({ english: 'The * ’ bowl', },),
              markedPastMarkup({ english: 'The *kittens* ’ bowl', },),
              markedPastMarkup({ english: 'The [kittens](a )’ bowl', },),
              markedPastMarkup({ english: 'The [kittens](a’ bowl', },),
              markedPastMarkup({ english: 'The 猫*’ bowl', },),
              markedPastMarkup({ english: 'The *kittens*‘ bowl', },),
              markedPastMarkup({ english: '*’ bowl', },),
              markedPastMarkup({ english: '', },),
            ],).toEqual([
              'The kittens’ bowl',
              'The * ’ bowl',
              'The *kittens* ’ bowl',
              'The [kittens](a )’ bowl',
              'The [kittens](a’ bowl',
              'The 猫*’ bowl',
              'The *kittens*‘ bowl',
              '*’ bowl',
              '',
            ],);
          },
        },),
        it({
          name: 'READS NO MARK BEFORE OPENING MARKUP AS PAST MARKUP, since the reading run has no leading mirror: a mark before '
            + 'emphasis, a bracket, a quotation mark or inline code is read where it stands, and the result names no leading marks',
          fn: async () => {
            expect([
              markedPastMarkup({ english: 'He said ’*tis* well', },),
              markedPastMarkup({ english: 'He said \'(tis) well', },),
              markedPastMarkup({ english: 'He said ’"tis" well', },),
              markedPastMarkup({ english: 'He said ’“tis” well', },),
              markedPastMarkup({ english: 'He said ’`tis` well', },),
              markedPastMarkup({ english: 'He said ’「tis」 well', },),
              markedPastMarkup({ english: 'He said ’**cats** now.', },),
              markedPastMarkup({ english: 'He said \'_cats_ now.', },),
              markedPastMarkup({ english: 'He said ‘*tis* well', },),
            ],).toEqual([
              'He said ’*tis* well',
              'He said \'(tis) well',
              'He said ’"tis" well',
              'He said ’“tis” well',
              'He said ’`tis` well',
              'He said ’「tis」 well',
              'He said ’**cats** now.',
              'He said \'_cats_ now.',
              'He said ‘*tis* well',
            ],);
            expect('leading' in marksPastMarkupOf({ english: 'He said ’*tis* well', },),).toBe(false,);
          },
        },),
        it({
          name: 'READS A LONE EMPHASIS CHARACTER AS A CHARACTER OF THE TEXT, never as markup: a `*`, `_` or `~` that the parse makes '
            + 'no delimiter of emphasis, strong emphasis or strikethrough names no mark past markup',
          fn: async () => {
            expect([
              markedPastMarkup({ english: 'He said 猫_cats_’ now.', },),
              markedPastMarkup({ english: 'He said \'_cats_s now.', },),
              markedPastMarkup({ english: 'the kittens*’ bowl and *', },),
              markedPastMarkup({ english: 'the kittens_’ bowl', },),
              markedPastMarkup({ english: 'the kittens~’ bowl', },),
              markedPastMarkup({ english: 'a*’s bowl', },),
              markedPastMarkup({ english: 'He said 5 * kittens*’ now.', },),
            ],).toEqual([
              'He said 猫_cats_’ now.',
              'He said \'_cats_s now.',
              'the kittens*’ bowl and *',
              'the kittens_’ bowl',
              'the kittens~’ bowl',
              'a*’s bowl',
              'He said 5 * kittens*’ now.',
            ],);
          },
        },),
        it({
          name: 'STILL READS A DELIMITER THE PARSE MAKES CLOSING AS MARKUP, so the mark after real emphasis, strong emphasis or '
            + 'strikethrough is named, whatever stands before the emphasis',
          fn: async () => {
            expect([
              markedPastMarkup({ english: 'the *kittens*’ bowl', },),
              markedPastMarkup({ english: 'He said 猫*cats*’ now.', },),
              markedPastMarkup({ english: 'The _kittens_’ bowl', },),
              markedPastMarkup({ english: 'the **kittens**’ bowl', },),
              markedPastMarkup({ english: 'the ~~kittens~~’ bowl', },),
            ],).toEqual([
              'the *kittens*⟦’⟧ bowl',
              'He said 猫*cats*⟦’⟧ now.',
              'The _kittens_⟦’⟧ bowl',
              'the **kittens**⟦’⟧ bowl',
              'the ~~kittens~~⟦’⟧ bowl',
            ],);
          },
        },),
        it({
          name: 'NAMES A MARK BETWEEN TWO RUNS AS TRAILING ALONE, since the reading run has no leading side to name it on',
          fn: async () => {
            const { trailing, } = marksPastMarkupOf({ english: 'the *cat*’*dog* naps', },);
            expect([...trailing,],).toEqual([9,],);
          },
        },),
        it({
          name: 'READS A LONG ENGLISH OF LINK-LIKE RUNS IN ONE PASS, one that never closes a destination and one that closes every '
            + 'destination at the last bracket, so no run is walked again for each letter before it',
          fn: async () => {
            /**
             The count of link-like runs, each started by a letter.
             */
            const runs = 100_000;
            const open = `a${'](b'.repeat(runs,)}`;
            const closed = `${open}/)’ bowl`;
            const never = marksPastMarkupOf({ english: `${open}’ bowl`, },);
            const last = marksPastMarkupOf({ english: closed, },);
            expect([
              never.trailing.size,
              last.trailing.size,
              [...last.trailing,],
            ],).toEqual([0, 1, [closed.indexOf('’',),],],);
          },
        },),
      ],
    },),
  ],
},);
