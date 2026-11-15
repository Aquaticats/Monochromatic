/**
 Writes and reads the Markdown code span a range is written in, so that a range comes back byte for byte.

 A range is a pair of Git references, and a reference may hold a comma, a semicolon, an underscore or a backtick
 (`git check-ref-format` allows every printable byte except space, `~`, `^`, `:`, `?`, `*`, `[` and `\`). A review's own file is
 Markdown, so a range crosses from the grammar of Git references into the grammar of Markdown and back: the brief writes every
 range in a code span (CommonMark, section "Code spans"), and this module reads one out of a line with nothing removed from
 inside it.

 A fence one backtick longer than the longest run of backticks in the range keeps a range that holds a backtick unambiguous, and a
 space on each side of the content keeps a range that begins or ends with a backtick from joining the fence.

 @module
 */

/**
 The number of backticks a fence carries beyond the longest run of backticks in the text it holds.
 */
const A_FENCE_LONGER_THAN_ITS_CONTENT = 1;

/**
 The character a code span is fenced with.
 */
const BACKTICK = '`';

/**
 The space a fence is padded with, which CommonMark removes again when the content begins and ends with one.
 */
const A_SPACE = ' ';

/**
 Reads how many backticks a run holds, from where it starts.

 @param text - the text the run is in

 @param from - where the run starts, which is a backtick

 @returns the number of backticks in the run

 @example
 ```ts
 const length = runLengthIn({ text: '``a', from: 0, },);
 ```
 */
function runLengthIn({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
}): number {
  for (let index = from; index < text.length; index += 1) {
    if (text.charAt(index,) !== BACKTICK) {
      return index - from;
    }
  }
  return text.length - from;
}

/**
 Reads where the next run of backticks of a given length starts, which is what closes a code span.

 A longer or shorter run inside the span is literal text, as CommonMark says, so the scan steps over it and looks on.

 @param text - the text the span continues in

 @param from - where the search starts

 @param length - how many backticks the closing run holds

 @returns where that run starts, or minus one when the text holds none

 @example
 ```ts
 const close = closingRunIn({ text: 'a` b`', from: 0, length: 1, },);
 ```
 */
function closingRunIn({
  text,
  from,
  length,
}: {
  readonly text: string;
  readonly from: number;
  readonly length: number;
}): number {
  for (let index = text.indexOf(
    BACKTICK,
    from,
  ); index !== (-1);) {
    /**
     How many backticks this run holds.
     */
    const runLength = runLengthIn({
      text,
      from: index,
    },);
    if (runLength === length) {
      return index;
    }
    index = text.indexOf(
      BACKTICK,
      index + runLength,
    );
  }
  return -1;
}

/**
 Reads the content of a code span, as CommonMark gives it: one space is removed from each end when the content begins and ends
 with one and is not all spaces.

 The test is the space character alone, never `String.trim()`: a range may begin or end with a byte `trim()` calls space, such as
 U+00A0, and that byte is part of the range.

 @param text - the text between the two fences

 @returns the content the span holds

 @example
 ```ts
 const content = contentOf(' main...fix/a ');
 ```
 */
function contentOf(text: string,): string {
  if ((!text.startsWith(A_SPACE,)) || (!text.endsWith(A_SPACE,))) {
    return text;
  }
  /**
   The content without one space at each end.
   */
  const inner = text.slice(
    1,
    -1,
  );
  return isAllSpaces({ text: inner, },) ? text : inner;
}

/**
 Says whether a text is nothing but the space character.

 @param text - the text to read

 @returns whether every character of it is a space, and false for a text that holds none

 @example
 ```ts
 const allSpaces = isAllSpaces({ text: '   ', },);
 ```
 */
function isAllSpaces({ text, }: { readonly text: string },): boolean {
  for (let index = 0; index < text.length; index += 1) {
    if (text.charAt(index,) !== A_SPACE) {
      return false;
    }
  }
  return text.length > 0;
}

/**
 Says how long the longest run of backticks in a text is.

 @param text - the text to read

 @returns the number of backticks in its longest run, and zero when it holds none

 @example
 ```ts
 const longest = longestRunIn({ text: 'a``b', },);
 ```
 */
function longestRunIn({ text, }: { readonly text: string },): number {
  /**
   The longest run found so far.
   */
  let longest = 0;
  for (let index = text.indexOf(BACKTICK,); index !== (-1);) {
    /**
     How many backticks the run at this place holds.
     */
    const runLength = runLengthIn({
      text,
      from: index,
    },);
    longest = Math.max(
      longest,
      runLength,
    );
    index = text.indexOf(
      BACKTICK,
      index + runLength,
    );
  }
  return longest;
}

/**
 Writes a text as a code span that reads back as the same bytes.

 @param text - the text the span holds

 @returns the span, fenced one backtick longer than any run inside it

 @example
 ```ts
 const span = codeSpanOf({ text: 'main...fix/a', },);
 ```
 */
export function codeSpanOf({ text, }: { readonly text: string },): string {
  /**
   The fence, which is longer than any run of backticks the text holds.
   */
  const fence = BACKTICK.repeat(longestRunIn({ text, },) + A_FENCE_LONGER_THAN_ITS_CONTENT,);
  /**
   The text, padded when it begins or ends with a backtick, so that no backtick of its own joins the fence.
   */
  const padded = (text.startsWith(BACKTICK,) || text.endsWith(BACKTICK,))
    ? `${A_SPACE}${text}${A_SPACE}`
    : text;
  return `${fence}${padded}${fence}`;
}

/**
 Reads every code span a text holds, in the order it holds them.

 @param text - the text to read, such as one line of a review's own file

 @returns the content of each span, and nothing when the text holds none

 @example
 ```ts
 const spans = codeSpansIn({ text: 'Ranges reviewed: `main...fix/a`, `main...fix/b`', },);
 ```
 */
export function codeSpansIn({ text, }: { readonly text: string },): readonly string[] {
  /**
   The spans read so far, in the order the text holds them.
   */
  const spans: string[] = [];
  for (let index = text.indexOf(BACKTICK,); index !== (-1);) {
    /**
     How many backticks the opening run holds.
     */
    const runLength = runLengthIn({
      text,
      from: index,
    },);
    /**
     Where a run of the same length closes the span.
     */
    const close = closingRunIn({
      text,
      from: index + runLength,
      length: runLength,
    },);
    // A backtick nothing closes is literal text, as CommonMark says, so the scan looks past it for a span that does close.
    if (close === (-1)) {
      index = text.indexOf(
        BACKTICK,
        index + runLength,
      );
      continue;
    }
    spans.push(
      contentOf(text.slice(
        index + runLength,
        close,
      ),),
    );
    index = text.indexOf(
      BACKTICK,
      close + runLength,
    );
  }
  return spans;
}
