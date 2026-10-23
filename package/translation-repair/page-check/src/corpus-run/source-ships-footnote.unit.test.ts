/**
 Tests for how a footnote definition is read: the plain parse takes a line `[^label]: body` for a link reference
 definition when the body is one destination, the strict grammar takes it for a footnote with blocks under it.

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
  footnoteBodyStart,
  isIndentedCode,
  isLazyDefinition,
  opensFootnote,
  rangeOf,
  withoutIndentation,
} from '../../dist/final/node/index.mjs';

/**
 The nodes under the root of a text, read by the plain parse.

 @param source - the text

 @returns Root's children

 @example
 ```ts
 const [first,] = blocksOf({ source: '[^1]: {', },);
 ```
 */
function blocksOf({ source, }: { readonly source: string; },): readonly TreeNode[] {
  return parseMarkdownMathBody({ body: source, },).children;
}

/**
 The only block of a text.

 @param source - the text

 @returns Block

 @throws {@link Error} when the text does not read as one block

 @example
 ```ts
 const block = onlyBlock({ source: '[^1]: {', },);
 ```
 */
function onlyBlock({ source, }: { readonly source: string; },): TreeNode {
  const [block, ...others] = blocksOf({ source, },);
  if ((block === undefined) || (others.length > 0))
    throw new Error('the text does not read as one block',);
  return block;
}

await describe({
  name: 'source ships footnote',
  children: [
    describe({
      name: footnoteBodyStart.name,
      children: [
        it({
          name: 'STARTS THE BODY AFTER THE COLON AND THE WHITE SPACE THAT FOLLOWS IT, and at the end of a '
            + 'definition that has none',
          fn: async () => {
            /**
             A definition with its body on the same line, and one with it on the next.
             */
            const same = '[^1]: <b>';
            const next = '[^2]:\n    {';
            expect(footnoteBodyStart({
              source: same,
              range: rangeOf({ node: onlyBlock({ source: same, },), },),
            },),).toBe(6,);
            expect(footnoteBodyStart({
              source: next,
              range: rangeOf({ node: onlyBlock({ source: next, },), },),
            },),).toBe(10,);
            expect(footnoteBodyStart({
              source: '[^3]:',
              range: {
                start: 0,
                end: 5,
              },
            },),).toBe(5,);
          },
        },),
      ],
    },),

    describe({
      name: opensFootnote.name,
      children: [
        it({
          name: 'READS A DEFINITION WITH A CARET LABEL AND A PARAGRAPH STARTING [^label]: AS THE FIRST LINE OF A '
            + 'FOOTNOTE, and no other block',
          fn: async () => {
            /**
             Whether the only block of a text opens a footnote.

             @param source - the text

             @returns Whether it does
             */
            function opens({ source, }: { readonly source: string; },): boolean {
              const node = onlyBlock({ source, },);
              return opensFootnote({
                source,
                node,
                range: rangeOf({ node, },),
              },);
            }
            expect(opens({ source: '[^1]: {', },),).toBe(true,);
            expect(opens({ source: '[^1]: a b', },),).toBe(true,);
            expect(opens({ source: '[r]: u', },),).toBe(false,);
            expect(opens({ source: '[^1] a', },),).toBe(false,);
            expect(opens({ source: '她说', },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isLazyDefinition.name,
      children: [
        it({
          name: 'CALLS A LINK REFERENCE DEFINITION ON THE LINE RIGHT AFTER A FOOTNOTE DEFINITION A LINE OF THE '
            + 'FOOTNOTE, and no other block',
          fn: async () => {
            /**
             Whether the second block of a text is a line of the footnote the first opens.

             @param source - the text

             @param footnoteOpen - whether the first block is part of a footnote definition

             @returns Whether it is
             */
            function lazy(
              {
                source,
                footnoteOpen,
              }: {
                readonly source: string;
                readonly footnoteOpen: boolean;
              },
            ): boolean {
              const [first, second,] = blocksOf({ source, },);
              if ((first === undefined) || (second === undefined))
                throw new Error('the text holds fewer than two blocks',);
              /**
               Line the first block ends on.
               */
              const previousEndLine = first.position
                ?.end
                .line;
              if (previousEndLine === undefined)
                throw new Error('the first block carries no position',);
              return isLazyDefinition({
                node: second,
                footnoteOpen,
                previousEndLine,
              },);
            }
            expect(lazy({
              source: '[^1]: 注\n[b]: u{\n',
              footnoteOpen: true,
            },),).toBe(true,);
            // A blank line ends the footnote, no footnote is open, the label is a footnote's, or the node is a paragraph.
            expect(lazy({
              source: '[^1]: 注\n\n[b]: u{\n',
              footnoteOpen: true,
            },),).toBe(false,);
            expect(lazy({
              source: '[r]: x\n[b]: u{\n',
              footnoteOpen: false,
            },),).toBe(false,);
            expect(lazy({
              source: '[^1]: 注\n[^2]: u{\n',
              footnoteOpen: true,
            },),).toBe(false,);
            expect(lazy({
              source: '[^1]: 注\n猫 {\n',
              footnoteOpen: true,
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isIndentedCode.name,
      children: [
        it({
          name: 'CALLS CODE THAT DOES NOT START WITH A FENCE INDENTED CODE, which MDX reads as a paragraph',
          fn: async () => {
            /**
             Whether the last block of a text is indented code.

             @param source - the text

             @returns Whether it is
             */
            function indented({ source, }: { readonly source: string; },): boolean {
              const node = blocksOf({ source, },).at(-1,);
              if (node === undefined)
                throw new Error('the text holds no block',);
              return isIndentedCode({
                source,
                node,
                range: rangeOf({ node, },),
              },);
            }
            expect(indented({ source: '猫\n\n    {x}\n', },),).toBe(true,);
            expect(indented({ source: '```\n{x}\n```\n', },),).toBe(false,);
            expect(indented({ source: '~~~\n{x}\n~~~\n', },),).toBe(false,);
            expect(indented({ source: '猫\n', },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: withoutIndentation.name,
      children: [
        it({
          name: 'KEEPS EACH LINE FROM BEHIND ITS INDENTATION OF UP TO FOUR SPACES OR ONE TAB, line endings included',
          fn: async () => {
            /**
             Characters kept from a block, as text.

             @param source - the text, of which the whole is the block

             @returns Kept characters joined
             */
            function kept({ source, }: { readonly source: string; },): string {
              return withoutIndentation({
                source,
                range: {
                  start: 0,
                  end: source.length,
                },
              },)
                .map(function characterAt(at,): string {
                  return source.charAt(at,);
                },)
                .join('',);
            }
            expect(kept({ source: '    ```\n    {\n    ```', },),).toBe('```\n{\n```',);
            expect(kept({ source: '\t注 {\n  \n      b', },),).toBe('注 {\n\n  b',);
            expect(kept({ source: '  x', },),).toBe('x',);
          },
        },),
      ],
    },),
  ],
},);
