import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Title reference scope
// THE PART OF A SLICE'S PAGE TEXT A TITLE SEARCH READS. A definitions tail
// carries several footnotes, each quoting its own title, so a whole-slice
// search for one quoted span reads ambiguous; where the original references
// the title on a footnote definition line, the search is held to the page's
// line with the same label, and where it references it on a line opening
// with a tag, to the page's line opening with the same tag in the same place.
//
// THE LINE THAT REFERENCES THE TITLE IS THE ONE THAT BRACKETS IT (ledger B23).
// A footnote naming the title bare ahead of the footnote that brackets it
// held the search to the wrong line, and the reference went unplaced; and a
// Han title matched bare inside a longer one. The marks here are the ones the
// unify pass reads a reference by, so the two cannot disagree.

/**
 Newline, which no span crosses.
 */
const LINE_END = '\n';

/**
 Marks the original may bracket a title in, opening then closing.
 */
const REFERENCE_MARKS: readonly (readonly [
  string,
  string,
])[] = [
  [
    '《',
    '》',
  ],
  [
    '《[',
    '](',
  ],
  [
    '「',
    '」',
  ],
  [
    '『',
    '』',
  ],
  [
    '“',
    '”',
  ],
  [
    '"',
    '"',
  ],
];

/**
 Whether a text references a title inside any bracketing marks.

 @param text - original text or one line of it, comments cut

 @param title - Han title

 @returns True when the title stands bracketed

 @example
 ```ts
 bracketsTitle({ text: '见「猫」篇', title: '猫', },); // true
 ```
 */
export function bracketsTitle(
  {
    text,
    title,
  }: {
    readonly text: string;
    readonly title: string;
  },
): boolean {
  return REFERENCE_MARKS.some(function wraps([
    open,
    close,
  ],): boolean {
    return text.includes(`${open}${title}${close}`,);
  },);
}

/**
 Opening of a footnote definition's label.
 */
const LABEL_OPEN = '[^';

/**
 Closing of a footnote definition's label.
 */
const LABEL_CLOSE = ']:';

/**
 One span of the page text.
 */
export type Span = {
  /**
   Offset of the span's first character.
   */
  readonly start: number;

  /**
   Offset just past the span.
   */
  readonly end: number;
};

/**
 Footnote label a definition line opens with, empty where the line is no
 definition.

 @param line - one line

 @returns Label from `[^` to `]:`

 @example
 ```ts
 footnoteLabel({ line: '[^6]: See the section.', },); // '[^6]:'
 ```
 */
function footnoteLabel({ line, }: { readonly line: string; },): string {
  if (!line.startsWith(LABEL_OPEN,))
    return '';
  /**
   Offset of the label's close, -1 for none.
   */
  const close = line.indexOf(LABEL_CLOSE,);
  if (close === (-1))
    return '';
  return line.slice(
    0,
    close + LABEL_CLOSE.length,
  );
}

/**
 Offset of the line in a text that opens with a label, -1 for none.

 @param pageText - page text of the slice

 @param label - footnote label

 @returns Offset of the line's first character

 @example
 ```ts
 lineStartingWith({ pageText: '[^5]: A.\n\n[^6]: B.', label: '[^6]:', },); // 10
 ```
 */
function lineStartingWith(
  {
    pageText,
    label,
  }: {
    readonly pageText: string;
    readonly label: string;
  },
): number {
  if (pageText.startsWith(label,))
    return 0;
  /**
   Offset of the newline before the labelled line, -1 for none.
   */
  const at = pageText.indexOf(`${LINE_END}${label}`,);
  return (at === (-1)) ? -1 : at + LINE_END.length;
}

/**
 Opening of an HTML tag.
 */
const TAG_OPEN = '<';

/**
 Opening of an HTML closing tag, which frames no line.
 */
const CLOSING_TAG_OPEN = '</';

/**
 Closing of an HTML tag.
 */
const TAG_CLOSE = '>';

/**
 Opening tag a line starts with, empty where the line opens with no tag.

 CLASS ONE HUNDRED FORTY-SIX (XingZ6013, 2026-09-26): the second song credit
 stands on its own `<p style="text-align: end;">` line in a slice whose
 summary quotes a line, and the whole-slice quote search read two spans and
 stood aside; the page writes the credit on a line opening with the same
 tag.

 @param line - one line

 @returns Opening tag through its `>`

 @example
 ```ts
 openingTag({ line: '<p style="text-align: end;">—— 雨猫</p>', },); // '<p style="text-align: end;">'
 ```
 */
function openingTag({ line, }: { readonly line: string; },): string {
  if ((!line.startsWith(TAG_OPEN,)) || line.startsWith(CLOSING_TAG_OPEN,))
    return '';
  /**
   Offset of the tag's close, -1 for none.
   */
  const close = line.indexOf(TAG_CLOSE,);
  if (close === (-1))
    return '';
  return line.slice(
    0,
    close + TAG_CLOSE.length,
  );
}

/**
 Offsets of every line in a text that opens with a frame, in order.

 @param text - text to read

 @param frame - label or tag

 @returns Offsets of the lines' first characters

 @example
 ```ts
 framedLineStarts({ text: '<p>a</p>\n\n<p>b</p>', frame: '<p>', },); // [0, 10]
 ```
 */
