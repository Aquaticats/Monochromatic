import { SENTENCE_TERMINATORS, } from './derive-seeds.ts';

//region Sentence region
// THE WHOLE SENTENCES A DEFECT SITS IN, which is the region a critic quotes when it flags a clause and the one an
// editor may rewrite. A sentence runs from the character after the previous terminator or line break to its own
// terminator, or to the line break or the end of the text where it has none. The separators on either side stay out,
// so replacing the region leaves the spacing the text had.

/**
 Line break, which closes a region as a terminator does: a heading or a list line has no terminator and is its own
 region.
 */
const LINE_BREAK = '\n';

/**
 Offsets of a region in one text.

 @example
 ```ts
 const region: SentenceRegion = { startOffset: 14, endOffset: 33, };
 ```
 */
export type SentenceRegion = {
  /**
   First character of the region.
   */
  readonly startOffset: number;

  /**
   End of the region, exclusive.
   */
  readonly endOffset: number;
};

/**
 Whether one character separates rather than says anything.

 @param character - single character to test

 @returns Whether it is whitespace

 @example
 ```ts
 const blank = isBlank({ character: ' ', },);
 ```
 */
function isBlank({ character, }: { readonly character: string; },): boolean {
  return character.trim() === '';
}

/**
 Whether one character closes the sentence it ends.

 @param character - single character to test

 @returns Whether it is a terminator of either language

 @example
 ```ts
 const closes = isTerminator({ character: '。', },);
 ```
 */
function isTerminator({ character, }: { readonly character: string; },): boolean {
  return SENTENCE_TERMINATORS.includes(character,);
}

/**
 Whether one character closes a sentence's head: a terminator or a line break.

 @param character - single character to test

 @returns Whether a sentence cannot run back past it

 @example
 ```ts
 const closes = closesHead({ character: '\n', },);
 ```
 */
function closesHead({ character, }: { readonly character: string; },): boolean {
  return isTerminator({ character, },) || (character === LINE_BREAK);
}

/**
 First character in a span that says something, past its leading blanks.

 @param text - passage the span is in

 @param startOffset - first character of the span

 @param endOffset - end of the span, exclusive

 @returns Offset of the first character that is not blank, or `endOffset` when the span is blank throughout

 @example
 ```ts
 const first = firstSaying({ text, startOffset: 3, endOffset: 9, },);
 ```
 */
function firstSaying(
  {
    text,
    startOffset,
    endOffset,
  }: {
    readonly text: string;
    readonly startOffset: number;
    readonly endOffset: number;
  },
): number {
  /**
   Cursor walked forward over blanks.
   */
  let cursor = startOffset;
  while ((cursor < endOffset) && isBlank({ character: text.charAt(cursor,), },))
    cursor += 1;
  return cursor;
}

/**
 End of a span once its trailing blanks are out, never before `floor`.

 @param text - passage the span is in

 @param floor - offset the end may not pass back over

 @param endOffset - end of the span, exclusive

 @returns End of the last character that is not blank

 @example
 ```ts
 const end = lastSaying({ text, floor: 3, endOffset: 9, },);
 ```
 */
function lastSaying(
  {
    text,
    floor,
    endOffset,
  }: {
    readonly text: string;
    readonly floor: number;
    readonly endOffset: number;
  },
): number {
  /**
   Cursor walked back over blanks.
   */
  let cursor = endOffset;
  while ((cursor > floor) && isBlank({ character: text.charAt(cursor - 1,), },))
    cursor -= 1;
  return cursor;
}

/**
 Head of the sentence a position is in, walked back to the character after a terminator or a line break and then
 forward over the blanks.

 @param text - passage the position is in

 @param start - first character that says something in the span

 @returns Offset of the sentence's first character

 @example
 ```ts
 const head = headOf({ text, start: 20, },);
 ```
 */
function headOf(
  {
    text,
    start,
  }: {
    readonly text: string;
    readonly start: number;
  },
): number {
  /**
   Cursor walked back to the head, then forward over its blanks.
   */
  let cursor = start;
  while ((cursor > 0) && (!closesHead({ character: text.charAt(cursor - 1,), },)))
    cursor -= 1;
  while ((cursor < start) && isBlank({ character: text.charAt(cursor,), },))
    cursor += 1;
  return cursor;
}

/**
 Walks forward from the end of a span to the next terminator, line break or the end of the text.

 @param text - passage the span is in

 @param end - end of the span once its trailing blanks are out, exclusive

 @returns Offset just past the terminator, or of the line break or the end of the text

 @example
 ```ts
 const reached = walkToTerminator({ text, end: 33, },);
 ```
 */
function walkToTerminator(
  {
    text,
    end,
  }: {
    readonly text: string;
    readonly end: number;
  },
): number {
  /**
   Cursor walked forward.
   */
  let cursor = end;
  while ((cursor < text.length) && (text.charAt(cursor,) !== LINE_BREAK)) {
    cursor += 1;
    if (isTerminator({ character: text.charAt(cursor - 1,), },))
      break;
  }
  return cursor;
}

/**
 End of the sentence a span finishes in, walked forward to its terminator unless the span already ends on one.

 @param text - passage the span is in

 @param end - end of the span once its trailing blanks are out, exclusive

 @returns End of the sentence, exclusive and without trailing blanks

 @example
 ```ts
 const tail = tailOf({ text, end: 33, },);
 ```
 */
function tailOf(
  {
    text,
    end,
  }: {
    readonly text: string;
    readonly end: number;
  },
): number {
  if (isTerminator({ character: text.charAt(end - 1,), },))
    return end;
  return lastSaying({
    text,
    floor: end,
    endOffset: walkToTerminator({
      text,
      end,
    },),
  },);
}

/**
 Widens a span to the whole sentences it touches.

 @param text - passage the span is in

 @param startOffset - first character of the span

 @param endOffset - end of the span, exclusive

 @returns The region, with the blanks around the span's sentences left out; a zero-width span, or one that is blank
 throughout, comes back unchanged

 @example
 ```ts
 const region = sentenceRegionAround({ text, startOffset: 20, endOffset: 24, },);
 ```
 */
export function sentenceRegionAround(
  {
    text,
    startOffset,
    endOffset,
  }: {
    readonly text: string;
    readonly startOffset: number;
    readonly endOffset: number;
  },
): SentenceRegion {
  /**
   First character of the span that says something.
   */
  const start = firstSaying({
    text,
    startOffset,
    endOffset,
  },);
  /**
   End of the span once its trailing blanks are out.
   */
  const end = lastSaying({
    text,
    floor: start,
    endOffset,
  },);
  if (start === end) {
    return {
      startOffset,
      endOffset,
    };
  }
  return {
    startOffset: headOf({
      text,
      start,
    },),
    endOffset: tailOf({
      text,
      end,
    },),
  };
}

//endregion Sentence region
