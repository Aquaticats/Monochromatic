import {
  compacted,
  referencePageTexts,
} from '@monochromatic-dev/translation-repair-reference/ts';
import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  countedCharacters,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';

import {
  isArchiveReferenceQuoteAnchored,
  isArchiveSourceQuoteAnchored,
  isVerifiableEditorialArchiveBlock,
  MINIMUM_SOURCE_QUOTE_CHARACTERS,
  shownCodePointCount,
} from './archive-block-evidence.ts';

//region Archive reply verdict
// WHY AN ARCHIVE REVIEW REPLY IS NOT EVIDENCE, said in the words the reader of a run needs (ledger B715).
//
// The review drops a `source-supported` reply whose quote is no anchor and an `editorial-context` reply
// about a block that is not wholly apparatus, and it used to drop them without a line: run zheermao20
// logged "2 eligible assessments from 4 heard reviews" and nothing said which reply failed which test.
// The decision and its explanation are one function, so the line cannot say a test failed that the
// decision passed (the "two layers reading one refusal" mistake, ledger B40).

/**
 UTF-16 units of a quote's opening a refusal line quotes, cut so it never ends inside a character.
 */
const OPENING_UNITS = 20;

/**
 The longest opening of a quote that a text holds, with whitespace set aside on both sides.

 A search by halving: if the first `k` characters of the quote are in the text, so are the first `k - 1`.

 @param quote - words a reviewer quoted

 @param text - text they were looked for in

 @returns Code points of the compacted quote's opening found in the compacted text, zero when not even the
 first is

 @example
 ```ts
 longestOpeningFound({ quote: '猫在窗边吃鱼', text: '猫在窗边睡觉', },); // 4
 ```
 */
function longestOpeningFound(
  {
    quote,
    text,
  }: {
    readonly quote: string;
    readonly text: string;
  },
): number {
  /**
   Quote's code points, whitespace gone.
   */
  const characters = Array.from(compacted({ text: quote, },),);
  /**
   Text as the comparison reads it.
   */
  const haystack = compacted({ text, },);
  /**
   Longest opening known to be found.
   */
  let found = 0;
  /**
   Shortest opening known to be missing, one past the whole quote at the start.
   */
  let missing = characters.length + 1;
  while ((missing - found) > 1) {
    /**
     Opening length tried.
     */
    const tried = Math.floor((found + missing) / 2,);
    /**
     The opening of that length.
     */
    const opening = characters.slice(
      0,
      tried,
    )
      .join('',);
    if (haystack.includes(opening,))
      found = tried;
    else
      missing = tried;
  }
  return found;
}

/**
 What the archive block review makes of one reply: it stands as evidence, or it
 is dropped for a reason a log line can state.
 */
export type ArchiveReplyVerdict = {
  /**
   The reply is evidence.
   */
  readonly kind: 'stands';
} | {
  /**
   The reply is dropped.
   */
  readonly kind: 'dropped';

  /**
   Which test refused it and the numbers that decided it.
   */
  readonly reason: string;
};

/**
 Whether a reply to the archive block review is evidence, and if not, why.

 THE DECISION IS MADE HERE and read by the stage, which logs what this returns: a `source-supported` reply
 stands when its quote shows the minimum a reader sees and is a span of the expected section with its
 picture transcripts or of one cited page (`isArchiveSourceQuoteAnchored`, `isArchiveReferenceQuoteAnchored`);
 an `editorial-context` reply stands when the block is wholly apparatus; a `revise` reply always stands,
 since a revision earns publication through the selector.

 @param disposition - what the reply claimed

 @param sourceQuote - the reply's support claim

 @param sourceContext - expected section with the picture transcripts the sheet carried

 @param referenceContext - reference lines, with any attested lines under them, empty when the original
 links nowhere

 @param blockText - the block under review

 @returns That the reply stands, or the sentence saying which test refused it and the numbers that decided it

 @throws ReferenceLineHeadError when a reference line has no numbered head

 @example
 ```ts
 archiveReplyVerdict({ disposition: 'source-supported', sourceQuote: '猫', sourceContext: '猫在窗边。', referenceContext: '', blockText: 'The cat.', },);
 ```
 */
export function archiveReplyVerdict(
  {
    disposition,
    sourceQuote,
    sourceContext,
    referenceContext,
    blockText,
  }: {
    readonly disposition: string;
    readonly sourceQuote: string;
    readonly sourceContext: string;
    readonly referenceContext: string;
    readonly blockText: string;
  },
): ArchiveReplyVerdict {
  if (disposition === 'editorial-context') {
    if (isVerifiableEditorialArchiveBlock({ blockText, },))
      return { kind: 'stands', };
    return {
      kind: 'dropped',
      reason: `the block of ${countedCharacters({ count: blockText.length, },)} is not wholly apparatus (every line it shows must `
        + 'be a contributor declaration, an apparatus label or pictures alone, or the block closed comments alone)',
    };
  }
  if (disposition !== 'source-supported')
    return { kind: 'stands', };
  // A CITED PAGE ANCHORS TOO (ledger B28): a detail the page the original links states is the translator's
  // knowledge. Only a page's own text counts, never an attested line, which quotes the archive back.
  if (isArchiveSourceQuoteAnchored({
    sourceContext,
    sourceQuote,
  },)
    || isArchiveReferenceQuoteAnchored({
      referenceContext,
      sourceQuote,
    },))
    return { kind: 'stands', };
  /**
   Characters of the quote a reader sees.
   */
  const shown = shownCodePointCount({ text: sourceQuote.trim(), },);
  if (shown < MINIMUM_SOURCE_QUOTE_CHARACTERS) {
    return {
      kind: 'dropped',
      reason: `its quote shows ${countedCharacters({ count: shown, },)} a reader sees, below the ${
        String(MINIMUM_SOURCE_QUOTE_CHARACTERS,)
      } an anchor needs`,
    };
  }
  /**
   Quote's code points with whitespace gone, as the comparison reads them.
   */
  const quoteCharacters = Array.from(compacted({ text: sourceQuote, },),);
  /**
   Section's length with whitespace gone.
   */
  const sectionLength = Array.from(compacted({ text: sourceContext, },),)
    .length;
  /**
   Text of each cited page.
   */
  const pages = referencePageTexts({ referenceContext, },);
  /**
   Longest opening of the quote each cited page holds.
   */
  const pageOpenings = pages.map(function openingIn(pageText,): number {
    return longestOpeningFound({
      quote: sourceQuote,
      text: pageText,
    },);
  },);
  /**
   Quote's opening as the line quotes it.
   */
  const opening = wholeOpening({
    text: quoteCharacters.join('',),
    units: OPENING_UNITS,
  },);
  /**
   Longest opening of the quote the expected section holds.
   */
  const sectionOpening = longestOpeningFound({
    quote: sourceQuote,
    text: sourceContext,
  },);
  return {
    kind: 'dropped',
    reason: `its quote runs ${countedCharacters({ count: quoteCharacters.length, },)} with whitespace set aside and opens ${
      JSON.stringify(opening,)
    }; it is no span of the expected section and its picture transcripts (${
      countedCharacters({ count: sectionLength, },)
    }, the longest opening of the quote found there ${String(sectionOpening,)} of ${
      String(quoteCharacters.length,)
    }) or of any of the ${String(pages.length,)} ${
      wordForCount({
        count: pages.length,
        one: 'cited page',
        many: 'cited pages',
      },)
    } (the longest opening found in one, ${
      String(Math.max(
        0,
        ...pageOpenings,
      ),)
    })`,
  };
}

//endregion Archive reply verdict
