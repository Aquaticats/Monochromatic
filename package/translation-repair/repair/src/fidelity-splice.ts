//region Fidelity splice
// Removing a sentence WITHOUT LEAVING A MARK, which the fidelity fixtures need
// and a plain deletion does not give.
//
// WHY THIS EXISTS. A sentence is stored trimmed, and prose separates sentences
// with whitespace on both sides, so cutting the sentence alone leaves both
// separators behind. Measured on the fixture text: a mid-paragraph deletion left
// a DOUBLE SPACE at the join, and deleting a trailing paragraph left THREE
// CONSECUTIVE NEWLINES. Either is a typographic edit-mark a judge can see
// without reading a word of the original, which would let the damaged candidate
// lose on tidiness rather than on coverage, and coverage is the only ground the
// trial means to test.
//
// WHY NOT FIX `applySeededErrors`. The recall benchmark and the introduced-defect
// probe seed the same way and are measured against what those seeds do today;
// changing the shared primitive would move their numbers for a reason that
// belongs to this trial alone.
//
// THE JOIN RULE, which is what makes the result read as text nobody edited:
// remove the sentence and ONE of the two whitespace runs around it, keeping
// whichever run the surrounding structure needs. At the end of the text the
// trailing run stays, at the start the leading one does, and in the middle the
// STRONGER run stays, where stronger means more line breaks. A paragraph deleted
// from the middle therefore leaves one paragraph break rather than two, and a
// sentence deleted from a paragraph leaves one space.

/**
 Characters that separate blocks and sentences in either language, written as
 escapes so no literal break can hide in the source.
 */
const WHITESPACE = [
  ' ',
  '\t',
  '\r',
  '\n',
  '\u00A0',
  '\u3000',
] as const;

/**
 Whether one character separates rather than says anything.

 @param character - single character to test

 @returns Whether it is one of {@link WHITESPACE}

 @example
 ```ts
 const separates = isSeparator({ character: '\n', },);
 ```
 */
function isSeparator({ character, }: { readonly character: string; },): boolean {
  return WHITESPACE.some(function matches(candidate,) {
    return candidate === character;
  },);
}

/**
 Counts whitespace immediately before a position.

 @param text - passage being cut

 @param at - position the run ends at, exclusive

 @returns How many characters that run holds

 @example
 ```ts
 const before = whitespaceBefore({ text, at: start, },);
 ```
 */
function whitespaceBefore(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): number {
  for (
    let cursor = at;
    cursor > 0;
    cursor -= 1
  ) {
    /**
     Character just before the cursor.
     */
    const character = text.charAt(cursor - 1,);
    if (!isSeparator({ character, },))
      return at - cursor;
  }
  // Every character before this position separates, which happens when the
  // sentence opens the passage after a blank line.
  return at;
}

/**
 Counts whitespace immediately after a position.

 @param text - passage being cut

 @param at - position the run starts at

 @returns How many characters that run holds

 @example
 ```ts
 const after = whitespaceAfter({ text, at: end, },);
 ```
 */
function whitespaceAfter(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): number {
  for (
    let cursor = at;
    cursor < text.length;
    cursor += 1
  ) {
    /**
     Character sitting at the cursor.
     */
    const character = text.charAt(cursor,);
    if (!isSeparator({ character, },))
      return cursor - at;
  }
  // Every character after this position separates, which happens when the
  // sentence closes the passage.
  return text.length - at;
}

/**
 Counts line breaks in a whitespace run, which is what decides whether it
 carries a paragraph.

 @param run - whitespace run to weigh

 @returns How many line feeds it holds

 @example
 ```ts
 const breaks = lineBreaksIn({ run: '\n\n', },);
 ```
 */
function lineBreaksIn({ run, }: { readonly run: string; },): number {
  /**
   Pieces the run falls into when cut at every line feed, which is always one
   more than the number of feeds.
   */
  const pieces = run.split('\n',);
  return pieces.length - 1;
}

/**
 Picks which of the two runs survives the cut.

 @param before - whitespace preceding the removed sentence

 @param after - whitespace following it

 @param atStart - whether the preceding run reaches the start of the text

 @param atEnd - whether the following run reaches the end of the text

 @returns Run to write in place of both

 @example
 ```ts
 const join = survivingRun({ before: ' ', after: ' ', atStart: false, atEnd: false, },);
 ```
 */
function survivingRun(
  {
    before,
    after,
    atStart,
    atEnd,
  }: {
    readonly before: string;
    readonly after: string;
    readonly atStart: boolean;
    readonly atEnd: boolean;
  },
): string {
  // A BOUNDARY DECIDES BY ITSELF. Whatever ends or begins the document has to go
  // on ending or beginning it: keeping the inner run instead would leave the
  // text starting with a blank line or ending without its newline, which is a
  // mark of the same kind this exists to avoid.
  if (atEnd)
    return after;
  if (atStart)
    return before;
  if (lineBreaksIn({ run: before, },) !== lineBreaksIn({ run: after, },)) {
    return (lineBreaksIn({ run: before, },) > lineBreaksIn({ run: after, },))
      ? before
      : after;
  }
  return (before.length >= after.length) ? before : after;
}

/**
 @internal

 Removes one occurrence of a sentence and the separator it no longer needs.

 @param text - passage to cut

 @param needle - exact sentence to remove, which must occur

 @returns Passage without it, joined as though it had never been written

 @example
 ```ts
 const cut = spliceOutSentence({ text: cleanText, needle, },);
 ```
 */
export function spliceOutSentence(
  {
    text,
    needle,
  }: {
    readonly text: string;
    readonly needle: string;
  },
): string {
  /**
   Where the sentence sits, or absence spelled as minus one.
   */
  const start = text.indexOf(needle,);
  if (start === (-1))
    return text;

  /**
   Position just past the sentence.
   */
  const end = start + needle.length;

  /**
   Whitespace run preceding it.
   */
  const before = text.slice(
    start - whitespaceBefore({
      text,
      at: start,
    },),
    start,
  );

  /**
   Whitespace run following it.
   */
  const after = text.slice(
    end,
    end + whitespaceAfter({
      text,
      at: end,
    },),
  );

  /**
   Where the cut begins, taking the preceding run with it.
   */
  const cutFrom = start - before.length;

  /**
   Where the cut ends, taking the following run with it.
   */
  const cutTo = end + after.length;
  return text.slice(
    0,
    cutFrom,
  )
    + survivingRun({
      before,
      after,
      atStart: cutFrom === 0,
      atEnd: cutTo === text.length,
    },)
    + text.slice(cutTo,);
}

//endregion Fidelity splice
