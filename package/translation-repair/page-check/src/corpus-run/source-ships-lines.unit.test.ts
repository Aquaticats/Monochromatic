/**
 Tests for the line endings the strict grammar reads a block at: a brace or a less-than sign that stands first on a line
 starts an expression or a tag, and ends the paragraph, which cuts a code span, inline math or a title that the plain
 reading keeps whole.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure';

import {
  contentStretches,
  joinedLines,
  lineJoinsOf,
  rangeOf,
} from '../../dist/final/node/index.mjs';

/**
 What the lines of a text are joined by, the text being read whole.

 @param source - text

 @returns Edits

 @example
 ```ts
 const edits = joinedWhole({ source: '`a\n{b`', },);
 ```
 */
function joinedWhole({ source, }: { readonly source: string; },): ReturnType<typeof joinedLines> {
  return joinedLines({
    source,
    range: {
      start: 0,
      end: source.length,
    },
  },);
}

/**
 What the lines of a node are joined by.

 @param source - text

 @param node - node of the parse

 @returns Edits

 @example
 ```ts
 const edits = joinsOfNode({ source, node, },);
 ```
 */
function joinsOfNode(
  {
    source,
    node,
  }: {
    readonly source: string;
    readonly node: TreeNode;
  },
): ReturnType<typeof lineJoinsOf> {
  return lineJoinsOf({
    source,
    node,
    range: rangeOf({ node, },),
  },);
}

/**
 What the lines of the first block of a text, or of its first inline node, are joined by.

 @param source - text

 @param inline - whether to read the first child of the first block and not the block

 @returns Edits

 @throws {@link Error} when the text holds no such node

 @example
 ```ts
 const edits = joinsOfFirst({ source: '`a\n{b`', inline: true, },);
 ```
 */
function joinsOfFirst(
  {
    source,
    inline,
  }: {
    readonly source: string;
    readonly inline: boolean;
  },
): ReturnType<typeof lineJoinsOf> {
  const [block,] = parseMarkdownMathBody({ body: source, },).children;
  if (block === undefined)
    throw new Error('the text holds no block',);
  if (!inline)
    return joinsOfNode({
      source,
      node: block,
    },);
  if (!('children' in block))
    throw new Error('the block holds no children',);
  const [child,] = block.children;
  if (child === undefined)
    throw new Error('the block holds no inline node',);
  return joinsOfNode({
    source,
    node: child,
  },);
}

/**
 The content stretches of the first raw HTML or code node of a text.

 @param source - text

 @returns Stretches of the content of the node's lines

 @throws {@link Error} when the text holds no such node

 @example
 ```ts
 const stretches = contentOfFirst({ source: '> <div>\n> a\n', },);
 ```
 */
function contentOfFirst({ source, }: { readonly source: string; },): ReturnType<typeof contentStretches> {
  /**
   Nodes still to read, the first of them on top.
   */
  const pending: TreeNode[] = [parseMarkdownMathBody({ body: source, },),];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if ((node.type === 'html') || (node.type === 'code')) {
      return contentStretches({
        source,
        node,
        range: rangeOf({ node, },),
      },);
    }
    if ('children' in node)
      pending.push(...node.children.toReversed(),);
  }
  throw new Error('the text holds no raw HTML or code node',);
}

