/**
 Tests for where the lines of the block of raw HTML a footnote definition opens with stand, and what a block among them
 comes to for the page.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { footnoteDefinitionRangesOf, } from '@monochromatic-dev/translation-repair-document-structure/ts';

import {
  rawHtmlBlockReading,
  rawHtmlRegionsOf,
} from '../../dist/final/node/index.mjs';

/**
 The regions of the text, with the footnotes read as a reader of the source reads them.

 @param source - text

 @returns Where the lines of each block of raw HTML a footnote opens with stand

 @example
 ```ts
 const regions = regionsOf({ source: '[^1]: <b>\n    x\n', },);
 ```
 */
function regionsOf({ source, }: { readonly source: string; },): ReturnType<typeof rawHtmlRegionsOf> {
  return rawHtmlRegionsOf({
    source,
    footnotes: footnoteDefinitionRangesOf({ body: source, },),
  },);
}

await describe({
  name: 'the lines of the block of raw HTML a footnote opens with',
  children: [
    describe({
      name: rawHtmlRegionsOf.name,
      children: [
        it({
          name: 'STARTS AT THE LINE AFTER THE DEFINITION AND ENDS WITH THE FOOTNOTE',
          fn: async () => {
            expect(regionsOf({ source: '- [^1]: <b>\n      ```\n      ```\n', },),).toEqual([
              {
                start: 12,
                end: 31,
                prefix: '- ',
              },
            ],);
          },
        },),

        it({
          name: 'ENDS AT THE FIRST BLANK LINE, a line that holds only the marks of its container among them',
          fn: async () => {
            expect(regionsOf({ source: '[^1]: <b>\n    x\n\n    y\n', },),).toEqual([
              {
                start: 10,
                end: 16,
                prefix: '',
              },
            ],);
            expect(regionsOf({ source: '> [^1]: <b>\n>     x\n>\n>     y\n', },),).toEqual([
              {
                start: 12,
                end: 20,
                prefix: '> ',
              },
            ],);
          },
        },),

        it({
          name: 'GOES ON OVER A LINE WITHOUT THE MARKS OF ITS BLOCK QUOTE, a lazy line',
          fn: async () => {
            expect(regionsOf({ source: '> [^1]: <b>\n>     x\ny\n', },),).toEqual([
              {
                start: 12,
                end: 19,
                prefix: '> ',
              },
            ],);
          },
        },),

        it({
          name: 'FINDS NONE FOR A FOOTNOTE THAT DOES NOT OPEN WITH A TAG ALONE OR HAS NO LINE AFTER ITS DEFINITION',
          fn: async () => {
            expect(regionsOf({ source: '- [^1]: x\n      ```\n', },),).toEqual([],);
            expect(regionsOf({ source: '[^1]: <b>', },),).toEqual([],);
          },
        },),

        it({
          name: 'FINDS ONE FOR A FOOTNOTE LISTED TWICE, as it is for a reader of the source and for the page',
          fn: async () => {
            /**
             The text, and the range of its footnote as both readers give it.
             */
            const source = '[^1]: <b>\n    x\n';
            const range = {
              start: 0,
              end: 15,
            };
            expect(rawHtmlRegionsOf({
              source,
              footnotes: [
                range,
                range,
              ],
            },),).toHaveLength(1,);
          },
        },),
      ],
    },),

    describe({
      name: rawHtmlBlockReading.name,
      children: [
        it({
          name: 'READS NOTHING FOR A BLOCK THAT STARTS OUTSIDE EVERY REGION, or stands at the top',
          fn: async () => {
            expect(rawHtmlBlockReading({
              source: 'a\n\nb\n',
              readable: 'a\n\nb\n',
              regions: [],
              range: {
                start: 3,
                end: 4,
              },
              inContainer: true,
            },),).toEqual({
              inRegion: false,
              literal: [],
              text: [],
              edits: [],
              insideLinks: [],
              streamStarts: [],
            },);
            expect(rawHtmlBlockReading({
              source: '[^1]: <b>\n    x\n',
              readable: '[^1]: <b>\n    x\n',
              regions: [
                {
                  start: 10,
                  end: 15,
                  prefix: '',
                },
              ],
              range: {
                start: 14,
                end: 15,
              },
              inContainer: false,
            },).inRegion,).toBe(false,);
          },
        },),

        it({
          name: 'GIVES THE CONTENT OF EACH LINE LESS THE MARKS OF ITS CONTAINER AND A CARRIAGE RETURN',
          fn: async () => {
            /**
             A quote and a footnote whose block of raw HTML goes on over a fence, with Windows line endings.
             */
            const source = '> [^1]: <b>\r\n>     ```\r\n>     ```\r\n';
            expect(rawHtmlBlockReading({
              source,
              readable: source,
              regions: [
                {
                  start: 13,
                  end: 33,
                  prefix: '> ',
                },
              ],
              range: {
                start: 19,
                end: 33,
              },
              inContainer: true,
            },).literal,).toEqual([
              {
                start: 19,
                end: 22,
              },
              {
                start: 26,
                end: 33,
              },
            ],);
          },
        },),

        it({
          name: 'PUTS A BLANK LINE AFTER A BLOCK THAT ENDS THE REGION, and none after one with lines of the region after it',
          fn: async () => {
            /**
             A list item whose footnote opens with a tag alone, a paragraph of two lines, and a definition line.
             */
            const source = '- [^1]: <b>\n      a\n      b\n  [r]: u\n';
            /**
             The region and the block, which ends it.
             */
            const region = {
              start: 12,
              end: 27,
              prefix: '- ',
            };
            expect(rawHtmlBlockReading({
              source,
              readable: source,
              regions: [region,],
              range: {
                start: 18,
                end: 27,
              },
              inContainer: true,
            },).edits,).toEqual([
              {
                start: 27,
                end: 28,
                replacement: '\n\n',
              },
            ],);
            expect(rawHtmlBlockReading({
              source,
              readable: source,
              regions: [region,],
              range: {
                start: 18,
                end: 19,
              },
              inContainer: true,
            },).edits,).toEqual([],);
          },
        },),

        it({
          name: 'READS THE LINES OF A BLOCK PAST THE REGION AS TEXT, with a blank line after the last line in the region',
          fn: async () => {
            /**
             A paragraph the plain reading makes of the fence line and a definition line, of which the region holds the first.
             */
            const source = '- [^1]: <b>\n      ```\n  [r]: u{b}\n';
            expect(rawHtmlBlockReading({
              source,
              readable: source,
              regions: [
                {
                  start: 12,
                  end: 21,
                  prefix: '- ',
                },
              ],
              range: {
                start: 18,
                end: 33,
              },
              inContainer: true,
            },),).toEqual({
              inRegion: true,
              literal: [
                {
                  start: 18,
                  end: 21,
                },
              ],
              text: [
                {
                  start: 22,
                  end: 33,
                },
              ],
              edits: [
                {
                  start: 21,
                  end: 22,
                  replacement: '\n\n',
                },
              ],
              insideLinks: [],
              streamStarts: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
