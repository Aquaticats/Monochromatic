import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Source ships label line table
// A TABLE THAT THE PLAIN READING STARTS ON THE LABEL LINE OF A FOOTNOTE DEFINITION. The body of a footnote definition is
// read as a text of its own, and its first line is the rest of the line that holds the label. On the page that line is
// part of the footnote's paragraph, and no table starts on it: a line of cells and a line of hyphens after it are two lines
// of a paragraph, in which a code span goes across the pipes and a brace in it is the code's own. Read alone, the same two
// lines are a header and a delimiter row, and the cells cut the code span at every pipe. The delimiter row is written with
// letters in place of its hyphens and colons for the parse, which reads the two lines as the paragraph they are; the parse
// is the encoder's own input, and the text it writes is the source's, so no character of the page changes.

/**
 What takes the place of a hyphen or a colon in the delimiter row.
 */
const LETTER = 'x';

/**
 Characters of a delimiter row that the parse reads as the row's marks.
 */
const ROW_MARKS = '-:';

/**
 The offset a node starts at.

 @param node - node of the parse

 @returns Offset, none for a node without a position

 @example
 ```ts
 const offset = startOffsetOf({ node, },);
 ```
 */
function startOffsetOf({ node, }: { readonly node: TreeNode; },): number | undefined {
  return node.position
    ?.start
    .offset;
}

/**
 Writes the second line of a body with letters in place of its hyphens and colons.

 @param body - text of the body, as the parse reads it

 @returns The body with the second line changed, the body itself when it has no second line

 @example
 ```ts
 const written = withLetteredSecondLine({ body: 'a||\n-|-\n', },); // 'a||\nx|x\n'
 ```
 */
function withLetteredSecondLine({ body, }: { readonly body: string; },): string {
  /**
   Where the first line ends.
   */
  const firstEnd = body.indexOf('\n',);
  if (firstEnd === (-1))
    return body;
  /**
   Where the second line ends.
   */
  const rawEnd = body.indexOf(
    '\n',
    firstEnd + 1,
  );
  /**
   The index after the second line's last character.
   */
  const secondEnd = (rawEnd === (-1)) ? body.length : rawEnd;
  return body.slice(
    0,
    firstEnd + 1,
  )
    + Array.from(body.slice(
      firstEnd + 1,
      secondEnd,
    ),)
      .map(function lettered(character,): string {
        return ROW_MARKS.includes(character,) ? LETTER : character;
      },)
      .join('',)
    + body.slice(secondEnd,);
}

/**
 Takes the table that the plain reading starts on the first line of a body out of the text the parse reads.

 @param body - text of the body of a footnote definition, as the parse reads it, its first line being the rest of the
 label's line

 @returns The body with the delimiter row lettered when the parse reads a table on the first line, the body otherwise

 @example
 ```ts
 const readable = withoutLabelLineTable({ body: '`{||\n-|-\n`', },);
 ```
 */
export function withoutLabelLineTable({ body, }: { readonly body: string; },): string {
  if ((!body.includes('|',)) || (!body.includes('-',)))
    return body;
  /**
   The first block of the body.
   */
  const first = parseMarkdownMathBody({ body, },);
  if (!('children' in first))
    return body;
  /**
   Whether the block is a table that starts on the first line.
   */
  const startsHere = (first.children[0]?.type === 'table') && (startOffsetOf({ node: first.children[0], },) === 0);
  return startsHere ? withLetteredSecondLine({ body, },) : body;
}

//endregion Source ships label line table
