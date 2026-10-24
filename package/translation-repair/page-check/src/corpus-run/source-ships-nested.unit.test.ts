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
  nestedSpans,
  spansOf,
  withoutIndentation,
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
          name: 'READS THE BLOCKS INDENTED UNDER A FOOTNOTE AS A TEXT OF THEIR OWN and says where each stretch stands in '
            + 'the whole text: the paragraph is text, the link holds its text, the fence is code and no stretch of it is '
            + 'text',
          fn: async () => {
            expect(nestedSpans({
              source: SOURCE,
              readable: SOURCE,
              indices: withoutIndentation({
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
          name: 'READS NO MODULE LINE IN A NESTED BLOCK, and gives a stretch that crosses the indentation cut away as one '
            + 'stretch for each unbroken run of the whole text',
          fn: async () => {
            /**
             A footnote whose indented paragraph holds a module word at the start of its second line.
             */
            const source = '[^1]: x\n\n    a\n    import b\n';
            expect(nestedSpans({
              source,
              readable: source,
              indices: withoutIndentation({
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
          name: 'READS A FOOTNOTE THAT STARTS ON ITS OWN LINE AS A TEXT, a fence mark among its indented lines opening a '
            + 'fence of the footnote whose braces are code',
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
          name: 'READS THE LINES AFTER THE END OF THE FOOTNOTE AS A TEXT OF THEIR OWN: the fence the footnote opens does '
            + 'not reach them, so their braces are text',
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
  ],
},);
