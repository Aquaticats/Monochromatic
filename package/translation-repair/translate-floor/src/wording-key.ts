import { requireFrontMatterRefusal, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  straightenProseQuotes,
  wrapReplacementText,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { foldSoftBreaks, } from './soft-break-fold.ts';

//region Wording key
// ONE ANSWER TO "IS THIS STILL THE WORDING THAT STANDS?", for every place that
// decides whether a proposal changes anything (ledger B26).
//
// Until 2026-09-29 each place answered by bytes, or by bytes and one rewrap
// of the base, while the proposals reaching it had passed through different
// whitespace: a model's own line breaks, the semantic wrap, or none. The site
// renders a soft line break as a space. The data repository compiles every page
// with remark-math and nothing that turns a newline into a break
// (one-among-us/data `scripts/mdx.ts`), and the front end sets no white-space
// rule on the page container (one-among-us/web). So a proposal that is the
// archive with its soft breaks elsewhere publishes the page the archive already
// publishes, and shipping it reports a change nobody made. Over the stored
// artifacts 186 lane texts were such a proposal, 66 of them from runs on or
// after 2026-09-26; 490 of 2,905 stored slates carried one beside another.
//
// A LINE-STRUCTURED SLICE KEEPS ITS BREAKS. There the line-structure rule makes
// one output line per original line the producer's work, and re-lining verse
// is the repair that rule asks for, so only what `collapseKey` folds counts as
// the same wording.

/**
 Whether a line carries nothing but blockquote marks and spaces, so that its
 trailing whitespace is formatting churn rather than a Markdown hard break.

 @param line - one line, whitespace included

 @returns Whether every character is `>` or a space

 @example
 ```ts
 isBlankQuoteLine({ line: '> ', },);
 // => true
 ```
 */
function isBlankQuoteLine({ line, }: { readonly line: string; },): boolean {
  for (const character of line) {
    if ((character !== '>') && (character !== ' '))
      return false;
  }
  return true;
}

/**
 Key two candidates share when their texts differ only in trailing whitespace
 at the end of the text or on blank lines and blank quote lines, or in the
 style of their prose quotes.

 Trailing newlines vary between models for reasons no judge should be asked to
 rank, and a fresh candidate differing from the incumbent by one of them would
 otherwise be counted as replacing it. BLANK QUOTE LINES TOO, since
 2026-09-02: the Toka_ls reading found a candidate judged a replacement five
 ballots to two that differed from the archive in exactly two bytes, a
 trailing space after `>` on two blank quote lines copied from the source's
 formatting. Lines carrying content keep their trailing spaces: 65 of the
 pinned corpus's pages use Markdown hard breaks (two trailing spaces before a
 newline; saurikissa's archive has 35 such lines), and those are content.
 Leading whitespace is NOT stripped, since Markdown list indentation is
 content.

 PROSE QUOTE STYLE TOO, since 2026-09-29 (ledger B24): a rendering apart
 from another only in whether its prose apostrophes and quotation marks are
 straight or curly is one wording, which the typography restoration makes one
 after the ballot. Of 2,905 stored slates, 305 carried such a twin, 246 of the
 pairs with the incumbent, and in 30 the chosen candidate was the incumbent
 with its quotes straightened. Quotes in code and markup stay as written
 (`straightenProseQuotes`), since there they are content.

 The whole key on a line-structured slice; {@link wordingKey} adds the soft
 break fold on every other.

 @param text - candidate text

 @returns Comparison key

 @example
 ```ts
 const key = collapseKey({ text: 'The cat naps.\n\n', },);
 // => 'The cat naps.'
 ```
 */
function collapseKey({ text, }: { readonly text: string; },): string {
  /**
   Each line, blank and blank-quote lines without their trailing spaces.
   */
  const lines = straightenProseQuotes({ text, },)
    .split('\n',)
    .map(function foldedBlank(line,): string {
      return isBlankQuoteLine({ line, },) ? line.trimEnd() : line;
    },);
  return lines
    .join('\n',)
    .trimEnd();
}

/**
 Wording as the page lays it out: wrapped, then each top-level paragraph on
 one line.

 A FENCED BLOCK WHOSE YAML DOES NOT PARSE is laid out as written. The parser
 that locates paragraphs refuses the whole text, and such a text publishes no
 page at all (`validateFrontMatterTranslation` refuses it), so it is one
 wording only with a text differing from it in what {@link collapseKey}
 folds, which is the comparison every site made before ledger B26. It is
 reachable: the repair turn's copy check reads a model's reply before any
 floor has.

 @param text - wording to lay out

 @returns Layout the key compares

 @throws Whatever the wrap or the parser throws other than
 `FrontMatterParseError`

 @example
 ```ts
 pageLayout({ text: 'The cat naps on the mat\nall afternoon.', },);
 // => 'The cat naps on the mat all afternoon.'
 ```
 */
function pageLayout({ text, }: { readonly text: string; },): string {
  try {
    return foldSoftBreaks({
      text: wrapReplacementText({ text, },),
    },);
  }
  catch (error) {
    // The YAML refusal alone lays the text out as written; anything else
    // keeps propagating through the shared narrowing, whose own case
    // reaches the rethrow no caller here can.
    requireFrontMatterRefusal({ error, },);
    return text;
  }
}

/**
 Key two wordings share when they publish the same page: what
 {@link collapseKey} folds, and on a slice the line-structure rule does not
 govern, where a paragraph's soft line breaks fall.

 THE WRAP INSIDE IS COMPARED AND NEVER SHIPPED. `wrapReplacementText` is
 applied here to text a lane may have kept, which its own contract forbids for
 text that ships; nothing this returns ships. It is here for blockquotes and
 lists, whose soft breaks `foldSoftBreaks` leaves as written: a quotation the
 rule rewrapped is the wording of the one-line quotation it came from, and
 since the wrap only adds breaks and a second application moves nothing,
 wrapping both sides keeps that pair equal. Over the stored artifacts the fold
 alone missed 6 such lane texts, all blockquotes.

 WHAT STAYS APART: a hard break (two trailing spaces or a backslash), a blank
 line, indentation that opens a line, front matter, and every character that
 is not whitespace, since each is a different page.

 @param text - wording to key

 @param lineStructured - whether the line-structure rule governs the slice,
 which makes its line breaks the producer's work

 @returns Comparison key, never shipped

 @example
 ```ts
 wordingKey({ text: 'The cat naps on the mat\nall afternoon.', lineStructured: false, },)
   === wordingKey({ text: 'The cat naps on the mat all afternoon.', lineStructured: false, },);
 // => true
 ```
 */
export function wordingKey(
  {
    text,
    lineStructured,
  }: {
    readonly text: string;
    readonly lineStructured: boolean;
  },
): string {
  if (lineStructured)
    return collapseKey({ text, },);
  return collapseKey({ text: pageLayout({ text, },), },);
}

/**
 Whether a proposal is the wording that stands, as the page would show it.

 @param proposal - wording a lane, a polish or a review proposed

 @param standing - wording already there

 @param lineStructured - whether the line-structure rule governs the slice

 @returns Whether shipping the proposal would publish the page the standing
 wording already publishes

 @example
 ```ts
 const unchanged = sameWording({ proposal: 'The cat naps.\n', standing: 'The cat naps.', lineStructured: true, },);
 // => true
 ```
 */
export function sameWording(
  {
    proposal,
    standing,
    lineStructured,
  }: {
    readonly proposal: string;
    readonly standing: string;
    readonly lineStructured: boolean;
  },
): boolean {
  return wordingKey({
    text: proposal,
    lineStructured,
  },)
    === wordingKey({
      text: standing,
      lineStructured,
    },);
}

//endregion Wording key
