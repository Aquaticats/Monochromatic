import {
  carriesAsciiLetter,
  closedMarkSpans,
  isHanCharacter,
  titleText,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { straightenQuotes, } from '@monochromatic-dev/translation-repair-typography/ts';
import { withoutHtmlComments, } from './translate-address-drop.ts';

//region Latin title floor
// CLASS ONE HUNDRED FORTY-ONE (XingZ6012, 2026-09-26). The second song
// credit of the XingZ60 tail shipped "— Yuli【妄想症Paranoia】《Zero-Layer
// Prayer》": the title rendered, the Chinese title marks kept around it.
// 《》 are Chinese punctuation with no meaning in English prose, where a
// song's title stands in quotation marks, the way every other credit on the
// page reads (——from “Bird in a Cage”), and a longer work's in italics
// (owner, 2026-09-27). Read pages had shipped 《Nonuple
// Reality》, 《Zero-Layer Prayer》 and the original's own English poem title
// in the marks. A candidate bracketing a title that carries Latin letters
// and no Han in 《》 is refused HERE before any judge reads it, unless the
// page it would replace writes the same bracketed title (the XingZ60 archive
// keeps one such line), curly and straight quotation marks counting as one
// (ledger B24). A bracketed title that keeps Han (《舞萌DX》) is left
// to the Han residue floor (`translate-han-residue.ts`, ledger F-3), which
// refuses its Han unless an English gloss stands beside it, and to the
// judges; comments are cut.

/**
 Opening title mark.
 */
const TITLE_OPEN = '《';

/**
 Closing title mark.
 */
const TITLE_CLOSE = '》';

/**
 Newline, which no title crosses.
 */
const LINE_END = '\n';

/**
 Whether a title is written in Latin letters with no Han.

 @param title - text that names the work

 @returns True where the title carries a Latin letter and no Han ideograph

 @example
 ```ts
 isLatinOnly({ title: 'Meow Song', },); // true
 ```
 */
function isLatinOnly({ title, }: { readonly title: string; },): boolean {
  for (const character of title) {
    if (isHanCharacter({ character, },))
      return false;
  }
  return carriesAsciiLetter({
    text: title,
    from: 0,
    to: title.length,
  },);
}

/**
 Every distinct 《》-bracketed span of a text whose title is in Latin letters
 alone, brackets included, in order of first appearance.

 @param text - candidate with its comments cut

 @returns Bracketed spans as written

 @example
 ```ts
 latinBracketed({ text: '— Maomao 《Meow Song》', },); // ['《Meow Song》']
 ```
 */
function latinBracketed({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Spans read so far.
   */
  const spans: string[] = [];
  // A MARK THAT NEVER CLOSED brackets nothing (ledger B38).
  for (const marks of closedMarkSpans({
    text,
    open: TITLE_OPEN,
    close: TITLE_CLOSE,
  },)) {
    /**
     Text between the marks.
     */
    const bracketed = text.slice(
      marks.open + TITLE_OPEN.length,
      marks.close,
    );
    /**
     Span with its marks.
     */
    const span = `${TITLE_OPEN}${bracketed}${TITLE_CLOSE}`;
    /**
     Whether the span stays on one line and is read for the first time.
     */
    const fresh = !(bracketed.includes(LINE_END,) || spans.includes(span,));
    if (fresh && isLatinOnly({ title: titleText({ bracketed, },), },))
      spans.push(span,);
  }
  return spans;
}

/**
 Findings against a candidate that sets a title written in Latin letters in
 the Chinese title marks 《》, one per title; empty where no such title stands
 or the page the candidate would replace writes it the same way.

 @param candidateText - rendering under the floor

 @param pageText - text the rendering would replace, empty where the page
 has none

 @returns Findings, empty where the candidate passes

 @example
 ```ts
 latinTitleFindings({ candidateText: '— Maomao 《Meow Song》', pageText: '', },); // one finding
 ```
 */
export function latinTitleFindings(
  {
    candidateText,
    pageText = '',
  }: {
    readonly candidateText: string;
    readonly pageText?: string;
  },
): readonly string[] {
  /**
   Page as the typography fold reads it: a title the page writes with a
   curly apostrophe is the one a candidate writes with a straight one, since
   the typography restoration makes them one (ledger B24).
   */
  const foldedPage = straightenQuotes({ text: pageText, },);
  return latinBracketed({ text: withoutHtmlComments({ text: candidateText, },), },)
    .filter(function unkeptByPage(span,): boolean {
      return !foldedPage.includes(straightenQuotes({ text: span, },),);
    },)
    .map(function toFinding(span,): string {
      /**
       Title as it names the work.
       */
      const title = titleText({ bracketed: span.slice(
        TITLE_OPEN.length,
        -TITLE_CLOSE.length,
      ), },);
      return `Your translation sets the English title ${span} in the Chinese title marks 《》, which mean nothing in English prose: an English page sets a work's title in quotation marks, so write “${title}” (keeping any link on the title).`;
    },);
}

//endregion Latin title floor
