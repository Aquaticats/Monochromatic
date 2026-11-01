/**
 Line-level difference between two unequal strings,
 for the value difference walker in `value-difference.ts`.

 The common leading and trailing lines drop out as context markers,
 the differing middle renders one line at a time from each side,
 and counts stay in the summary so a reader sees the shape of the
 disagreement without reading both strings whole.

 @module
 */

//region Line split

/**
 Splits a string into lines the way a reader counts them,
 keeping empty lines so line numbers line up with editors.

 @param value - string to split

 @returns Lines in order

 @example
 ```ts
 textLines('a\nb',); // ['a', 'b']
 ```
 */
export function textLines(value: string,): readonly string[] {
  return value.split('\n',);
}

//endregion Line split

//region String difference lines

/**
 The lines describing how two unequal strings differ,
 prefixed like every other difference line.

 Emits a summary naming the first differing line and both line counts,
 one context line either side of the differing block when one exists,
 then one line per differing line from the side holding it.

 @param actual - string the assertion found

 @param expected - string the assertion listed

 @param prefix - text opening every line, naming where the strings sit

 @returns Difference lines; one summary at minimum for unequal strings

 @example
 ```ts
 stringDifferenceLines({ actual: 'a\nx', expected: 'a\ny', prefix: '', });
 // ['string differs at line 2 (line counts: actual 2, expected 2)', ...]
 ```
 */
export function stringDifferenceLines({
  actual,
  expected,
  prefix,
}: {
  readonly actual: string;
  readonly expected: string;
  readonly prefix: string;
},): readonly string[] {
  /**
   Lines of the actual string.
   */
  const actualLines = textLines(actual,);
  /**
   Lines of the expected string.
   */
  const expectedLines = textLines(expected,);
  /**
   Count of matching leading lines,
   which drop out of the rendered difference as context.
   */
  const head = commonHeadCount({
    actualLines,
    expectedLines,
  },);
  /**
   Count of matching trailing lines,
   also dropping out of the rendered difference.
   */
  const tail = commonTailCount({
    actualLines,
    expectedLines,
    head,
  },);
  /**
   Differing lines of the actual string.
   */
  const actualMiddle = actualLines.slice(
    head,
    actualLines.length - tail,
  );
  /**
   Differing lines of the expected string.
   */
  const expectedMiddle = expectedLines.slice(
    head,
    expectedLines.length - tail,
  );
  /**
   Summary naming the first differing line and both line counts,
   so the shape of the disagreement reads without both strings.
   */
  const summary = `${prefix}string differs at line ${head + 1} (line counts: actual ${actualLines.length}, expected ${expectedLines.length})`;
  /**
   Rendered difference block: summary, context, and differing lines.
   */
  const lines: string[] = [summary];
  if (head > 0) {
    lines.push(`${prefix}context line ${head}: ${actualLines[head - 1]}`,);
  }
  /**
   Longer middle length,
   so a side with extra lines still renders each of them.
   */
  const middleLength = Math.max(
    actualMiddle.length,
    expectedMiddle.length,
  );
  for (let offset = 0; offset < middleLength; offset += 1) {
    if (offset < actualMiddle.length) {
      lines.push(`${prefix}line ${head + offset
        + 1}: actual ${actualMiddle[offset]}`,);
    }
    if (offset < expectedMiddle.length) {
      lines.push(`${prefix}line ${head + offset
        + 1}: expected ${expectedMiddle[offset]}`,);
    }
  }
  if (tail > 0) {
    lines.push(`${prefix}context line ${(actualLines.length - tail) + 1}: ${actualLines[actualLines.length - tail]}`,);
  }
  return lines;
}

/**
 The count of matching leading lines both strings share.

 @param actualLines - lines of the actual string

 @param expectedLines - lines of the expected string

 @returns Shared leading line count

 @example
 ```ts
 commonHeadCount({ actualLines: ['a', 'x'], expectedLines: ['a', 'y'], }); // 1
 ```
 */
function commonHeadCount({
  actualLines,
  expectedLines,
}: {
  readonly actualLines: readonly string[];
  readonly expectedLines: readonly string[];
},): number {
  /**
   Shared leading line count so far.
   */
  let count = 0;
  while ((count < actualLines.length)
    && (count < expectedLines.length)
    && (actualLines[count] === expectedLines[count])) {
    count += 1;
  }
  return count;
}

/**
 The count of matching trailing lines both strings share past the head,
 stopping at the head so the counted blocks never overlap.

 @param actualLines - lines of the actual string

 @param expectedLines - lines of the expected string

 @param head - shared leading line count already measured

 @returns Shared trailing line count

 @example
 ```ts
 commonTailCount({ actualLines: ['a', 'x', 'z'], expectedLines: ['a', 'y', 'z'], head: 1, }); // 1
 ```
 */
function commonTailCount({
  actualLines,
  expectedLines,
  head,
}: {
  readonly actualLines: readonly string[];
  readonly expectedLines: readonly string[];
  readonly head: number;
},): number {
  /**
   Shared trailing line count so far.
   */
  let count = 0;
  while ((count < (actualLines.length - head))
    && (count < (expectedLines.length - head))
    && (actualLines[actualLines.length - count
      - 1]
      === expectedLines[expectedLines.length - count
        - 1])) {
    count += 1;
  }
  return count;
}

//endregion String difference lines
