import { closedMarkSpans, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Work-title scan
// The works an original names, read off its 《…》 marks, and the query each
// becomes. Split from `work-title-lookup.ts` at its line budget and along a
// seam: this is pure text, the rest is the web and the disk.

/**
 Opening title mark.
 */
const TITLE_OPEN = '《';

/**
 Closing title mark.
 */
const TITLE_CLOSE = '》';

/**
 Every 《…》 span in a text, once each, in order of first appearance, marks
 included; a mark that never closed brackets nothing, so the title after it
 is the one looked up (ledger B38).

 Measured over the pinned corpus on 2026-09-02: 32 of 92 entries carry one or
 more, 118 spans, at most 13 in one entry (XingZ60).

 @param text - original document

 @returns Titles as the original writes them

 @example
 ```ts
 workTitlesOf({ text: '她读《活着》，又读《活着》。', },);
 // => ['《活着》']
 ```
 */
export function workTitlesOf(
  { text, }: { readonly text: string; },
): readonly string[] {
  return [
    ...new Set(closedMarkSpans({
      text,
      open: TITLE_OPEN,
      close: TITLE_CLOSE,
    },)
      .map(function withMarks(span,): string {
        return text.slice(
          span.open,
          span.close + TITLE_CLOSE.length,
        );
      },),),
  ];
}

/**
 Query sent for one title, which is also the cache key.

 @param title - title with its marks

 @returns Search string asking for the official English title

 @example
 ```ts
 lookupQueryFor({ title: '《活着》', },);
 // => '《活着》 official English title'
 ```
 */
export function lookupQueryFor(
  { title, }: { readonly title: string; },
): string {
  return `${title} official English title`;
}

//endregion Work-title scan
