import type { ParsedSpan, } from './footnote-parsed-spans.ts';

//region Footnote span union
// SPANS OF THE PARSE, PUT IN ORDER AND JOINED. Kept apart from the reading of the parse
// (`footnote-parsed-spans.ts`) so that file stays within its length; the reading hands
// this the spans one parse gives and takes back the disjoint ones a search can bisect.

/**
 Puts spans in source order and joins those that overlap or touch, so a search
 over them is sound. A masked comment can lie inside a code span or a link's
 destination, and one text can give the same span twice.

 @internal

 @param spans - spans in any order

 @returns Disjoint spans in source order

 @example
 ```ts
 const joined = disjointSpans({ spans: [{ start: 4, end: 9, }, { start: 0, end: 5, },], },);
 ```
 */
export function disjointSpans({ spans, }: { readonly spans: readonly ParsedSpan[]; },): readonly ParsedSpan[] {
  /**
   Owned result list.
   */
  const joined: ParsedSpan[] = [];
  for (const span of spans.toSorted(function bySource(
    left,
    right,
  ): number {
    return left.start - right.start;
  },)) {
    /**
     Latest span joined so far, absent before the first.
     */
    const last = joined.at(-1,);
    if ((last !== undefined) && (span.start <= last.end))
      joined[joined.length - 1] = {
        start: last.start,
        end: Math.max(
          last.end,
          span.end,
        ),
      };
    else
      joined.push(span,);
  }
  return joined;
}

//endregion Footnote span union