function framedLineStarts(
  {
    text,
    frame,
  }: {
    readonly text: string;
    readonly frame: string;
  },
): readonly number[] {
  /**
   Offsets found so far.
   */
  const starts: number[] = [];
  for (
    let at = 0;
    at !== (-1);
    at = lineAfter({
      text,
      at,
    },)
  ) {
    if (text.startsWith(
      frame,
      at,
    ))
      starts.push(at,);
  }
  return starts;
}

/**
 Offset of the line after the one at an offset, -1 at the text's last line.

 @param text - text to read

 @param at - offset inside a line

 @returns Offset of the next line's first character, or -1

 @example
 ```ts
 lineAfter({ text: 'a\nb', at: 0, },); // 2
 ```
 */
function lineAfter(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): number {
  /**
   Offset of the line's end, -1 for none.
   */
  const end = text.indexOf(
    LINE_END,
    at,
  );
  return (end === (-1)) ? -1 : end + LINE_END.length;
}

/**
 Offset of the page line that renders a framed original line: the page's
 line with the same label for a definition, else the page line holding the
 same place among the lines that open with the same tag, where the page
 opens as many lines with it as the original does; -1 otherwise.

 @param sourceText - original text of the slice, comments cut

 @param pageText - page text of the slice

 @param line - original line carrying the title

 @param offset - offset of that line in the original

 @returns Offset of the page line's first character, or -1

 @example
 ```ts
 framedPageLine({ sourceText: '<p>猫</p>', pageText: '<p>Cat</p>', line: '<p>猫</p>', offset: 0, },); // 0
 ```
 */
function framedPageLine(
  {
    sourceText,
    pageText,
    line,
    offset,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly line: string;
    readonly offset: number;
  },
): number {
  /**
   Label the line opens with, if it is a definition.
   */
  const label = footnoteLabel({ line, },);
  if (label !== '')
    return lineStartingWith({
      pageText,
      label,
    },);
  /**
   Opening tag the line starts with, if any.
   */
  const frame = openingTag({ line, },);
  if (frame === '')
    return -1;
  /**
   Original lines opening with the tag.
   */
  const sourceStarts = framedLineStarts({
    text: sourceText,
    frame,
  },);
  /**
   Page lines opening with the tag.
   */
  const pageStarts = framedLineStarts({
    text: pageText,
    frame,
  },);
  if (sourceStarts.length !== pageStarts.length)
    return -1;
  // The line opens with its own tag, so its offset is among the original's
  // starts, and the page has a line at every place the original has one.
  return nonNullishOrThrow(pageStarts[sourceStarts.indexOf(offset,)],);
}

/**
 Span of the page text a bracket or quote search reads: the page's line
 rendering the original line that carries the title, where that line opens
 with a footnote label or a tag the page keeps, the whole text otherwise.

 @param sourceText - original text of the slice, comments cut

 @param pageText - page text of the slice

 @param title - Han title

 @returns Span to search

 @example
 ```ts
 referenceScope({ sourceText: '[^6]: 见「猫」篇', pageText: '[^6]: See “Cat”.', title: '猫', },); // whole line
 ```
 */
export function referenceScope(
  {
    sourceText,
    pageText,
    title,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly title: string;
  },
): Span {
  for (
    let offset = 0;
    offset !== (-1);
    offset = lineAfter({
      text: sourceText,
      at: offset,
    },)
  ) {
    /**
     Offset of the line's end, -1 for the text's end.
     */
    const lineEnd = sourceText.indexOf(
      LINE_END,
      offset,
    );
    /**
     Original line under the scan.
     */
    const line = sourceText.slice(
      offset,
      (lineEnd === (-1)) ? sourceText.length : lineEnd,
    );
    if (!bracketsTitle({
      text: line,
      title,
    },))
      continue;
    /**
     Offset of the page line rendering it, -1 for none.
     */
    const start = framedPageLine({
      sourceText,
      pageText,
      line,
      offset,
    },);
    if (start === (-1))
      continue;
    /**
     Offset of that line's end, -1 for the text's end.
     */
    const end = pageText.indexOf(
      LINE_END,
      start,
    );
    return {
      start,
      end: (end === (-1)) ? pageText.length : end,
    };
  }
  return {
    start: 0,
    end: pageText.length,
  };
}

/**
 One rendering located in the page text.
 */
export type LocatedTitle = {
  /**
   Shape the rendering stands in.
   */
  readonly kind: 'link' | 'gloss' | 'bracket' | 'quote';

  /**
   Offset of the rendering's first character.
   */
  readonly start: number;

  /**
   Offset just past the rendering.
   */
  readonly end: number;
};

/**
 Outcome of a search: a located rendering, an ambiguous slice, or none.
 */
export type TitleLocation = LocatedTitle | { readonly kind: 'ambiguous'; } | { readonly kind: 'none'; };

/**
 Outcome of a slice's search: every rendering located, in page order; an
 ambiguous slice; or none.
 */
export type TitleLocations = {
  readonly kind: 'located';
  readonly renderings: readonly LocatedTitle[];
} | { readonly kind: 'ambiguous'; } | { readonly kind: 'none'; };

//endregion Title reference scope
