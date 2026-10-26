/**
 Tests the blank line put between the end of a footnote and the block that follows it directly.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  afterLiteralBlockEdits,
  separatingEdits,
} from '../../dist/final/node/index.mjs';

/**
 The edits for a text whose footnote ends at a given line ending and whose rest is read to the end.

 @param source - whole text

 @param footnoteEnd - index of the line ending of the footnote's last line

 @param pageGoesOn - whether the page goes on with the footnote's paragraph

 @returns Edits that separate the footnote from what follows

 @example
 ```ts
 const edits = editsFor({ source: '[^1]: x\n[r]: u\n', footnoteEnd: 7, pageGoesOn: false, },);
 ```
 */
function editsFor(
  {
    source,
    footnoteEnd,
    pageGoesOn,
  }: {
    readonly source: string;
    readonly footnoteEnd: number;
    readonly pageGoesOn: boolean;
  },
): ReturnType<typeof separatingEdits> {
  return separatingEdits({
    source,
    readable: source,
    range: {
      start: footnoteEnd,
      end: source.length,
    },
    pageGoesOn,
  },);
}

await describe({
  name: 'the edits that keep a block of the original from what follows it on the page',
  children: [
    describe({
      name: separatingEdits.name,
      children: [
        it({
          name: 'DOUBLES THE LINE ENDING BEFORE A DEFINITION LINE, whether or not the page goes on',
          fn: async () => {
            /**
             The edit that doubles the line feed after the footnote.
             */
            const doubled = [
              {
                start: 7,
                end: 8,
                replacement: '\n\n',
              },
            ];
            expect(editsFor({ source: '[^1]: x\n[r]: u\n', footnoteEnd: 7, pageGoesOn: false, },),).toEqual(doubled,);
            expect(editsFor({ source: '[^1]: x\n[r]: u\n', footnoteEnd: 7, pageGoesOn: true, },),).toEqual(doubled,);
            expect(editsFor({ source: '[^1]: x\n[r]: u\n\npurr\n', footnoteEnd: 7, pageGoesOn: false, },),).toEqual(doubled,);
          },
        },),
        it({
          name: 'DOUBLES A CARRIAGE RETURN AND LINE FEED AS ONE ENDING',
          fn: async () => {
            expect(editsFor({ source: '[^1]: x\r\n[r]: u\r\n', footnoteEnd: 7, pageGoesOn: false, },),).toEqual([
              {
                start: 7,
                end: 9,
                replacement: '\r\n\r\n',
              },
            ],);
          },
        },),
        it({
          name: 'DOUBLES THE LINE ENDING BEFORE A PARAGRAPH ONLY WHERE THE PAGE GOES ON with the footnote',
          fn: async () => {
            expect(editsFor({ source: '[^1]: x\ncat\n', footnoteEnd: 7, pageGoesOn: false, },),).toEqual([],);
            expect(editsFor({ source: '[^1]: x\ncat\n', footnoteEnd: 7, pageGoesOn: true, },),).toEqual([
              {
                start: 7,
                end: 8,
                replacement: '\n\n',
              },
            ],);
          },
        },),
        it({
          name: 'LEAVES A BLOCK THAT IS NEITHER A DEFINITION NOR A PARAGRAPH, such as a heading, where the page does not go on',
          fn: async () => {
            expect(editsFor({ source: '[^1]: x\n# cat\n', footnoteEnd: 7, pageGoesOn: false, },),).toEqual([],);
          },
        },),
        it({
          name: 'DOUBLES THE LINE ENDING BEFORE ANY BLOCK WHERE THE PAGE GOES ON, a heading and a table too',
          fn: async () => {
            /**
             The edit that doubles the line feed after the footnote.
             */
            const doubled = [
              {
                start: 7,
                end: 8,
                replacement: '\n\n',
              },
            ];
            expect(editsFor({ source: '[^1]: x\n# cat\n', footnoteEnd: 7, pageGoesOn: true, },),).toEqual(doubled,);
            expect(editsFor({ source: '[^1]: x\n| a |\n| - |\n', footnoteEnd: 7, pageGoesOn: true, },),).toEqual(doubled,);
          },
        },),
        it({
          name: 'LEAVES A BLANK LINE THAT STANDS THERE ALREADY, and a footnote that ends with the text',
          fn: async () => {
            expect(editsFor({ source: '[^1]: x\n\n[r]: u\n', footnoteEnd: 7, pageGoesOn: true, },),).toEqual([],);
            expect(editsFor({ source: '[^1]: x\n \t\n[r]: u\n', footnoteEnd: 7, pageGoesOn: true, },),).toEqual([],);
            expect(editsFor({ source: '[^1]: x', footnoteEnd: 7, pageGoesOn: true, },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: afterLiteralBlockEdits.name,
      children: [
        it({
          name: 'DOUBLES THE LINE ENDING AFTER AN HTML BLOCK WHEN THE NEXT LINE HOLDS TEXT',
          fn: async () => {
            expect(afterLiteralBlockEdits({ readable: '<?x?>\n[r]: u\n', range: { start: 0, end: 5, }, },),).toEqual([
              {
                start: 5,
                end: 6,
                replacement: '\n\n',
              },
            ],);
          },
        },),
        it({
          name: 'DOUBLES A CARRIAGE RETURN AND LINE FEED AS ONE ENDING',
          fn: async () => {
            expect(afterLiteralBlockEdits({ readable: '<?x?>\r\ncat\r\n', range: { start: 0, end: 5, }, },),).toEqual([
              {
                start: 5,
                end: 7,
                replacement: '\r\n\r\n',
              },
            ],);
          },
        },),
        it({
          name: 'LEAVES A BLANK LINE THAT STANDS THERE ALREADY, and the end of the text',
          fn: async () => {
            expect(afterLiteralBlockEdits({ readable: '<?x?>\n\ncat\n', range: { start: 0, end: 5, }, },),).toEqual([],);
            expect(afterLiteralBlockEdits({ readable: '<?x?>\n \t\ncat\n', range: { start: 0, end: 5, }, },),).toEqual([],);
            expect(afterLiteralBlockEdits({ readable: '<?x?>\n', range: { start: 0, end: 5, }, },),).toEqual([],);
            expect(afterLiteralBlockEdits({ readable: '<?x?>', range: { start: 0, end: 5, }, },),).toEqual([],);
          },
        },),
        it({
          name: 'PUTS A BLANK LINE WITH THE MARKS OF THE CONTAINER AFTER A BLOCK IN A BLOCK QUOTE OR A LIST ITEM',
          fn: async () => {
            expect(afterLiteralBlockEdits({ readable: '> <?x?>\n> [r]: u\n', range: { start: 2, end: 7, }, inContainer: true, },),)
              .toEqual([{ start: 7, end: 8, replacement: '\n>\n', },],);
            expect(afterLiteralBlockEdits({ readable: '> > <?x?>\n> > [r]: u\n', range: { start: 4, end: 9, }, inContainer: true, },),)
              .toEqual([{ start: 9, end: 10, replacement: '\n> >\n', },],);
            expect(afterLiteralBlockEdits({ readable: '- <?x?>\n  [r]: u\n', range: { start: 2, end: 7, }, inContainer: true, },),)
              .toEqual([{ start: 7, end: 8, replacement: '\n\n', },],);
            expect(afterLiteralBlockEdits({ readable: '> <?x?>\n[r]: u\n', range: { start: 2, end: 7, }, inContainer: true, },),)
              .toEqual([{ start: 7, end: 8, replacement: '\n\n', },],);
          },
        },),
        it({
          name: 'LEAVES A LINE OF ONLY MARKS AFTER A BLOCK IN A CONTAINER, which is a blank line of the container already',
          fn: async () => {
            expect(afterLiteralBlockEdits({ readable: '> <?x?>\n>\n> [r]: u\n', range: { start: 2, end: 7, }, inContainer: true, },),)
              .toEqual([],);
            expect(afterLiteralBlockEdits({ readable: '> <?x?>\n> \n> [r]: u\n', range: { start: 2, end: 7, }, inContainer: true, },),)
              .toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
