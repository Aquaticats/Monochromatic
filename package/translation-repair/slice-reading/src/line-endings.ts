//region Line endings
// Folding Windows line endings to the one form every splitter here looks for.
//
// MEASURED, NOT DEFENSIVE. Of the 184 markdown files in the pinned corpus, one
// uses CRLF throughout: a source page. A splitter looking for `\n\n` never
// finds a boundary in it, so the line-structure predicate counted the whole
// page as one block and answered false, the invisible-line mask saw a `\r` on
// every line and blanked none, and the quote normalizer read each ending as
// two breaks and refused every wrapped quote. One fold at the corpus read
// closes all three, and the two splitters fold again for callers that read
// text by other means.

/**
 Windows line ending.
 */
const CRLF = '\r\n';

/**
 Line ending every splitter in this package looks for.
 */
const LF = '\n';

/**
 Folds every CRLF to LF, counting how many there were.

 A lone carriage return is left alone: it is not a line ending this corpus
 writes, and folding it would be a guess about text nobody has measured.

 @param text - text as read

 @returns Folded text and the number of endings folded, zero for text that
 was already LF

 @example
 ```ts
 const { text: folded, folded: count, } = foldCarriageReturns({ text: page, },);
 ```
 */
export function foldCarriageReturns({ text, }: { readonly text: string; },): {
  /**
   Text with every CRLF folded to LF.
   */
  readonly text: string;

  /**
   How many endings were folded.
   */
  readonly folded: number;
} {
  /**
   Text with the endings folded, one linear pass by the string API.
   */
  const folded = text.replaceAll(
    CRLF,
    LF,
  );
  return {
    text: folded,
    folded: text.length - folded.length,
  };
}

//endregion Line endings
