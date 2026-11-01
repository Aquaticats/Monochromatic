import {
  parseSliceBody,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Consistency block breaks
// HOW MANY TIMES SOME WORDS BREAK INTO A NEW BLOCK. A replacement of words by words
// that hold another number of block breaks changes the number of blocks of the page,
// and the structure floor that every candidate page must pass
// (`translate-validate-blocks.ts`, "Every block of the PAGE AS IT STANDS has to appear in
// your translation") refuses a page that has lost one. So a splice between two such
// places can never pass the floors, and the planner (`consistency-splice.ts`) calls it
// unavailable instead of offering a splice that is always refused. A block break here is
// a blank line, or a line break that the Markdown grammar of the slice
// (`parse-slice-body.ts`) reads as the start of another block (a list item, a heading,
// a block quote after a paragraph's line). The words are read alone, as a fragment, the
// way the grammar reads them: the number of top-level blocks it finds, less one.
//
// WORDS THE GRAMMAR REFUSES ARE COUNTED BY THEIR BLANK LINES. The grammar is strict
// and words with a stray brace are refused; then only the blank lines can be counted,
// a run of them between two lines of text being one break.

/**
 The characters a blank line may hold: a space, a tab and the carriage return of a Windows line ending.
 */
const BLANK_CHARACTERS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\r',
],);

/**
 The blank lines of some words, counted as the breaks of a text no parse is to be had of:
 a run of blank lines between two lines that hold something is one break.

 @param text - the words

 @returns The number of runs of blank lines that stand between two lines of text

 @example
 ```ts
 const breaks = blankLineBreaksIn({ text: 'One\n\n\nTwo', },); // 1
 ```
 */
function blankLineBreaksIn({ text, }: { readonly text: string; },): number {
  /**
   The count so far, and whether a line of text and then a blank line have been read.
   */
  const reading = {
    breaks: 0,
    sawText: false,
    pendingBlank: false,
  };
  for (const line of text.split('\n',)) {
    // A line of nothing but spaces, tabs and a carriage return is blank, as the Markdown grammar reads it.
    if (Array.from(line,)
      .every(function isBlank(character,): boolean {
        return BLANK_CHARACTERS.has(character,);
      },)) {
      reading.pendingBlank = reading.sawText;
      continue;
    }
    if (reading.pendingBlank)
      reading.breaks += 1;
    reading.sawText = true;
    reading.pendingBlank = false;
  }
  return reading.breaks;
}

/**
 How many block breaks some words hold.

 @param text - the words a place's quote names

 @returns The number of top-level blocks the Markdown grammar finds in the words, less one; the runs of blank lines
 between lines of text when the grammar refuses the words; none for words on one line

 @throws {@link Error} when reading fails for a reason other than the grammar's own refusal

 @example
 ```ts
 const breaks = blockBreaksIn({ text: 'The kitten naps\n\nin the sun', },); // 1
 ```
 */
export function blockBreaksIn({ text, }: { readonly text: string; },): number {
  if (!text.includes('\n',))
    return 0;
  try {
    /**
     The tree of the words as the grammar reads them.
     */
    const { root, } = parseSliceBody({ text, },);
    /**
     The top-level blocks of the tree.
     */
    const { children, } = root;
    return Math.max(
      0,
      children.length - 1,
    );
  }
  catch (error) {
    // Only the grammar's own refusal is words to be counted by their blank lines; anything else keeps propagating.
    requireMdxRefusal({ error, },);
    return blankLineBreaksIn({ text, },);
  }
}

//endregion Consistency block breaks
