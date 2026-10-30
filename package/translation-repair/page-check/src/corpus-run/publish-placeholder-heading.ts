import { isLetterOrDigitOfAnyScript, } from '@monochromatic-dev/translation-repair-markup/ts';
import { headingWords, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { PublishDefect, } from './publish-defects.ts';

//region Publish placeholder heading
// A HEADING OF PLACEHOLDER MARKS ONLY is a defect no slice floor sees. XingZ6022 (2026-10-29) shipped `### Part Nine: __`
// over the original's `### 其九：空白` ("Part nine: blank"): the archive, a hand translation, held `### __`, both lanes
// fell back to it, and the series pass wrote the original's number onto it (B1052). The owner's answer of 2026-10-29,
// "Report it as a defect (Recommended)": the page ships, no wording changes, and the defect is reported, as the
// ruling of 2026-09-27 ("Ship with the defect") asks.
//
// READ OFF THE PAGE AND THE ORIGINAL, like `pageWithoutBodyDefects` (B1046), and not through a check that throws. A
// heading has words when its title holds a letter or a digit; the title is what follows the first colon when the text
// before that colon holds a letter or a digit (the number of a series), else the whole heading. So `Part Nine: __`
// has the title `__`, and `Part Nine:` has the title `Part Nine:` because nothing follows its colon.
//
// ONLY TOP-LEVEL HEADINGS ARE READ, the way `assertHeadingsStayDistinct` reads them: a heading inside an HTML block
// such as `<details>` belongs to that block in the parse, so a placeholder there is not reported.

/**
 Node kind the parser gives a heading.
 */
const HEADING_KIND = 'heading';

/**
 Whether a text holds a letter or a digit in any script, which is what makes a title words.

 @param text - text to scan

 @returns Whether any character is a letter or a decimal digit

 @example
 ```ts
 const words = hasWords({ text: '其九', },); // true
 ```
 */
function hasWords({ text, }: { readonly text: string; },): boolean {
  for (const character of text) {
    if (isLetterOrDigitOfAnyScript({ character, },))
      return true;
  }
  return false;
}

/**
 Where the number of a series ends: the first colon, in either width.

 @param words - heading words, marks stripped

 @returns Index of the colon, or minus one where there is none

 @example
 ```ts
 const colon = seriesColonOf({ words: 'Part Nine: __', },); // 9
 ```
 */
function seriesColonOf({ words, }: { readonly words: string; },): number {
  /**
   First colon of the narrow width, minus one where there is none.
   */
  const narrow = words.indexOf(':',);
  /**
   First colon of the wide width, minus one where there is none.
   */
  const wide = words.indexOf('：',);
  if (narrow === (-1))
    return wide;
  return (wide === (-1)) ? narrow : Math.min(
    narrow,
    wide,
  );
}

/**
 Title of a heading: what follows the number of a series, else the whole heading.

 @param words - heading words, marks stripped

 @returns Text the heading's words are read from

 @example
 ```ts
 const title = titleOf({ words: 'Part Nine: __', },); // '__'
 ```
 */
function titleOf({ words, }: { readonly words: string; },): string {
  /**
   Where the number of a series would end.
   */
  const colon = seriesColonOf({ words, },);
  if (colon === (-1))
    return words;
  // Heading words arrive folded, so a colon that ends them has nothing after it, and the whole heading is the title.
  if (colon === (words.length - 1))
    return words;
  /**
   Text before that colon, which is the number of a series only if it holds words.
   */
  const number = words.slice(
    0,
    colon,
  );
  return hasWords({ text: number, },) ? words.slice(colon + 1,) : words;
}

/**
 For every heading of one document, in document order, whether it holds placeholder marks only.

 @param text - whole document, front matter included

 @returns One entry per heading, true where the heading has no words

 @example
 ```ts
 const marks = placeholderMarksOf({ text: '## 简介\n\n## __\n', },); // [false, true]
 ```
 */
function placeholderMarksOf({ text, }: { readonly text: string; },): readonly boolean[] {
  return parseDocument({ text, },)
    .nodes
    .filter(function isHeading(node,): boolean {
      return node.kind === HEADING_KIND;
    },)
    .map(function hasNoWords(node,): boolean {
      return !hasWords({ text: titleOf({ words: headingWords({ text: node.text, },), },), },);
    },);
}

/**
 Positions, from one, of the entries that are true.

 @param marks - one entry per heading

 @returns One-based positions of the true entries

 @example
 ```ts
 const positions = positionsOf({ marks: [false, true, true,], },); // [2, 3]
 ```
 */
function positionsOf({ marks, }: { readonly marks: readonly boolean[]; },): readonly number[] {
  return marks.flatMap(function positionOf(
    mark,
    index,
  ): readonly number[] {
    return mark ? [index + 1,] : [];
  },);
}

/**
 Reports a page heading made of placeholder marks only (underscores, dashes, blanks, an ellipsis or nothing) where the
 original heading has words (owner, 2026-10-29, "Report it as a defect (Recommended)", B1052).

 PAIRED BY POSITION where the original and the page hold as many headings, so a heading the original also leaves
 without words is no defect. Where the counts differ nothing is paired, and the page is reported when it holds more such
 headings than the original does. The message names page heading positions and counts only, and quotes no text.

 @param entryId - entry the page belongs to

 @param sourceText - whole original page

 @param pageText - page that would be written

 @returns One defect naming the page headings, or none

 @example
 ```ts
 const defects = placeholderHeadingDefects({ entryId: 'BookshopCat', sourceText, pageText, },);
 ```
 */
export function placeholderHeadingDefects(
  {
    entryId,
    sourceText,
    pageText,
  }: {
    readonly entryId: string;
    readonly sourceText: string;
    readonly pageText: string;
  },
): readonly PublishDefect[] {
  /**
   Whether each heading of the original holds marks only.
   */
  const sourceMarks = placeholderMarksOf({ text: sourceText, },);
  /**
   Whether each heading of the page holds marks only.
   */
  const pageMarks = placeholderMarksOf({ text: pageText, },);
  /**
   Whether the headings of the two documents can be paired by position.
   */
  const paired = sourceMarks.length === pageMarks.length;
  /**
   Page headings reported, one-based.
   */
  const reported = positionsOf({
    marks: paired
      ? pageMarks.map(function onlyWhereSourceHasWords(
        mark,
        index,
      ): boolean {
        return mark && (sourceMarks[index] !== true);
      },)
      : pageMarks,
  },);
  /**
   Marks-only headings the original holds, which the unpaired reading weighs the page's against.
   */
  const { length: sourceCount, } = positionsOf({ marks: sourceMarks, },);
  if (reported.length === 0)
    return [];
  if ((!paired) && (reported.length <= sourceCount))
    return [];
  /**
   Page headings as the message names them.
   */
  const where = reported.map(function named(position,): string {
    return `page heading ${String(position,)}`;
  },)
    .join(', ',);
  if (paired) {
    return [{
      check: 'placeholder-heading',
      message: (reported.length === 1)
        ? `entry ${entryId} ships a heading of placeholder marks only where the original heading has words, at ${where}`
        : `entry ${entryId} ships ${
          String(reported.length,)
        } headings of placeholder marks only where the original headings have words, at ${where}`,
    },];
  }
  /**
   The headings reported, counted with their noun.
   */
  const shown = `${String(reported.length,)} ${(reported.length === 1) ? 'heading' : 'headings'}`;
  /**
   The original's headings, counted with their noun and the verb that follows it.
   */
  const unpaired = `${String(sourceMarks.length,)} ${
    (sourceMarks.length === 1) ? 'heading is' : 'headings are'
  }`;
  return [{
    check: 'placeholder-heading',
    message: `entry ${entryId} ships ${shown} of placeholder marks only against ${
      String(sourceCount,)
    } in the original, whose ${unpaired} not paired with the page's ${String(pageMarks.length,)}, at ${where}`,
  },];
}

//endregion Publish placeholder heading
