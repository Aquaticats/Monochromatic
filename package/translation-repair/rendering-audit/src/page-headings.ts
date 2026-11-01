//region Page headings
// ONE READER FOR THE HEADINGS A PAGE SHOWS, shared by the page-name glossary
// (`page-name-glossary.ts`), which pairs the archive's rendering of each, and
// the page title spans (`page-title-spans.ts`), which count the titles a page
// repeats. Each kept its own reader, and the glossary's read Markdown headings
// only, so an archive's rendering of an HTML heading never reached the sheets
// (ledger X20). Both read what the page shows (`page-visible-text.ts`), and
// both keep where each heading stands.

/**
 Highest heading level, in Markdown markers or in an HTML tag.
 */
const DEEPEST_HEADING = 6;

/**
 One heading's text and where it stands.

 @example
 ```ts
 const heading: PlacedText = { text: '猫之歌', at: 0, };
 ```
 */
export type PlacedText = {
  /**
   Text a reader sees, trimmed.
   */
  readonly text: string;

  /**
   Offset where the heading's line or tag opens.
   */
  readonly at: number;
};

/**
 Markdown (ATX) headings, one to six markers and a space, by one scan over line
 starts.

 @param text - page text

 @returns Every heading's text and where its line starts, in page order

 @example
 ```ts
 markdownHeadings({ text: '## 猫之歌\n\n喵。', },); // 猫之歌 at 0
 ```
 */
export function markdownHeadings({ text, }: { readonly text: string; },): readonly PlacedText[] {
  /**
   Headings found.
   */
  const headings: PlacedText[] = [];
  for (let start = 0; start <= text.length;) {
    /**
     End of this line, the text's end for the last.
     */
    const newline = text.indexOf(
      '\n',
      start,
    );
    /**
     This line.
     */
    const line = text.slice(
      start,
      (newline === (-1)) ? text.length : newline,
    );
    /**
     Heading markers opening the line, counted by a cursor.
     */
    let depth = 0;
    while (line.charAt(depth,) === '#')
      depth += 1;
    if ((depth >= 1) && (depth <= DEEPEST_HEADING)
      && (line.charAt(depth,) === ' ')) {
      headings.push({
        text: line.slice(depth,)
          .trim(),
        at: start,
      },);
    }
    if (newline === (-1))
      break;
    start = newline + 1;
  }
  return headings;
}

/**
 HTML headings (`<h1>` to `<h6>`) holding no nested tag.

 @param text - page text

 @returns Every such heading's text and where its tag opens, in page order

 @example
 ```ts
 htmlHeadings({ text: '<h3 align="center">猫之歌</h3>', },); // 猫之歌 at 0
 ```
 */
export function htmlHeadings({ text, }: { readonly text: string; },): readonly PlacedText[] {
  /**
   Headings found.
   */
  const headings: PlacedText[] = [];
  for (
    let at = text.indexOf('<h',);
    at !== (-1);
    at = text.indexOf(
      '<h',
      at + 1,
    )
  ) {
    /**
     Heading level, NaN where `<h` opens another tag.
     */
    const level = Math.trunc(
      Number(text.charAt(at + 2,),),
    );
    if (!((level >= 1) && (level <= DEEPEST_HEADING)))
      continue;
    /**
     End of the opening tag.
     */
    const opened = text.indexOf(
      '>',
      at,
    );
    /**
     Start of the closing tag.
     */
    const closing = text.indexOf(
      `</h${String(level,)}>`,
      opened,
    );
    if ((opened === (-1)) || (closing === (-1)))
      break;
    /**
     Heading content.
     */
    const inner = text.slice(
      opened + 1,
      closing,
    );
    if (!inner.includes('<',)) {
      headings.push({
        text: inner.trim(),
        at,
      },);
    }
  }
  return headings;
}

//endregion Page headings
