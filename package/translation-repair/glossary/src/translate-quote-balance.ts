import { isAsciiDigit, } from '@monochromatic-dev/translation-repair-markup/ts';
import { withoutHtmlComments, } from './translate-address-drop.ts';

//region Closing quotation marks with no opening one
// CLASS ONE HUNDRED SIXTY-FIVE (TianqiChen6665, 2026-09-26). The original's
// final message quotes each of its three lines with 「」, and the archive's
// rendering closed the last line with ” and opened none; the page shipped it
// on an unendorsed standing. A quotation mark that closes nothing is a
// broken sentence on the page whatever the judges read, so a candidate
// carrying one is refused before any judge reads it, which also makes such
// an archive rendering an ineligible standing. THE FLOOR STANDS ASIDE where
// the original itself closes a quotation it never opened (a slice boundary
// inside a quotation), since the rendering then owes the same shape. An
// opening mark with no closing one is no fault: a quotation over several
// paragraphs opens each and closes only the last. Single marks are left
// alone, since an apostrophe is a single closing mark.
//
// CLASS ONE HUNDRED SEVENTY-FIVE (TianqiChen66610 slice 16, 2026-09-26): the
// same stray closer shipped as a straight double quote ("rely on.""), which
// this floor left alone and the typography restore left straight on its odd
// count. A straight double quote is read by its shape: one between a word or
// punctuation and a space or line end closes, one between a space or line
// start and a word opens, one after a digit is an inch mark and reads as
// neither, and any other shape is left unread.

/**
 Closing mark of each quotation pair, keyed by its opening mark. A map, as
 every table keyed by text is (ledger B77), though a key here is one unit and
 no name an object inherits is.
 */
const CLOSER_OF: ReadonlyMap<string, string> = new Map([
  [
    '“',
    '”',
  ],
  [
    '「',
    '」',
  ],
  [
    '『',
    '』',
  ],
],);

/**
 Opening mark of each quotation pair, keyed by its closing mark.
 */
const OPENER_OF: ReadonlyMap<string, string> = new Map([
  [
    '”',
    '“',
  ],
  [
    '」',
    '「',
  ],
  [
    '』',
    '『',
  ],
],);

/**
 Straight double quote the scan reads as a curly one by its shape.
 */
const STRAIGHT_DOUBLE = '"';

/**
 Characters a closing quotation mark may stand before without a space.
 */
const CLOSE_FOLLOWERS: ReadonlySet<string> = new Set([
  ',',
  '.',
  ';',
  ':',
  '!',
  '?',
  ')',
  ']',
  '…',
],);

/**
 Characters an opening quotation mark may stand after without a space.
 */
const OPEN_PRECEDERS: ReadonlySet<string> = new Set([
  '(',
  '[',
  '>',
  '*',
  '_',
],);

/**
 Whether a character is a space, a line break, or the text's edge.

 @param character - character to test, empty at the edge

 @returns Whether nothing prints there

 @example
 ```ts
 isBlank({ character: '', },); // true
 ```
 */
function isBlank({ character, }: { readonly character: string; },): boolean {
  return character.trim() === '';
}

/**
 What a straight double quote does by its shape: open a quotation, close
 one, or neither where the shape does not say.

 @param text - passage scanned

 @param index - position of the straight double quote

 @returns The curly mark it stands for, or `unread`

 @example
 ```ts
 straightDoubleRole({ text: 'on."', index: 3, },); // '”'
 ```
 */
function straightDoubleRole(
  {
    text,
    index,
  }: {
    readonly text: string;
    readonly index: number;
  },
): '“' | '”' | 'unread' {
  /**
   Character before the quote, empty at the start.
   */
  const before = (index === 0) ? '' : text.charAt(index - 1,);
  /**
   Character after the quote, empty at the end.
   */
  const after = text.charAt(index + 1,);
  // A digit before it makes it an inch mark, not a quotation.
  if (isAsciiDigit({ character: before, },))
    return 'unread';
  /**
   Whether the side before prints nothing or is opening punctuation.
   */
  const openSide = isBlank({ character: before, },) || OPEN_PRECEDERS.has(before,);
  /**
   Whether the side after prints nothing or is closing punctuation.
   */
  const closeSide = isBlank({ character: after, },) || CLOSE_FOLLOWERS.has(after,);
  // Both sides open or both close: the shape does not say which it is.
  if (openSide === closeSide)
    return 'unread';
  return openSide ? '“' : '”';
}

/**
 Closing marks in a text that close no quotation opened before them, in
 text order.

 @param text - passage to scan, comments already cut

 @returns Each stray closing mark

 @example
 ```ts
 strayClosers({ text: '猫说。」', },); // ['」']
 ```
 */
function strayClosers({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Open quotations of each kind so far.
   */
  const depth = new Map<string, number>();
  /**
   Closing marks met with nothing of their kind open.
   */
  const strays: string[] = [];
  // Code-unit scan: every quotation mark read here is one BMP code unit, so
  // surrogate halves and combining marks read as neither opener nor closer.
  for (let index = 0; index < text.length; index += 1) {
    /**
     Code unit under the cursor, a straight double quote read as the curly
     mark its shape stands for.
     */
    const character = (text.charAt(index,) === STRAIGHT_DOUBLE)
      ? straightDoubleRole({
        text,
        index,
      },)
      : text.charAt(index,);
    if (CLOSER_OF.has(character,)) {
      depth.set(
        character,
        (depth.get(character,) ?? 0) + 1,
      );
      continue;
    }
    /**
     Opening mark this character would close.
     */
    const opener = OPENER_OF.get(character,);
    if (opener === undefined)
      continue;
    /**
     Quotations of this kind still open.
     */
    const open = depth.get(opener,) ?? 0;
    if (open === 0) {
      // The mark as the text writes it, so the finding names what to repair.
      strays.push(text.charAt(index,),);
      continue;
    }
    depth.set(
      opener,
      open - 1,
    );
  }
  return strays;
}

/**
 Findings for a candidate carrying a closing quotation mark with no opening
 mark before it, where the original's own marks close only what they open.

 @param sourceText - original slice

 @param candidateText - candidate under validation

 @returns One finding naming the stray marks, or none

 @example
 ```ts
 const findings = strayClosingQuoteFindings({ sourceText: '「猫。」', candidateText: 'The cat.”', },);
 ```
 */
export function strayClosingQuoteFindings(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Closing marks in the original that close nothing.
   */
  const sourceStrays = strayClosers({ text: withoutHtmlComments({ text: sourceText, },), },);
  if (sourceStrays.length > 0)
    return [];
  /**
   Closing marks in the candidate that close nothing.
   */
  const strays = strayClosers({ text: withoutHtmlComments({ text: candidateText, },), },);
  if (strays.length === 0)
    return [];
  /**
   Stray marks as the finding names them.
   */
  const named = strays.join(' ',);
  /**
   Finding telling the writer what to repair.
   */
  const finding = `Your translation closes a quotation it never opened (${named}): the ORIGINAL opens every `
    + 'quotation it closes. Open each quoted passage where the ORIGINAL opens it, and close it where the '
    + 'ORIGINAL closes it.';
  return [finding,];
}

//endregion Closing quotation marks with no opening one
