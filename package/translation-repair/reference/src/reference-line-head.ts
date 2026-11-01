import { isWholeNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Reference line head
// The head every cited-reference line opens with, written by the lookup and
// read back by the attestation, which looks for a reference quote in the one
// line of the reference an item names (ledger B25), and by the archive block
// review, which anchors a retention in what one page says (ledger B28).
// Writer and readers share this module so a reader cannot drift from what is
// written.

/**
 What every reference line opens with, before its number.
 */
const REFERENCE_LINE_MARK = '- reference ';

/**
 Between reference lines in a block.
 */
const LINE_BREAK = '\n';

/**
 What `indexOf` answers when nothing is found.
 */
const NOT_FOUND = -1;

/**
 What the lookup writes after a head when the page could not be fetched,
 before the endpoint's own failure text.
 */
export const REFERENCE_UNFETCHED = 'could not be fetched';

/**
 What the lookup writes after a head when the page held no readable text.
 */
export const REFERENCE_UNREADABLE = 'nothing readable on the page';

/**
 Head of one reference line: the mark, the position and the page, before
 the title and text the lookup writes after it.

 @param index - position among the entry's references, from one

 @param url - page as the original links it

 @returns Head, without what follows it

 @example
 ```ts
 referenceLineHead({ index: 2, url: 'https://cats.example/b', },);
 // => '- reference 2 https://cats.example/b'
 ```
 */
export function referenceLineHead(
  {
    index,
    url,
  }: {
    readonly index: number;
    readonly url: string;
  },
): string {
  return `${REFERENCE_LINE_MARK}${String(index,)} ${url}`;
}

/**
 One reference line read back with the number its head carries.

 @example
 ```ts
 const line: NumberedReferenceLine = { reference: 1, line: '- reference 1 https://cats.example/a: Mittens naps.', };
 ```
 */
export type NumberedReferenceLine = {
  /**
   Number the head carries, from one.
   */
  readonly reference: number;
  /**
   Whole line, head included.
   */
  readonly line: string;
};

/**
 A line in a reference block that does not open with a numbered head. The
 lookup writes one line per reference and folds every text it puts on it,
 so such a line is a defect upstream, not text to guess a number for.

 @example
 ```ts
 throw new ReferenceLineHeadError({ position: 2, count: 3, },);
 ```
 */
export class ReferenceLineHeadError extends Error {
  /**
   Declares this message safe to forward: it names a line's position, the
   block's line count and the mark this module writes, never the line.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal naming where the line sits, never what it says, since
   a reference line carries a fetched page's text.

   @param position - which line of the block, from one

   @param count - lines in the block

   @example
   ```ts
   new ReferenceLineHeadError({ position: 2, count: 3, },);
   ```
   */
  constructor({
    position,
    count,
  }: {
    readonly position: number;
    readonly count: number;
  },) {
    super(
      `reference line ${String(position,)} of ${String(count,)} does not open with "${REFERENCE_LINE_MARK}<number> ", `
        + 'so the reference it belongs to cannot be read; the block must be one referenceLineOf line per reference',
    );
    this.name = 'ReferenceLineHeadError';
  }
}

/**
 Lines of a reference block, each with the number its head carries. An
 empty block is an original that links nowhere and has no lines.

 @param referenceContext - reference lines as `citedReferenceBlock` joins them

 @returns Lines in block order

 @throws ReferenceLineHeadError when a line does not open with a numbered head

 @example
 ```ts
 numberedReferenceLines({ referenceContext: '- reference 1 https://cats.example/a: Mittens naps.', },);
 // => [{ reference: 1, line: '- reference 1 https://cats.example/a: Mittens naps.', }]
 ```
 */
export function numberedReferenceLines(
  { referenceContext, }: { readonly referenceContext: string; },
): readonly NumberedReferenceLine[] {
  if (referenceContext === '')
    return [];
  /**
   Lines as written.
   */
  const lines = referenceContext.split(LINE_BREAK,);
  return lines.map(function numbered(
    line,
    at,
  ): NumberedReferenceLine {
    /**
     Line after the mark, opening with the number; empty without the mark.
     */
    const rest = line.startsWith(REFERENCE_LINE_MARK,) ? line.slice(REFERENCE_LINE_MARK.length,) : '';
    /**
     Where the number ends.
     */
    const numberEnd = rest.indexOf(' ',);
    /**
     Number as written, empty when there is no head.
     */
    const digits = (numberEnd === NOT_FOUND) ? '' : rest.slice(
      0,
      numberEnd,
    );
    if (!isWholeNumberText({ text: digits, },)) {
      throw new ReferenceLineHeadError({
        position: at + 1,
        count: lines.length,
      },);
    }
    return {
      reference: Number(digits,),
      line,
    };
  },);
}

/**
 What one reference line says its page states: the title and the text the
 lookup wrote after the head, without the mark, the number or the address.

 READ WHERE A QUOTE IS TO BE FOUND ON A PAGE (ledger B28, B104): an address,
 the head's number and the lookup's own note on a page it could not fetch or
 read are no page's statement, so a page with nothing readable states nothing.
 An address holds no space, so the first space after it ends it.

 @param line - line with the number its head carries, as
 `numberedReferenceLines` reads it

 @returns What the page states, empty where it states nothing

 @example
 ```ts
 const line = { reference: 1, line: '- reference 1 https://cats.example/a ("Naps"): Mittens naps.', };
 referenceLinePageText({ line, },);
 // => '("Naps"): Mittens naps.'
 ```
 */
export function referenceLinePageText({ line, }: { readonly line: NumberedReferenceLine; },): string {
  /**
   The line as written, and the number its head carries.
   */
  const {
    line: written,
    reference,
  } = line;
  /**
   Line after its number: the address, then what the page says.
   */
  const afterNumber = written.slice(`${REFERENCE_LINE_MARK}${String(reference,)} `.length,);
  /**
   Where the address ends: a space opens the title, or follows the colon
   after an address the page gave no title.
   */
  const addressEnd = afterNumber.indexOf(' ',);
  if (addressEnd === NOT_FOUND)
    return '';
  /**
   What follows the address.
   */
  const text = afterNumber.slice(addressEnd + 1,);
  if (text.startsWith(REFERENCE_UNFETCHED,) || (text === REFERENCE_UNREADABLE))
    return '';
  return text;
}

/**
 What each cited page says, as the lookup wrote it after the line's head.
 Lines that do not open with the reference mark are skipped, since the block a
 sheet carries puts the attestation's lines under the pages and each of those
 quotes the archive; a page the lookup could not fetch or read says nothing.

 READ WHERE A QUOTE IS TO BE FOUND ON A PAGE (ledger B28): the archive block
 review anchors a retention in what a cited page states, read by
 `referenceLinePageText` as the attestation reads it.

 @param referenceContext - reference lines, with any attested lines under them

 @returns Page texts in block order

 @throws ReferenceLineHeadError when a line opening with the mark has no number

 @example
 ```ts
 referencePageTexts({ referenceContext: '- reference 1 https://cats.example/a ("Naps"): Mittens naps.', },);
 // => ['("Naps"): Mittens naps.']
 ```
 */
export function referencePageTexts(
  { referenceContext, }: { readonly referenceContext: string; },
): readonly string[] {
  /**
   Lines the lookup wrote for pages, the attested lines left out.
   */
  const pageLines = referenceContext
    .split(LINE_BREAK,)
    .filter(function isPageLine(line,): boolean {
      return line.startsWith(REFERENCE_LINE_MARK,);
    },);
  return numberedReferenceLines({ referenceContext: pageLines.join(LINE_BREAK,), },)
    .flatMap(function pageText(line,): readonly string[] {
      /**
       What this page states, empty where it states nothing.
       */
      const text = referenceLinePageText({ line, },);
      return (text === '') ? [] : [text,];
    },);
}

//endregion Reference line head
