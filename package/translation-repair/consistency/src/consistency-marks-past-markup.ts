import {
  closingRunMarksOf,
  WRITTEN_APOSTROPHES,
} from './consistency-markup-runs.ts';
import {
  type EmphasisDelimiters,
  emphasisDelimitersOf,
} from './consistency-prose-text.ts';
import type { MarksPastMarkup, } from './consistency-proven-apostrophes.ts';

//region Consistency marks past markup
// A MARK PAST MARKUP IS AN EDGE MARK FOR THE PROOF, read over the reading run of `consistency-markup-runs.ts`. The proof reads
// the prose, where emphasis is blanked to spaces, so `The *kittens*’ bowl` shows it a mark with a space on both sides, a free
// mark that could open a quotation and so stops every later plural possessive of the page from being proven. The mark ends a
// run of closing delimiters that a Latin letter stands before, the way `kittens’` ends a word: the markup is no part of the
// word, and the proof reads the mark as a trailing one.
//
// THE READING RUN HOLDS ONLY CLOSING DELIMITERS OF EMPHASIS, STRONG EMPHASIS AND STRIKETHROUGH BY THE PARSE, AND A LINK'S CLOSING
// `](…)`, ON THE TRAILING SIDE ALONE. A `*`, `_` or `~` that the parse makes no delimiter is a character of the text, so
// `猫_cats_’`, `the kittens*’ bowl and *` and `a*’s bowl` name no mark past markup. A closing double quotation mark, a closing
// bracket and inline code are not in the run: after `"ok"'` or `(here)’` a single mark is most often a closing quotation mark.
// There is no leading mirror: a mark before opening markup (`’**cats**`, `'_cats_`) is read where it stands. Only a free mark is
// re-read, and the caller applies the reading to it (`consistency-proven-apostrophes.ts`): a mark with a letter directly beside
// it keeps its reading, and a mark with a single mark directly beside it (D5) or with a character after it that does not part
// it from a word (D3: a letter of another script, a symbol, a pictograph) stays free whatever run it ends. A bare leading mark
// that no later mark could close still reads as an elision (D9, an accepted limit).
//
// THE READING IS MADE ONCE FOR A WHOLE TEXT, RIGHT TO LEFT, and asked in constant time. A run of closing delimiters that starts
// at an offset ends in such an apostrophe exactly when the run that starts one unit later does (or, after the `]` of a link,
// the run that starts past the link's destination), so one pass from the end of the text to its start records the answer for
// every offset from the answer for the offsets after it (`closingRunMarksOf`). The reading has no limit: a run of any number of
// delimiters and a link destination of any length are read the same way, each character a constant number of times.

/**
 The apostrophe-like marks of an English that stand past markup, so the proof can read the markup as no part of the word: the
 marks that end a reading run (closing delimiters of emphasis by the parse, the destination of a link) that a Latin letter or a
 digit stands directly before (`*kittens*’`). The reading says nothing of what stands directly beside the mark: the caller
 applies it to a mark that no letter stands directly beside, which is a free mark.

 One pass reads the text from its end to its start, a constant number of reads for each unit, whatever the length of a run or of
 a destination.

 @param english - the English as it was written, where the markup is not blanked yet

 @param emphasis - the delimiters of the English's emphasis, strong emphasis and strikethrough by the parse, which the caller may
 hold already (`proseViewOf`); read from the English, with the definitions, where absent. A `*`, `_` or `~` that is none of them
 is a character of the text and no part of a run.

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: a reference
 in the English whose definition stands in another slice is then plain text, and an unresolved `[b*]` has a `*` that can pair with
 an outer one (n173)

 @returns The offsets of the marks past closing markup

 @example
 ```ts
 const { trailing, } = marksPastMarkupOf({ english: 'The *kittens*’ bowl', },); // trailing holds 13
 ```
 */
export function marksPastMarkupOf(
  {
    english,
    emphasis,
    definitions = '',
  }: {
    readonly english: string;
    readonly emphasis?: EmphasisDelimiters;
    readonly definitions?: string;
  },
): MarksPastMarkup {
  // An English with no apostrophe-like character has none to read.
  if (![...WRITTEN_APOSTROPHES,].some(function isHeld(mark,): boolean {
    return english.includes(mark,);
  },))
    return { trailing: new Set(), };
  return {
    trailing: closingRunMarksOf({
      text: english,
      closing: (emphasis ?? emphasisDelimitersOf({
        english,
        definitions,
      },)).closing,
    },),
  };
}

//endregion Consistency marks past markup
