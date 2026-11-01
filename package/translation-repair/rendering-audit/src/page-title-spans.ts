import {
  carriesHan,
  closedMarkSpans,
  codePointCount,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  htmlHeadings,
  markdownHeadings,
  type PlacedText,
} from './page-headings.ts';
import { pairedPageNames, } from './page-name-glossary.ts';
import { visibleText, } from './page-visible-text.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Page title spans
// LEDGER H16 (2026-09-28). Each slice is written and judged alone, and where
// the archive renders a passage its English anchors the slice; where it does
// not, a title the page repeats reaches every slice's bench with nothing
// saying how the others rendered it. XingZ60's partial archive shipped one
// section title in three forms: its heading, its attribution and a footnote
// naming it. The page-name glossary (`page-name-glossary.ts`) carries the
// titles and names the archive pairs; these are the ones it leaves, which the
// preparation settles once per page (`corpus-run/pass-page-titles.ts`).
//
// A TITLE IS A MARKED SPAN: heading text (Markdown or HTML), 《…》, 【…】, or
// 「…」 where the same words stand in one of those elsewhere on the page, since
// 「」 alone marks quotations and dialogue far more often than titles.
//
// LISTED IN PAGE ORDER. Every span keeps the offset it stands at (the heading
// readers are the page-name glossary's too, `page-headings.ts`), and titles
// are listed by where they first stand; the first version listed every
// heading before any 《》 span wherever each stood, while saying it listed by
// first appearance.

/**
 Longest span read as a title, in code points; a longer one is a sentence.
 */
const LONGEST_TITLE = 24;

/**
 One title the page repeats and the archive leaves unpaired.

 @example
 ```ts
 const span: RepeatedTitleSpan = { source: '猫之歌', occurrences: 3, };
 ```
 */
export type RepeatedTitleSpan = {
  /**
   Title as the original writes it.
   */
  readonly source: string;

  /**
   Marked places it stands on the page.
   */
  readonly occurrences: number;
};

/**
 Link text of a span written as a Markdown link, the span itself otherwise.

 @param span - text inside the markers

 @returns Text a reader sees

 @example
 ```ts
 shownText({ span: '[猫之歌](https://example.invalid/song)', },); // '猫之歌'
 ```
 */
function shownText({ span, }: { readonly span: string; },): string {
  /**
   Span without surrounding space.
   */
  const trimmed = span.trim();
  /**
   Where the link's text closes, -1 for no link.
   */
  const close = trimmed.indexOf('](',);
  return (trimmed.startsWith('[',) && (close !== (-1))) ? trimmed.slice(
    1,
    close,
  ) : trimmed;
}

/**
 Texts between an opening and a closing marker, a marker that never closed
 enclosing nothing (ledger B38).

 @param text - page text

 @param open - opening marker

 @param close - closing marker

 @returns Each span's shown text and where it opens, in page order

 @example
 ```ts
 spansBetween({ text: '《猫》与《狗》', open: '《', close: '》', },); // 猫 at 0, 狗 at 4
 ```
 */
function spansBetween(
  {
    text,
    open,
    close,
  }: {
    readonly text: string;
    readonly open: string;
    readonly close: string;
  },
): readonly PlacedText[] {
  return closedMarkSpans({
    text,
    open,
    close,
  },)
    .map(function placed(span,): PlacedText {
      return {
        text: shownText({ span: text.slice(
          span.open + open.length,
          span.close,
        ), },),
        at: span.open,
      };
    },);
}

/**
 Titles a page repeats that the archive leaves unpaired: each title-marked span
 carrying Han that stands in two or more marked places on what the page shows,
 minus the source texts the page-name glossary pairs.

 @param sourceText - whole original document

 @param targetText - whole archive document

 @returns Repeated titles in order of where each first stands

 @example
 ```ts
 const spans = repeatedTitleSpans({ sourceText, targetText, },);
 ```
 */
export function repeatedTitleSpans(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): readonly RepeatedTitleSpan[] {
  /**
   What the original shows.
   */
  const shown = visibleText({ text: sourceText, },);
  /**
   Spans the page marks as titles in their own right.
   */
  const titled = [
    ...markdownHeadings({ text: shown, },),
    ...htmlHeadings({ text: shown, },),
    ...spansBetween({
      text: shown,
      open: '《',
      close: '》',
    },),
    ...spansBetween({
      text: shown,
      open: '【',
      close: '】',
    },),
  ];
  /**
   Those spans' texts, for the quotation rule.
   */
  const titles = new Set(titled.map(function textOf(placed,): string {
    return placed.text;
  },),);
  /**
   Quoted spans that repeat a title.
   */
  const quoted = spansBetween({
    text: shown,
    open: '「',
    close: '」',
  },)
    .filter(function repeatsTitle(placed,): boolean {
      return titles.has(placed.text,);
    },);
  /**
   Titles the page-name glossary already carries.
   */
  const paired = pairedPageNames({
    sourceText,
    targetText,
  },);
  /**
   Marked places per title, in order of where each first stands.
   */
  const counts = [
    ...titled,
    ...quoted,
  ]
    .toSorted(function byPlace(
      left,
      right,
    ): number {
      return left.at - right.at;
    },)
    .reduce(
    function tally(
      seen,
      placed,
    ) {
      seen.set(
        placed.text,
        (seen.get(placed.text,) ?? 0) + 1,
      );
      return seen;
    },
    new Map<string, number>(),
    );
  return [...counts.entries(),]
    .filter(function repeatedAndUnpaired([span, occurrences,]: ReadonlyEntry<string, number>,): boolean {
      return (occurrences >= 2)
        && carriesHan({ text: span, },)
        && (codePointCount({ text: span, },) <= LONGEST_TITLE)
        && (!paired.has(span,));
    },)
    .map(function toSpan([source, occurrences,]: ReadonlyEntry<string, number>,): RepeatedTitleSpan {
      return {
        source,
        occurrences,
      };
    },);
}

//endregion Page title spans
