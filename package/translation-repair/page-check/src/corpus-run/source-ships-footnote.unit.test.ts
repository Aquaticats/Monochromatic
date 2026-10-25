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
  footnoteDefinitionRangesOf,
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure';

import {
  continuesDefinition,
  footnoteBodyStart,
  footnoteStartInside,
  isIndentedCode,
  isLazyDefinition,
  NO_FOOTNOTE_INSIDE,
  opensFootnote,
  rangeOf,
  withoutIndentation,
  withoutQuoteMarks,
} from '../../dist/final/node/index.mjs';

/**
 The nodes under the root of a text, read by the plain parse.

 @param source - text

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

 @param source - text

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

/**
 Cases of the marks of a block quote a line gives up: what it is, the text, where the stretch starts, and the
 characters kept.
 */
const MARK_CASES: readonly (readonly [string, string, number, string])[] = [
  ['A MARK AND THE SPACE AFTER IT', '> a\n> b', 0, 'a\nb',],
  ['A MARK WITH NO SPACE AFTER IT', '>a\n>b', 0, 'a\nb',],
  ['A MARK AND THE TAB AFTER IT', '>\ta', 0, 'a',],
  ['MARKS ONE AFTER THE OTHER, A NESTED QUOTE', '> > a\n>> b', 0, 'a\nb',],
  ['UP TO THREE SPACES BEFORE A MARK', '   > a\n > b', 0, 'a\nb',],
  ['NO MARK IN FOUR SPACES BEFORE A SIGN, the indentation of a line of a paragraph', '    > a', 0, '    > a',],
  ['NO MARK ON A LINE WITH NONE, a lazy line of the quote', '> a\nb\n  c', 0, 'a\nb\n  c',],
  ['NO MARK BEFORE THE FIRST LINE OF A STRETCH THAT STARTS INSIDE A LINE', '> a\n> b', 2, 'a\nb',],
  ['THE MARK AND ONE SPACE ONLY, the extra spaces being indentation', '>    a', 0, '   a',],
];

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
          name: 'READS A DEFINITION WITH A CARET LABEL, A PARAGRAPH AND A HEADING STARTING [^label]: AS THE FIRST LINE '
            + 'OF A FOOTNOTE, and no other block',
          fn: async () => {
            /**
             Whether the only block of a text opens a footnote.

             @param source - text

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
            expect(opens({ source: '[^1]: x {\ny\n===\n', },),).toBe(true,);
            expect(opens({ source: '# [^1]: x {\n', },),).toBe(false,);
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

             @param source - text

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
            // A blank line ends the footnote, no footnote is open, the label is
            // a footnote's, or the node is a paragraph.
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

             @param source - text

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
      name: continuesDefinition.name,
      children: [
        it({
          name: 'TAKES A PARAGRAPH ON THE LINE RIGHT AFTER THE DEFINITION AS ITS CONTINUATION, and no paragraph after '
            + 'a blank line and no block that is not a paragraph',
          fn: async () => {
            /**
             The blocks of a definition that a paragraph goes on in.
             */
            const [definition, adjacent,] = blocksOf({ source: '[^1]: https://example.net/a\n注\n', },);
            /**
             The blocks of a definition that a blank line parts from the next block.
             */
            const [, parted,] = blocksOf({ source: '[^1]: https://example.net/a\n\n注\n', },);
            /**
             The blocks of a definition followed by a heading.
             */
            const [, heading,] = blocksOf({ source: '[^1]: https://example.net/a\n# 注\n', },);
            if (
              (definition === undefined)
                || (adjacent === undefined)
                || (parted === undefined)
                || (heading === undefined)
            )
              throw new Error('the texts do not read as two blocks',);
            expect(continuesDefinition({
              definition,
              block: adjacent,
            },),).toBe(true,);
            expect(continuesDefinition({
              definition,
              block: parted,
            },),).toBe(false,);
            expect(continuesDefinition({
              definition,
              block: heading,
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: footnoteStartInside.name,
      children: [
        it({
          name: 'GIVES WHERE A FOOTNOTE DEFINITION STARTS ON A LATER LINE OF A PARAGRAPH, and says none does for a '
            + 'paragraph that opens one, one that holds none and a definition after the paragraph',
          fn: async () => {
            /**
             A paragraph whose code span runs across a footnote definition's line, then a footnote of its own.
             */
            const source = 'a `b\n[^1]: c\ne` f\n\n[^2]: g\n';
            /**
             Where the strict grammar reads the footnotes.
             */
            const footnotes = footnoteDefinitionRangesOf({ body: source, },);
            expect(footnoteStartInside({
              footnotes,
              range: {
                start: 0,
                end: source.indexOf('\n\n',),
              },
            },),).toBe(source.indexOf('[^1]',),);
            expect(footnoteStartInside({
              footnotes,
              range: {
                start: source.indexOf('[^1]',),
                end: source.indexOf('\n\n',),
              },
            },),).toBe(NO_FOOTNOTE_INSIDE,);
            expect(footnoteStartInside({
              footnotes: [],
              range: {
                start: 0,
                end: source.indexOf('\n\n',),
              },
            },),).toBe(NO_FOOTNOTE_INSIDE,);
            expect(footnoteStartInside({
              footnotes,
              range: {
                start: 0,
                end: source.indexOf('[^1]',),
              },
            },),).toBe(NO_FOOTNOTE_INSIDE,);
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

             @param source - text, of which the whole is the block

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

    describe({
      name: withoutQuoteMarks.name,
      children: MARK_CASES.map(function markCase([name, source, start, expected,],) {
        return it({
          name: `KEEPS EACH LINE FROM BEHIND ${name}`,
          fn: async () => {
            expect(withoutQuoteMarks({
              source,
              range: {
                start,
                end: source.length,
              },
            },)
              .map(function characterAt(at,): string {
                return source.charAt(at,);
              },)
              .join('',),).toBe(expected,);
          },
        },);
      },),
    },),
  ],
},);
