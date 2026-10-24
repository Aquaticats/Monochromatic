import type {
  BlockRange,
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  closingBracket,
  FENCE_MARKS,
  indicesOf,
  startsAny,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships footnote
// HOW A FOOTNOTE DEFINITION IS READ, which the plain parse reads one way and the strict grammar (with GFM footnotes
// on) reads another: a line `[^label]: body` that CommonMark takes for a link reference definition is a footnote whose
// body is blocks, and the blocks indented under a footnote definition are its own and not indented code. A link
// reference definition on the line right after one is a line of the footnote's paragraph to the strict grammar, since
// a definition cannot interrupt a paragraph and the paragraph of the footnote continues with every unindented line.

/**
 White space between the colon of a footnote definition and its body.
 */
const FOOTNOTE_GAP = ' \t\r\n';

/**
 Each white space character of a footnote label's forbidden set, one by one.
 */
const FOOTNOTE_GAPS: readonly string[] = [
  ' ',
  '\t',
  '\r',
  '\n',
];

/**
 Whether a label is spelled as the label of a footnote definition: a caret and at least one more character, none of
 them white space.

 @param label - the text between the brackets

 @returns Whether the label is one a footnote definition can have

 @example
 ```ts
 const spelled = isFootnoteLabel({ label: '^1', },); // true
 ```
 */
export function isFootnoteLabel({ label, }: { readonly label: string; },): boolean {
  if ((label.length < 2) || (!label.startsWith('^',)))
    return false;
  return FOOTNOTE_GAPS.every(function absent(gap,): boolean {
    return !label.includes(gap,);
  },);
}

/**
 Whether a link reference definition of the plain reading has the label of a footnote definition. CommonMark reads
 `[^a b]: x` as a link reference definition and so does GFM, whose footnote label holds no space, tab or line ending
 (`micromark-extension-gfm-footnote@2.1.0`, `lib/syntax.js`, `labelInside`), so a definition with such a label is no
 footnote definition to the strict grammar.

 @param node - node of the parse

 @returns Whether the node is a link reference definition the strict grammar reads as a footnote definition

 @example
 ```ts
 const footnote = hasFootnoteLabel({ node, },);
 ```
 */
export function hasFootnoteLabel({ node, }: { readonly node: TreeNode; },): boolean {
  if (node.type !== 'definition')
    return false;
  // The label as written, or its identifier where the parse kept no label.
  return isFootnoteLabel({ label: node.label ?? node.identifier, },);
}

/**
 How many columns a block under a footnote definition is indented by.
 */
const FOOTNOTE_INDENT = 4;

/**
 What {@link footnoteStartInside} answers for a paragraph no footnote definition starts inside.
 */
export const NO_FOOTNOTE_INSIDE = -1;

/**
 Where a footnote definition starts inside a paragraph of the plain reading, on a line after the paragraph's first. The
 plain reading has no footnotes, so it keeps the definition's line in the paragraph, and the inline constructs it reads
 across that line (a code span, math, a link) go on past a place where the strict grammar ends the paragraph.

 @param footnotes - where the strict grammar reads a footnote definition in the whole text

 @param range - where the paragraph stands

 @returns Index of the definition's first character, or {@link NO_FOOTNOTE_INSIDE} when none starts in the paragraph

 @example
 ```ts
 const split = footnoteStartInside({ footnotes: [{ start: 4, end: 12, },], range: { start: 0, end: 12, }, },);
 ```
 */
export function footnoteStartInside(
  {
    footnotes,
    range,
  }: {
    readonly footnotes: readonly BlockRange[];
    readonly range: TextRange;
  },
): number {
  return footnotes.find(function startsInside({ start, },): boolean {
    return (start > range.start) && (start < range.end);
  },)
    ?.start
    ?? NO_FOOTNOTE_INSIDE;
}

/**
 Where the body of a footnote definition starts: after its label, the colon and the white space that follows.

 @param source - text

 @param range - where the definition stands

 @returns Index of the first character of the body, or the end of the definition when it has none

 @example
 ```ts
 const from = footnoteBodyStart({ source: '[^1]: 注\n', range: { start: 0, end: 7, }, },);
 ```
 */
export function footnoteBodyStart(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
  },
): number {
  /**
   Where the bracket closing the label stands, the colon being the character after it.
   */
  const close = closingBracket({
    source,
    from: range.start + 1,
    limit: range.end,
  },);
  /**
   Index of the character after the colon.
   */
  const open = { at: close + 2, };
  while ((open.at < range.end) && FOOTNOTE_GAP.includes(source.charAt(open.at,),))
    open.at += 1;
  return Math.min(
    open.at,
    range.end,
  );
}

/**
 Whether a node opens a footnote definition that the strict grammar reads: a paragraph starting `[^label]:`, a heading
 that starts so (its underline is a lazy line of the footnote's paragraph to the strict grammar), or a link reference
 definition whose label starts with a caret and holds no white space, which CommonMark reads as that and GFM as a
 footnote.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands

 @returns Whether the node is the first line of a footnote definition

 @example
 ```ts
 const opens = opensFootnote({ source, node, range, },);
 ```
 */