await describe({
  name: 'source ships lines',
  children: [
    describe({
      name: contentStretches.name,
      children: [
        it({
          name: 'GIVES THE CONTENT OF EACH LINE OF A RAW HTML BLOCK, the marks of a block quote left out',
          fn: async () => {
            expect(contentOfFirst({ source: '> <div>\n> a\n', },),).toEqual([
              {
                start: 2,
                end: 7,
              },
              {
                start: 10,
                end: 11,
              },
            ],);
          },
        },),

        it({
          name: 'GIVES THE CONTENT OF EACH LINE OF INDENTED CODE, the indentation left out and an empty line given '
            + 'none',
          fn: async () => {
            expect(contentOfFirst({ source: '猫\n\n    a\n\n     b\n', },),).toEqual([
              {
                start: 7,
                end: 8,
              },
              {
                start: 14,
                end: 16,
              },
            ],);
          },
        },),

        it({
          name: 'KEEPS THE CARRIAGE RETURN OF A LINE IN ITS CONTENT, which the value of the node keeps too',
          fn: async () => {
            expect(contentOfFirst({ source: '<div>\r\nab\r\n</div>\r\n', },),).toEqual([
              {
                start: 0,
                end: 6,
              },
              {
                start: 7,
                end: 10,
              },
              {
                start: 11,
                end: 17,
              },
            ],);
          },
        },),

        it({
          name: 'GIVES THE WHOLE RANGE FOR A NODE WITH NO VALUE, and for lines that do not correspond to its value',
          fn: async () => {
            /**
             A paragraph, which holds no value of its own.
             */
            const [paragraph,] = parseMarkdownMathBody({ body: 'ab\n', },).children;
            if (paragraph === undefined)
              throw new Error('the text holds no block',);
            expect(contentStretches({
              source: 'ab\n',
              node: paragraph,
              range: rangeOf({ node: paragraph, },),
            },),).toEqual([rangeOf({ node: paragraph, },),],);
            /**
             A code node read from other text than the lines it is given.
             */
            const [code,] = parseMarkdownMathBody({ body: '```\nabcdef\n```\n', },).children;
            if (code === undefined)
              throw new Error('the text holds no block',);
            expect(contentStretches({
              source: 'x\ny\n',
              node: code,
              range: {
                start: 0,
                end: 1,
              },
            },),).toEqual([
              {
                start: 0,
                end: 1,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: joinedLines.name,
      children: [
        it({
          name: 'JOINS A LINE ENDING TO A SPACE WHEN THE LINE AFTER IT STARTS WITH A BRACE OR A LESS-THAN SIGN, the '
            + 'indentation and the marks of a block quote after the ending going with it',
          fn: async () => {
            expect(joinedWhole({ source: '`a\n{b`', },),).toEqual([
              {
                start: 2,
                end: 3,
                replacement: ' ',
              },
            ],);
            expect(joinedWhole({ source: '`a\n  <b`', },),).toEqual([
              {
                start: 2,
                end: 5,
                replacement: ' ',
              },
            ],);
            expect(joinedWhole({ source: '`a\n> {b`', },),).toEqual([
              {
                start: 2,
                end: 5,
                replacement: ' ',
              },
            ],);
            expect(joinedWhole({ source: '`a\n{b\n<c`', },),).toEqual([
              {
                start: 2,
                end: 3,
                replacement: ' ',
              },
              {
                start: 5,
                end: 6,
                replacement: ' ',
              },
            ],);
          },
        },),

        it({
          name: 'TAKES THE CARRIAGE RETURN OF A LINE ENDING WITH IT, and leaves a line that starts with anything else',
          fn: async () => {
            expect(joinedWhole({ source: '`a\r\n{b`', },),).toEqual([
              {
                start: 2,
                end: 4,
                replacement: ' ',
              },
            ],);
            expect(joinedWhole({ source: '`a\nb {c`', },),).toEqual([],);
            expect(joinedWhole({ source: '`a\n', },),).toEqual([],);
            expect(joinedWhole({ source: '`a\n  ', },),).toEqual([],);
          },
        },),

        it({
          name: 'LOOKS ONLY INSIDE THE STRETCH IT IS GIVEN, so a brace that starts the line after a node is not joined '
            + 'to it',
          fn: async () => {
            expect(joinedLines({
              source: '`a`\n{b',
              range: {
                start: 0,
                end: 3,
              },
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: lineJoinsOf.name,
      children: [
        it({
          name: 'JOINS THE LINES OF A CODE SPAN AND OF INLINE MATH',
          fn: async () => {
            expect(joinsOfFirst({
              source: '`a\n{b`',
              inline: true,
            },),).toEqual([
              {
                start: 2,
                end: 3,
                replacement: ' ',
              },
            ],);
            expect(joinsOfFirst({
              source: '$a\n<b$',
              inline: true,
            },),).toEqual([
              {
                start: 2,
                end: 3,
                replacement: ' ',
              },
            ],);
          },
        },),

        it({
          name: 'JOINS THE LINES OF THE DESTINATION AND THE TITLE OF A LINK, AN IMAGE AND A DEFINITION, and not those '
            + 'of the text of a link',
          fn: async () => {
            /**
             The one edit expected, at an index.
             */
            const at = (start: number, end: number,) => [
              {
                start,
                end,
                replacement: ' ',
              },
            ];
            expect(joinsOfFirst({
              source: '[a](u "t\n{b")',
              inline: true,
            },),).toEqual(at(8, 9,),);
            expect(joinsOfFirst({
              source: '![a](u "t\n<b")',
              inline: true,
            },),).toEqual(at(9, 10,),);
            expect(joinsOfFirst({
              source: '[r]: u "t\n{b"',
              inline: false,
            },),).toEqual(at(9, 10,),);
            expect(joinsOfFirst({
              source: '[a\n{b](u)',
              inline: true,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'JOINS NOTHING OF A FOOTNOTE DEFINITION, AN ANGLE AUTOLINK OR A TEXT, whose braces are escaped or are '
            + 'no syntax',
          fn: async () => {
            expect(joinsOfFirst({
              source: '[^1]: x\n{b',
              inline: false,
            },),).toEqual([],);
            expect(joinsOfFirst({
              source: '<https://example.net/a>',
              inline: true,
            },),).toEqual([],);
            expect(joinsOfFirst({
              source: 'a\n{b',
              inline: true,
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
