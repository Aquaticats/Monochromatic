import { isAsciiDigit, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Editor fidelity difference
// WHERE A DAMAGED TWIN AND ITS REFERENCE PART WAYS, found by comparing the texts from both ends, and the whole number a
// differing digit belongs to. Plain string comparison: nothing here reads a model's reply.

/**
 How many UTF-16 units two texts share from their starts.

 @param left - first text

 @param right - second text

 @returns Length of the common prefix

 @example
 ```ts
 // yields 4
 const shared = commonPrefixLength({ left: 'cat naps', right: 'cat purrs', },);
 ```
 */
function commonPrefixLength(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   Cursor walked while the texts agree.
   */
  let cursor = 0;
  while ((cursor < left.length) && (cursor < right.length)
    && (left.charAt(cursor,) === right.charAt(cursor,)))
    cursor += 1;
  return cursor;
}

/**
 How many UTF-16 units two texts share from their ends, at most `room`.

 @param left - first text

 @param right - second text

 @param room - most units to count, so a suffix never overlaps the prefix already taken

 @returns Length of the common suffix

 @example
 ```ts
 // yields 4
 const shared = commonSuffixLength({ left: 'a cat', right: 'the cat', room: 5, },);
 ```
 */
function commonSuffixLength(
  {
    left,
    right,
    room,
  }: {
    readonly left: string;
    readonly right: string;
    readonly room: number;
  },
): number {
  /**
   Cursor walked back from the ends while the texts agree.
   */
  let cursor = 0;
  while (
    (cursor < room)
      && (left.charAt(left.length - 1
        - cursor,) === right.charAt(right.length - 1
          - cursor,))
  )
    cursor += 1;
  return cursor;
}

/**
 Offsets where two texts stop agreeing from either end.

 @param cleanText - the reference

 @param damagedText - its twin

 @returns Where the common prefix ends, and where the differing middle ends in each text

 @example
 ```ts
 const { start, damagedEnd, cleanEnd, } = differingMiddle({ cleanText, damagedText, },);
 ```
 */
export function differingMiddle(
  {
    cleanText,
    damagedText,
  }: {
    readonly cleanText: string;
    readonly damagedText: string;
  },
): {
  readonly start: number;
  readonly damagedEnd: number;
  readonly cleanEnd: number;
} {
  /**
   Where the texts first differ.
   */
  const start = commonPrefixLength({
    left: cleanText,
    right: damagedText,
  },);
  /**
   Length of the common suffix, kept clear of the prefix.
   */
  const suffix = commonSuffixLength({
    left: cleanText,
    right: damagedText,
    room: Math.min(
      cleanText.length,
      damagedText.length,
    ) - start,
  },);
  return {
    start,
    damagedEnd: damagedText.length - suffix,
    cleanEnd: cleanText.length - suffix,
  };
}

/**
 Counts the digits of a text from one position in one direction.

 @param text - text read

 @param from - position of the first character to test

 @param step - one to read forward, minus one to read back

 @returns How many consecutive digits start at `from` in that direction

 @example
 ```ts
 // yields 4
 const digits = digitsFrom({ text: 'in 2023 and', from: 6, step: -1, },);
 ```
 */
function digitsFrom(
  {
    text,
    from,
    step,
  }: {
    readonly text: string;
    readonly from: number;
    readonly step: 1 | -1;
  },
): number {
  /**
   Digits counted.
   */
  let counted = 0;
  while (isAsciiDigit({ character: text.charAt(from + (step * counted),), },))
    counted += 1;
  return counted;
}

/**
 Widens a differing middle to the whole digit run it cuts, so a number is read whole and not by the one digit that
 differs.

 @param text - the damaged text, whose digits bound the run

 @param start - where the middle begins

 @param end - where the middle ends in the damaged text

 @returns The widened start, and how far the end moved

 @example
 ```ts
 const { start, grown, } = digitRunAround({ text: damagedText, start: 37, end: 38, },);
 ```
 */
export function digitRunAround(
  {
    text,
    start,
    end,
  }: {
    readonly text: string;
    readonly start: number;
    readonly end: number;
  },
): {
  readonly start: number;
  readonly grown: number;
} {
  return {
    start: start - digitsFrom({
      text,
      from: start - 1,
      step: -1,
    },),
    grown: digitsFrom({
      text,
      from: end,
      step: 1,
    },),
  };
}

//endregion Editor fidelity difference
