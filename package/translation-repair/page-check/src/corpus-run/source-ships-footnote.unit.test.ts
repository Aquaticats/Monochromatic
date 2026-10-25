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
  hasFootnoteLabel,
  isFootnoteLabel,
  isIndentedCode,
  isLazyDefinition,
  NO_FOOTNOTE_INSIDE,
  opensFootnote,
  rangeOf,
  withoutContainerAndIndentation,
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
 Whether the only block of a text opens a footnote.

 @param source - text

 @returns Whether it does

 @example
 ```ts
 const yes = opens({ source: '[^1]: {', },);
 ```
 */
function opens({ source, }: { readonly source: string; },): boolean {
  const node = onlyBlock({ source, },);
  return opensFootnote({
    source,
    node,
    range: rangeOf({ node, },),
  },);
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

/**
 The most characters after the caret that the strict grammar reads in a label.
 */
const MOST_READ = 999;

/**
 Cases of a label as the strict grammar reads the label of a footnote definition (`micromark-extension-gfm-footnote@2.1.0`,
 `lib/syntax.js:354-381`, `labelInside` and `labelEscape`): the label with its caret, and whether it is one.
 */
const LABEL_CASES: readonly (readonly [string, string, boolean])[] = [
  ['A CARET AND ONE CHARACTER', '^1', true,],
  ['A NAME OF LETTERS AND DIGITS', '^note2', true,],
  ['NO CARET', 'note', false,],
  ['A CARET AND NOTHING', '^', false,],
  ['A SPACE IN THE NAME', '^a b', false,],
  ['A TAB IN THE NAME', '^a\tb', false,],
  ['A LINE FEED IN THE NAME', '^a\nb', false,],
  ['A CARRIAGE RETURN IN THE NAME', '^a\rb', false,],
  ['A NO-BREAK SPACE IN THE NAME, which is no white space to the strict grammar', '^a\u00A0b', true,],
  ['AN OPENING BRACKET IN THE NAME', '^a[b', false,],
  ['AN ESCAPED OPENING BRACKET IN THE NAME', String.raw`^a\[b`, true,],
  ['AN ESCAPED BACKSLASH IN THE NAME', String.raw`^a\\b`, true,],
  ['AN ESCAPED CLOSING BRACKET IN THE NAME', String.raw`^a\]b`, true,],
  ['A BACKSLASH BEFORE AN ORDINARY CHARACTER', String.raw`^a\b`, true,],
  ['A BACKSLASH LAST, which escapes the closing bracket', '^a\\', false,],
  ['A BACKSLASH BEFORE A SPACE', String.raw`^a\ b`, false,],
  ['A NAME OF 999 CHARACTERS, the most the strict grammar reads', `^${'a'.repeat(MOST_READ,)}`, true,],
  ['A NAME OF 1000 CHARACTERS', `^${'a'.repeat(MOST_READ + 1,)}`, false,],
  ['A NAME OF 999 CHARACTERS WITH AN ESCAPE, which counts two', `^${'a'.repeat(MOST_READ - 1,)}\\[`, false,],
];

await describe({
  name: 'source ships footnote',
  children: [
    describe({
      name: isFootnoteLabel.name,
      children: LABEL_CASES.map(function labelCase([name, label, expected,],) {
        return it({
          name: `CALLS ${name} ${expected ? 'A' : 'NOT A'} FOOTNOTE LABEL`,
          fn: async () => {
            expect(isFootnoteLabel({ label, },),).toBe(expected,);
          },
        },);
      },),
    },),

    describe({
      name: hasFootnoteLabel.name,
      children: [
        it({
          name: 'READS A LINK REFERENCE DEFINITION AS A FOOTNOTE DEFINITION BY ITS LABEL, an escaped bracket in it '
            + 'included, and no other block',
          fn: async () => {
            /**
             Whether the only block of a text is a definition with the label of a footnote.
             */
            function labelled({ source, }: { readonly source: string; },): boolean {
              return hasFootnoteLabel({
                source,
                node: onlyBlock({ source, },),
              },);
            }
            expect(labelled({ source: '[^1]: https://example.net/a', },),).toBe(true,);
            expect(labelled({ source: String.raw`[^a\[b]: https://example.net/a`, },),).toBe(true,);
            expect(labelled({ source: '[^a b]: https://example.net/a', },),).toBe(false,);
            // The identifier of the parse is trimmed, so a space last in the label shows in the source only.
            expect(labelled({ source: '[^a ]: https://example.net/a', },),).toBe(false,);
            expect(labelled({ source: '  [^1]: https://example.net/a', },),).toBe(true,);
            expect(labelled({ source: '[r]: https://example.net/a', },),).toBe(false,);
            expect(labelled({ source: '[^1]: a b', },),).toBe(false,);
          },
        },),

        it({
          name: 'READS THE LABEL AS WRITTEN, not as the parse decodes it: a character reference for a space is no '
            + 'space to the strict grammar, which reads the characters of the source',
          fn: async () => {
            /**
             The definition node of a text and whether its label is a footnote's.
             */
            const source = '[^a&#32;b]: https://example.net/a';
            expect(hasFootnoteLabel({
              source,
              node: onlyBlock({ source, },),
            },),).toBe(true,);
          },
        },),
      ],
    },),

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
            expect(opens({ source: '[^1]: {', },),).toBe(true,);
            expect(opens({ source: '[^1]: x {\ny\n===\n', },),).toBe(true,);
            expect(opens({ source: '# [^1]: x {\n', },),).toBe(false,);
            expect(opens({ source: '[^1]: a b', },),).toBe(true,);
            expect(opens({ source: '[r]: u', },),).toBe(false,);
            expect(opens({ source: '[^1] a', },),).toBe(false,);
            expect(opens({ source: '她说', },),).toBe(false,);
          },
        },),

        it({
          name: 'READS NO FOOTNOTE DEFINITION IN A PARAGRAPH WHOSE LABEL HOLDS A BRACKET OR IS LONGER THAN THE STRICT '
            + 'GRAMMAR READS',
          fn: async () => {
            expect(opens({ source: `[^${'a'.repeat(MOST_READ + 1,)}]: {`, },),).toBe(false,);
            expect(opens({ source: '[^a[b]]: {', },),).toBe(false,);
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
                source,
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
      name: withoutContainerAndIndentation.name,
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
              return withoutContainerAndIndentation({
                prefix: '',
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
