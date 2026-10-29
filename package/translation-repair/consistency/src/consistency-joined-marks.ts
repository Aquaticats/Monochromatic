import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { closingJointsOf, } from './consistency-closing-links.ts';
import {
  foldWithOffsets,
  offsetsInFold,
} from './consistency-fold-offsets.ts';
import { foldedLinksOf, } from './consistency-folded-links.ts';
import { WRITTEN_APOSTROPHES, } from './consistency-markup-runs.ts';
import {
  type EmphasisDelimiters,
  emphasisDelimitersOf,
} from './consistency-prose-text.ts';

//region Consistency joined marks
// THE APOSTROPHES THE PROOF READS AS JOINED TO THE WORD AFTER THEM, made from the reading the cut of a word makes
// (`closingJointsOf`, `consistency-closing-links.ts`): the same fold, the same links of the English and the same twin of the
// English a reader sees, so the proof and the cut agree about one unit. The proof reads an English alone unless the caller hands
// it the page's definitions, so it holds the links of that English and, with the definitions, those of the page; a reference
// whose definition stands in another slice is plain text to it where they are not handed.

/**
 The apostrophes of an English that join the letter or digit after them across closing marks, at the offsets of the English
 as it was written: the `'` in `**cat**'s`, `"cat"'s`, `(grey cat)'s` and `[the cat](x)'s`. It is the reading
 `closingJointsOf` makes for the cut of a word, made once for the whole English with the links the parse holds, so a reader of
 those marks as the character that opens a quotation reads the one the cut reads: a link that shows nothing is not there
 (`see [](u)'kit` reads as `see 'kit`, whose apostrophe opens a word) and a link that shows closing marks and nothing else closes
 as those marks do (`[\]](u)'kit` reads as `]'kit`).

 @param english - English as it was written

 @param emphasis - delimiters of the English's emphasis by the parse, which the caller may hold already (`proseViewOf`);
 read from the English, with the definitions, where absent

 @param definitions - page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: a reference
 in the English whose definition stands in another slice is then plain text, and its brackets and its `*` pair as plain text do
 (n173). They are read with the English where the delimiters are, and with its links, so the two agree

 @returns Offsets of the apostrophes that stand after a run of closing marks and before a letter or a digit

 @example
 ```ts
 const joined = joinedMarksOf({ english: 'Fill the **cat**\'s bowl.', },); // holds 16
 ```
 */
export function joinedMarksOf(
  {
    english,
    emphasis,
    definitions = '',
  }: {
    readonly english: string;
    readonly emphasis?: EmphasisDelimiters;
    readonly definitions?: string;
  },
): ReadonlySet<number> {
  /**
   The offsets recorded, mutated only inside this function.
   */
  const marks = new Set<number>();
  // An English with no apostrophe-like character has none to read.
  if (![...WRITTEN_APOSTROPHES,].some(function isHeld(mark,): boolean {
    return english.includes(mark,);
  },))
    return marks;
  /**
   The English in the fold the cut of a word reads it in, with the offsets of the original each unit stands for.
   */
  const read = foldWithOffsets({ text: english, },);
  /**
   The folded English, where each folded unit starts in the original, and the original's units.
   */
  const {
    folded,
    starts,
    originals,
  } = read;
  for (
    const at of closingJointsOf({
      folded,
      originals,
      closing: offsetsInFold({
        offsets: (emphasis ?? emphasisDelimitersOf({
          english,
          definitions,
        },)).closing,
        folded: read,
      },),
      links: foldedLinksOf({
        english,
        folded: read,
        definitions,
      },),
    },)
  ) {
    if (WRITTEN_APOSTROPHES.has(originals[at] ?? '',)) {
      // A joint stands only at a unit of the fold, and every unit maps to the original.
      marks.add(nonNullishOrThrow(starts[at],),);
    }
  }
  return marks;
}

//endregion Consistency joined marks
