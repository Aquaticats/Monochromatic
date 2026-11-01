/**
 Where a phrase stands, for the scan behind `position-references.unit.test.ts`
 (ledger D33): inside double quotes in its paragraph, inside a Markdown code
 span on its line, and where its paragraph starts. Split from
 `position-references.test-fixture.ts` so each file stays under the line
 limit; the scan calls these three and nothing else here.

 @module
 */

//region Phrase context

/**
 How often one character occurs in a text.

 @param text - text to count in

 @param character - one UTF-16 unit, which every quote mark is

 @returns Occurrences

 @example
 ```ts
 countOf({ text: 'a "b" c', character: '"', },); // 2
 ```
 */
function countOf(
  {
    text,
    character,
  }: {
    readonly text: string;
    readonly character: string;
  },
): number {
  return text.split(character,)
    .length
    - 1;
}

/**
 Whether an offset stands inside double quotes in its paragraph.

 READ FROM THE PARAGRAPH'S START, not the line's: a quotation wrapped across
 two lines opens on one and closes on the next, so counting from the line's
 start reads every phrase after it on the second line as quoted and every
 quoted phrase as bare.

 @param flat - joined text

 @param lineStart - offset its paragraph starts at

 @param offset - offset of the phrase

 @returns Whether an odd number of straight quotes, or more opening than
 closing curly quotes, stand before it in its paragraph

 @example
 ```ts
 insideQuotes({ flat: 'a "see above" rule', lineStart: 0, offset: 3, },); // true
 ```
 */
export function insideQuotes(
  {
    flat,
    lineStart,
    offset,
  }: {
    readonly flat: string;
    readonly lineStart: number;
    readonly offset: number;
  },
): boolean {
  /**
   The paragraph before the phrase.
   */
  const before = flat.slice(
    lineStart,
    offset,
  );
  return ((countOf({
    text: before,
    character: '"',
  },) % 2) === 1)
    || (countOf({
      text: before,
      character: '\u{201C}',
    },) > countOf({
      text: before,
      character: '\u{201D}',
    },));
}

/**
 Length of the run of backticks starting at an offset, zero when none does.

 @param line - the line as written

 @param at - offset to read from

 @returns Backticks in the run

 @example
 ```ts
 backtickRun({ line: 'a `` b', at: 2, },); // 2
 ```
 */
function backtickRun(
  {
    line,
    at,
  }: {
    readonly line: string;
    readonly at: number;
  },
): number {
  for (let end = at; end < line.length; end += 1) {
    if (line.charAt(end,) !== '`')
      return end - at;
  }
  return line.length - at;
}

/**
 Where the backtick run closing a code span starts: the next run of exactly
 the opening run's length, as CommonMark reads a span.

 @param line - the line as written

 @param from - offset just past the opening run

 @param run - length of the opening run

 @returns Offset of the closing run, -1 when the line holds none

 @example
 ```ts
 closingRun({ line: '`` a` ``', from: 2, run: 2, },); // 6
 ```
 */
function closingRun(
  {
    line,
    from,
    run,
  }: {
    readonly line: string;
    readonly from: number;
    readonly run: number;
  },
): number {
  for (let at = from; at < line.length;) {
    /**
     Backticks in the run here, zero on any other character.
     */
    const length = backtickRun({
      line,
      at,
    },);
    if (length === run)
      return at;
    at += Math.max(
      length,
      1,
    );
  }
  return -1;
}

/**
 Whether an offset stands inside a Markdown code span on its line, which
 quotes data (a label a page carries, a log line) rather than pointing.

 READ FROM THE LINE AS WRITTEN, not as joined: joining strips a backtick that
 opens a line, since in a source file that backtick opens a wrapped template,
 and a Markdown line opening with a code span would then count one short.

 A SPAN CLOSES ON A RUN OF ITS OWN LENGTH, as CommonMark reads it, so a span
 opened by two backticks can quote a single one, and a run nothing closes is
 plain text. Counting backticks for parity read such a span as outside.

 @param line - the line as written

 @param offset - offset of the phrase in it

 @returns Whether a code span on the line holds it

 @example
 ```ts
 insideCodeSpan({ line: 'a `the above` b', offset: 3, },); // true
 ```
 */
export function insideCodeSpan(
  {
    line,
    offset,
  }: {
    readonly line: string;
    readonly offset: number;
  },
): boolean {
  for (let at = 0; at < offset;) {
    /**
     Backticks opening a span here, zero on any other character.
     */
    const run = backtickRun({
      line,
      at,
    },);
    if (run === 0) {
      at += 1;
      continue;
    }
    /**
     Where the span this run opens closes, -1 when nothing does.
     */
    const close = closingRun({
      line,
      from: at + run,
      run,
    },);
    if (close < 0)
      at += run;
    else if (offset < close)
      return true;
    else
      at = close + run;
  }
  return false;
}

/**
 Offset of the paragraph a line belongs to: the line after the last blank one
 at or before it.

 @param flat - joined text

 @param lineStarts - where each line starts

 @param lineIndex - index of the line

 @returns Offset the paragraph starts at

 @example
 ```ts
 paragraphStart({ flat: 'a  b', lineStarts: [0, 1, 2,], lineIndex: 2, },); // 2
 ```
 */
export function paragraphStart(
  {
    flat,
    lineStarts,
    lineIndex,
  }: {
    readonly flat: string;
    readonly lineStarts: readonly number[];
    readonly lineIndex: number;
  },
): number {
  for (let at = lineIndex; at > 0; at -= 1) {
    /**
     The line before this one, as joined.
     */
    const previous = flat.slice(
      lineStarts[at - 1] ?? 0,
      lineStarts[at] ?? 0,
    );
    if (previous.trim() === '')
      return lineStarts[at] ?? 0;
  }
  return 0;
}

//endregion Phrase context