export function opensFootnote(
  {
    source,
    node,
    range,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
  },
): boolean {
  if (node.type === 'definition')
    return hasFootnoteLabel({ node, },);
  if ((node.type !== 'paragraph') && (node.type !== 'heading'))
    return false;
  if (!source.startsWith(
    '[^',
    range.start,
  ))
    return false;
  /**
   Where the bracket that closes the label stands.
   */
  const close = closingBracket({
    source,
    from: range.start + 1,
    limit: range.end,
  },);
  return (source.charAt(close + 1,) === ':') && isFootnoteLabel({
    label: source.slice(
      range.start + 1,
      close,
    ),
  },);
}

/**
 Whether a block is the paragraph that goes on right after a footnote definition: it starts on the line after the
 definition ends, and the strict grammar reads its lines as lines of the footnote.

 @param definition - link reference definition that CommonMark reads and GFM reads as a footnote definition

 @param block - block that comes next in the text

 @returns Whether the block continues the footnote definition

 @example
 ```ts
 const continues = continuesDefinition({ definition, block, },);
 ```
 */
export function continuesDefinition(
  {
    definition,
    block,
  }: {
    readonly definition: TreeNode;
    readonly block: TreeNode;
  },
): boolean {
  if (block.type !== 'paragraph')
    return false;
  return block.position
    ?.start
    .line
    === (endLineOf({
      node: definition,
      fallback: 0,
    },) + 1);
}

/**
 The line a node ends on.

 @param node - node of the parse

 @param fallback - line to give for a node that carries no position

 @returns One-based line

 @example
 ```ts
 const line = endLineOf({ node, fallback: 0, },);
 ```
 */
export function endLineOf(
  {
    node,
    fallback,
  }: {
    readonly node: TreeNode;
    readonly fallback: number;
  },
): number {
  return node.position
    ?.end
    .line
    ?? fallback;
}

/**
 Whether a node is a link reference definition on the line right after a footnote definition, which the strict grammar
 reads as a line of the footnote's paragraph and not as a definition.

 @param node - node of the parse

 @param footnoteOpen - whether the block read just before is part of a footnote definition

 @param previousEndLine - line the block read just before ends on

 @returns Whether the node is text of the footnote

 @example
 ```ts
 const lazy = isLazyDefinition({ node, footnoteOpen: true, previousEndLine: 3, },);
 ```
 */
export function isLazyDefinition(
  {
    node,
    footnoteOpen,
    previousEndLine,
  }: {
    readonly node: TreeNode;
    readonly footnoteOpen: boolean;
    readonly previousEndLine: number;
  },
): boolean {
  if (!footnoteOpen)
    return false;
  if (node.type !== 'definition')
    return false;
  if (hasFootnoteLabel({ node, },))
    return false;
  return node.position
    ?.start
    .line
    === (previousEndLine + 1);
}

/**
 Whether a node is code the strict grammar reads as a paragraph: indented code, since MDX has no such thing.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands

 @returns Whether the node is code that does not start with a fence

 @example
 ```ts
 const indented = isIndentedCode({ source, node, range, },);
 ```
 */
export function isIndentedCode(
  {
    source,
    node,
    range,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
  },
): boolean {
  if (node.type !== 'code')
    return false;
  return !startsAny({
    text: source,
    at: range.start,
    candidates: FENCE_MARKS,
  },);
}

/**
 Where the line starting at an index ends.

 @param source - text

 @param from - index the line starts at

 @param limit - index the block ends at, which a line does not pass

 @returns Index after the line ending, or the limit for the last line

 @example
 ```ts
 const end = lineEndAfter({ source: 'a\nb', from: 0, limit: 3, },);
 ```
 */
function lineEndAfter(
  {
    source,
    from,
    limit,
  }: {
    readonly source: string;
    readonly from: number;
    readonly limit: number;
  },
): number {
  /**
   Index of the line ending.
   */
  const lineBreak = source.indexOf(
    '\n',
    from,
  );
  if (lineBreak === (-1))
    return limit;
  return Math.min(
    lineBreak + 1,
    limit,
  );
}

/**
 The indices of the characters of an indented block that remain once each line gives up its indentation of up to four
 spaces or one tab, which is how a footnote definition reads the blocks indented under it.

 @param source - text

 @param range - where the block stands, from the start of its first line

 @returns Indices in text order

 @example
 ```ts
 const kept = withoutIndentation({ source, range, },);
 ```
 */
export function withoutIndentation(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
  },
): readonly number[] {
  /**
   Indices kept.
   */
  const kept: number[] = [];
  for (let lineStart = range.start; lineStart < range.end;) {
    /**
     Index after the line, its ending included.
     */
    const lineEnd = lineEndAfter({
      source,
      from: lineStart,
      limit: range.end,
    },);
    /**
     How many characters of indentation the line gives up.
     */
    const skipped = { count: 0, };
    while ((skipped.count < FOOTNOTE_INDENT) && (source.charAt(lineStart + skipped.count,) === ' '))
      skipped.count += 1;
    if ((skipped.count < FOOTNOTE_INDENT) && (source.charAt(lineStart + skipped.count,) === '\t'))
      skipped.count += 1;
    kept.push(...indicesOf({
      range: {
        start: Math.min(
          lineStart + skipped.count,
          lineEnd,
        ),
        end: lineEnd,
      },
    },),);
    lineStart = lineEnd;
  }
  return kept;
}

//endregion Source ships footnote
