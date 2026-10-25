/**
 Tests for a block nested in another, read as a text of its own and mapped back to the offsets of the whole text.

 A footnote definition holds blocks under it indented, which the plain parse reads as indented code and the strict
 grammar as blocks of the footnote; the nested reading parses them as a text of their own and says where they stand in
 the whole text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  footnoteBodySpans,
  footnoteInParagraphSpans,
  nestedSpans,
  spansOf,
  withoutContainerAndIndentation,
} from '../../dist/final/node/index.mjs';

/**
 A footnote definition with a paragraph and a fenced block indented under it.
 */
const SOURCE = '[^1]: x\n\n    注 {b} [c](u)\n\n    ```\n    {y}\n    ```\n';

await describe({
  name: 'source ships nested',
  children: [
    describe({
      name: nestedSpans.name,
      children: [
        it({
          name: 'READS THE BLOCKS INDENTED UNDER A FOOTNOTE AS A TEXT OF THEIR OWN and says where each stretch stands '
            + 'in the whole text: the paragraph is text, the link holds its text, the fence is code and no stretch '
            + 'of it is text',
          fn: async () => {
            expect(nestedSpans({
              source: SOURCE,
              readable: SOURCE,
              indices: withoutContainerAndIndentation({
                prefix: '',
                source: SOURCE,
                range: {
                  start: 9,
                  end: SOURCE.length,
                },
              },),
              readSpans: spansOf,
            },),).toEqual({
              text: [
                {
                  start: 13,
                  end: 19,
                },
                {
                  start: 20,
                  end: 21,
                },
              ],
              literal: [],
              edits: [],
              insideLinks: [
                {
                  start: 19,
                  end: 25,
                },
              ],
              streamStarts: [13,],
            },);
          },
        },),

        it({
          name: 'READS NO MODULE LINE IN A NESTED BLOCK, and gives a stretch that crosses the indentation cut away as '
            + 'one stretch for each unbroken run of the whole text',
          fn: async () => {
            /**
             A footnote whose indented paragraph holds a module word at the start of its second line.
             */
            const source = '[^1]: x\n\n    a\n    import b\n';
            expect(nestedSpans({
              source,
              readable: source,
              indices: withoutContainerAndIndentation({
                prefix: '',
                source,
                range: {
                  start: 9,
                  end: source.length,
                },
              },),
              readSpans: spansOf,
            },),).toEqual({
              text: [
                {
                  start: 13,
                  end: 15,
                },
                {
                  start: 19,
                  end: 27,
                },
              ],
              literal: [],
              edits: [],
              insideLinks: [],
              streamStarts: [13,],
            },);
          },
        },),
      ],
    },),

    describe({
      name: footnoteBodySpans.name,
      children: [
        it({
          name: 'READS A FOOTNOTE THAT STARTS ON ITS OWN LINE AS A TEXT, a fence mark among its indented lines opening '
            + 'a fence of the footnote whose braces are code',
          fn: async () => {
            /**
             A footnote whose first line holds a brace and whose indented lines hold a fence with a brace in it.
             */
            const source = '[^1]: 注 {b}\n    ```\n    {y}\n    ```\n';
            expect(footnoteBodySpans({
              source,
              readable: source,
              readSpans: spansOf,
              footnotes: [
                {
                  start: 0,
                  end: source.length,
                },
              ],
              range: {
                start: 6,
                end: source.length,
              },
            },),).toEqual({
              text: [
                {
                  start: 6,
                  end: 11,
                },
              ],
              literal: [],
              edits: [],
              insideLinks: [],
              streamStarts: [6,],
            },);
          },
        },),

        it({
          name: 'DOUBLES THE LINE ENDING THAT ENDS THE FOOTNOTE WHEN A DEFINITION LINE FOLLOWS IT DIRECTLY, so the '
            + 'paragraph the page writes for a tag alone on the last line ends there; a blank line already between '
            + 'them, and a line of text, need no edit',
          fn: async () => {
            /**
             A footnote and a tag alone on its last line, then a definition line.
             */
            const source = '[^1]: x\n<a href="x">\n[r]: u{b}\n';
            /**
             Where the footnote ends: after the tag.
             */
            const end = source.indexOf('\n[r]',);
            expect(footnoteBodySpans({
              source,
              readable: source,
              readSpans: spansOf,
              footnotes: [{ start: 0, end, },],
              range: { start: 6, end: source.length - 1, },
            },).edits,).toContainEqual({ start: end, end: end + 1, replacement: '\n\n', },);
            for (
              const after of ['\n\n[r]: u{b}\n', '\n{z}\n', '\r\n[r]: u{b}\n',] as const
            ) {
              /**
               The same footnote followed by lines that need no blank line, or a Windows line ending.
               */
              const other = `[^1]: x\n<a href="x">${after}`;
              /**
               The edits made for it.
               */
              const { edits, } = footnoteBodySpans({
                source: other,
                readable: other,
                readSpans: spansOf,
                footnotes: [{ start: 0, end, },],
                range: { start: 6, end: other.length - 1, },
              },);
              expect(edits.filter(function doubles({ replacement, },): boolean {
                return replacement
                  .split('\r',)
                  .join('',)
                  .includes('\n\n',);
              },),).toEqual(
                (after === '\r\n[r]: u{b}\n')
                  ? [{ start: end, end: end + 2, replacement: '\r\n\r\n', },]
                  : [],
              );
            }
          },
        },),

        it({
          name: 'DOUBLES THE LINE ENDING WHEN THE BODY ENDS WHERE THE FOOTNOTE DOES, so a definition line the plain '
            + 'reading keeps as a block of its own after the footnote is still a definition on the page',
          fn: async () => {
            /**
             A footnote that is a tag alone, then a definition line.
             */
            const source = '[^1]: <b>\n[r]: u{b}\n';
            /**
             Where the footnote and its body end: after the tag.
             */
            const end = source.indexOf('\n[r]',);
            expect(footnoteBodySpans({
              source,
              readable: source,
              readSpans: spansOf,
              footnotes: [{ start: 0, end, },],
              range: { start: 6, end, },
            },).edits,).toEqual([{ start: end, end: end + 1, replacement: '\n\n', },],);
          },
        },),

        it({
          name: 'READS THE LINES AFTER THE END OF THE FOOTNOTE AS A TEXT OF THEIR OWN: the fence the footnote opens '
            + 'does not reach them, so their braces are text',
          fn: async () => {
            /**
             A footnote whose fence never closes, then a line that is not indented.
             */
            const source = '[^1]: x\n    ```\n    {y}\n{z}\n';
            /**
             Where the footnote ends: after the line of the fence's code.
             */
            const end = source.indexOf('\n{z}',);
            const { text, literal, } = footnoteBodySpans({
              source,
              readable: source,
              readSpans: spansOf,
              footnotes: [
                {
                  start: 0,
                  end,
                },
              ],
              range: {
                start: 6,
                end: source.length - 1,
              },
            },);
            expect(text.some(function holdsZ({
              start,
              end: stop,
            },): boolean {
              return (start <= source.indexOf('{z}',)) && (source.indexOf('{z}',) < stop);
            },),).toBe(true,);
            expect(text.some(function holdsY({
              start,
              end: stop,
            },): boolean {
              return (start <= source.indexOf('{y}',)) && (source.indexOf('{y}',) < stop);
            },),).toBe(false,);
            expect(literal,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: footnoteInParagraphSpans.name,
      children: [
        it({
          name: 'READS THE LINES BEFORE A FOOTNOTE DEFINITION AS A TEXT OF THEIR OWN AND THE DEFINITION AS THE '
            + 'FOOTNOTE, so a code span that the plain reading runs across the definition holds nothing past it',
          fn: async () => {
            /**
             A paragraph whose code span would run across the definition's line.
             */
            const source = 'a `b\n[^1]: c {d}\ne` f\n';
            /**
             Where the definition starts.
             */
            const split = source.indexOf('[^1]',);
            const { text, } = footnoteInParagraphSpans({
              source,
              readable: source,
              readSpans: spansOf,
              footnotes: [
                {
                  start: split,
                  end: source.length - 1,
                },
              ],
              range: {
                start: 0,
                end: source.length - 1,
              },
              split,
              end: source.length - 1,
            },);
            /**
             Whether a stretch read as text holds the brace.
             */
            const holdsBrace = text.some(function holds({
              start,
              end,
            },): boolean {
              return (start <= source.indexOf('{d}',)) && (source.indexOf('{d}',) < end);
            },);
            expect(holdsBrace,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
