//region Closed mark spans
// LEDGER B38 (2026-09-30). Five readers of paired marks (the Han and Latin
// title floors, the page-title spans, the work-title scan and the
// title-reference locator) each paired an opening mark with the next closing
// mark, so an opening mark that never closed took the next title into its
// span: the Latin title floor told a model to write “Purr, and 《Cats Are
// Liquid”, the work-title scan looked that span up and never the title inside
// it, and the locator rewrote the words before the title with it. They read
// spans here now, one rule for all: a closing mark answers the last opening
// mark before it, so an opening mark followed by another before any closing
// one encloses nothing.

/**
 Where a pair of marks encloses a span: the offsets of its two marks.
 */
export type ClosedMarkSpan = {
  /**
   Offset of the opening mark.
   */
  readonly open: number;

  /**
   Offset of the closing mark.
   */
  readonly close: number;
};

/**
 Every span a pair of marks encloses, in order. A closing mark answers the
 last opening mark before it that no closing mark answered yet, so an opening
 mark with another after it before any closing one never closed, a closing
 mark with no opening mark before it is text, and an opening mark with no
 closing mark after it ends the reading.

 ONE PASS: each stretch of the text is searched forward once for its closing
 mark and backward once for the opening mark that closing mark answers, so
 the reading stays linear in the text however many marks stand unclosed.

 @param text - text to read

 @param open - opening mark

 @param close - closing mark, which may be the opening mark itself
 (straight quotation marks), so the marks pair in turn

 @returns Spans in order, each by its marks' offsets

 @example
 ```ts
 closedMarkSpans({ text: '《猫，《猫经》', open: '《', close: '》', },); // [{ open: 3, close: 6, }]
 ```
 */
export function closedMarkSpans(
  {
    text,
    open,
    close,
  }: {
    readonly text: string;
    readonly open: string;
    readonly close: string;
  },
): readonly ClosedMarkSpan[] {
  /**
   Spans read so far.
   */
  const spans: ClosedMarkSpan[] = [];
  for (let at = text.indexOf(open,); at !== (-1);) {
    /**
     First closing mark after this opening one, -1 for none.
     */
    const closeAt = text.indexOf(
      close,
      at + open.length,
    );
    if (closeAt === (-1))
      break;
    spans.push({
      // The last opening mark that ends before the closing one, which is this
      // opening mark itself where no other stands between.
      open: text.lastIndexOf(
        open,
        closeAt - open.length,
      ),
      close: closeAt,
    },);
    at = text.indexOf(
      open,
      closeAt + close.length,
    );
  }
  return spans;
}

//endregion Closed mark spans
