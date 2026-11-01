import {
  type ConsistencyRendering,
  planConsistencyEdit,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type { ConsistencyKeptReason, } from '@monochromatic-dev/translation-repair-artifact/ts';

//region Pass consistency edge refusal
// WHY THE PASS LEFT A PLACE AS IT WAS WHEN THE RECORD'S WORDS LET CODE MOVE THE CHOSEN WORDS IN. The reader of a
// section holds no English, so it recomputes each receiving place's plan from the recorded words and the single marks
// directly beside them (`consistency-splice.ts`). The pass holds the English, and refuses a splice on two readings of it
// that the record cannot make: a single mark past a run of markup beyond an edge the words bring an apostrophe to (the
// splice would write two apostrophes for the eye), and a single mark outside the words that the splice would make read
// differently, and, on a page that holds a character reference that is a quotation mark (`&#39;`, `&rsquo;`), any splice at
// all, whether or not it brings an apostrophe, wherever the reference stands on the page, in any slice of it (the guard is
// `consistency-splice.ts`, the `pageReference` test of `planConsistencyEdit`; a place whose own edge is the reference and a
// place far from it are kept alike). A place so refused is recorded `kept-current` with the plan
// the reader recomputes being `replace`, and the reader accepts that only with the reason this file names.

/**
 The reasons a splice is refused at the edge of the span, which the record's words cannot show: `edge-second-mark` (a single
 mark past markup beyond the edge), `edge-reading-changes` (a mark outside the words that would read differently) and
 `edge-reference-mark` (a character reference of the page that is a quotation mark, which no reader of the raw English sees).
 */
export type EdgeRefusal = Extract<
  ConsistencyKeptReason,
  'edge-reading-changes' | 'edge-reference-mark' | 'edge-second-mark'
  >;

/**
 What the pass found at the edge of a span: one of the reasons, or that it refused nothing there.
 */
export type EdgeVerdict = EdgeRefusal | 'unrefused';

/**
 A located rendering.
 */
type Located = Extract<ConsistencyRendering, { readonly kind: 'located'; }>;

/**
 A located rendering as the record holds it: without the single marks past markup, which the record does not keep.

 @param rendering - the located rendering the pass counted

 @returns The rendering with neither of the two fields

 @example
 ```ts
 const recorded = recordedLike({ rendering, },);
 ```
 */
function recordedLike({ rendering, }: { readonly rendering: Located; },): Located {
  /**
   The rendering with the two fields, to take them out of.
   */
  const {
    markPastBefore: _before,
    markPastAfter: _after,
    ...kept
  } = rendering;
  return kept;
}

/**
 Why the planner called a splice unavailable when the reader of the record, holding the recorded words and marks alone,
 would plan it as a replacement.

 @param chosen - the chosen place's located rendering

 @param target - the receiving place's located rendering

 @param framed - whether the group is a frame

 @param english - the English the receiving words stand in, which the pass holds and the record does not

 @param pageReference - the character references of the page that decode to a quotation-like character, asked as the planner
 asks it, when the pass holds the page; absent where it holds none

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), which the planner reads the English with
 where the pass holds the page, so the refusal is named for the reading the planner made; absent where it holds none (n173)

 @returns The refusal at the edge, `unrefused` when the planner did not call the splice unavailable with that English, or
 when it is unavailable on the recorded words and marks alone, which the reader of the record sees too; where more than one
 reason holds, the second mark first, then the reading that changes, then the reference

 @example
 ```ts
 const refusal = edgeRefusalOf({ chosen, target, framed: false, english, },);
 ```
 */
export function edgeRefusalOf(
  {
    chosen,
    target,
    framed,
    english,
    pageReference,
    definitions,
  }: {
    readonly chosen: Located;
    readonly target: Located;
    readonly framed: boolean;
    readonly english: string;
    readonly pageReference?: () => readonly string[];
    readonly definitions?: string;
  },
): EdgeVerdict {
  /**
   What the planner makes of the splice in the pass: the single marks past markup, the English and the page all known.
   */
  const inPass = planConsistencyEdit({
    chosen,
    target,
    framed,
    marks: 'compared',
    english,
    ...((definitions === undefined) ? {} : { definitions, }),
    ...((pageReference === undefined) ? {} : { pageReference, }),
  },);
  if (inPass.kind !== 'unavailable')
    return 'unrefused';
  /**
   What the reader of the record makes of it: neither the marks past markup nor the English.
   */
  const inRecord = planConsistencyEdit({
    chosen: recordedLike({ rendering: chosen, },),
    target: recordedLike({ rendering: target, },),
    framed,
    marks: 'compared',
  },);
  if (inRecord.kind !== 'replace')
    return 'unrefused';
  /**
   What the planner makes of it with the marks past markup and without the English.
   */
  const withMarks = planConsistencyEdit({
    chosen,
    target,
    framed,
    marks: 'compared',
  },);
  if (withMarks.kind === 'unavailable')
    return 'edge-second-mark';
  /**
   What the planner makes of it with the English and without the page.
   */
  const withEnglish = planConsistencyEdit({
    chosen,
    target,
    framed,
    marks: 'compared',
    english,
    ...((definitions === undefined) ? {} : { definitions, }),
  },);
  return (withEnglish.kind === 'unavailable') ? 'edge-reading-changes' : 'edge-reference-mark';
}

//endregion Pass consistency edge refusal
