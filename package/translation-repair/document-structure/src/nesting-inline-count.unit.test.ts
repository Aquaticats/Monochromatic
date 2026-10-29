/**
 Tests for the count of what a line opens and closes: brackets and emphasis
 delimiters under either grammar, open braces and open tags under the strict
 one, each carried from line to line in one state and read against its bound.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  NO_FENCE,
  scanInline,
  type ScanState,
} from '@monochromatic-dev/translation-repair-markup';
import {
  grammarAcceptsDocument,
  tagNameSpellings,
} from './tag-name-spellings.test-fixture.ts';

//region Nesting inline count tests

/**
 A state that has read nothing.

 @returns Fresh state, its counts all zero

 @example
 ```ts
 const state = freshState();
 ```
 */
function freshState(): ScanState {
  return {
    brackets: 0,
    delimiters: 0,
    tags: 0,
    braces: 0,
    pendingTag: 'none',
    fence: NO_FENCE,
    loose: NO_FENCE,
    html: 'none',
    mathSeen: false,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: scanInline.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS an open bracket and closes it on its closing bracket, under either grammar',
          fn: async () => {
            /**
             State after a line that opens two brackets and closes one.
             */
            const state = freshState();

            expect(scanInline({
              line: '[[cat]',
              start: 0,
              lineNumber: 1,
              grammar: 'markdown',
              state,
              skipDelimiters: false,
            },),).toEqual({ kind: 'within', },);
            expect(state,).toEqual({ ...freshState(), brackets: 1, },);
          },
        },),
        it({
          name: 'SKIPS an escaped character, which opens and closes nothing',
          fn: async () => {
            /**
             State after a line of escaped brackets.
             */
            const state = freshState();

            scanInline({
              line: String.raw`\[\[\*`,
              start: 0,
              lineNumber: 1,
              grammar: 'mdx',
              state,
              skipDelimiters: false,
            },);

            expect(state,).toEqual(freshState(),);
          },
        },),
        it({
          name: 'COUNTS emphasis delimiters unless the line is told its delimiters nest nothing',
          fn: async () => {
            /**
             State after a line read as prose.
             */
            const counted = freshState();
            /**
             State after the same line read as a thematic break.
             */
            const skipped = freshState();

            scanInline({
              line: '*a _b ~~c',
              start: 0,
              lineNumber: 1,
              grammar: 'markdown',
              state: counted,
              skipDelimiters: false,
            },);
            scanInline({
              line: '*a _b ~~c',
              start: 0,
              lineNumber: 1,
              grammar: 'markdown',
              state: skipped,
              skipDelimiters: true,
            },);

            expect(counted.delimiters,).toBe(4,);
            expect(skipped.delimiters,).toBe(0,);
          },
        },),
        it({
          name: 'COUNTS open braces only under the strict grammar',
          fn: async () => {
            /**
             State after the strict grammar reads two braces and a close.
             */
            const mdx = freshState();
            /**
             State after plain markdown reads the same line.
             */
            const markdown = freshState();

            scanInline({
              line: '{{a}',
              start: 0,
              lineNumber: 1,
              grammar: 'mdx',
              state: mdx,
              skipDelimiters: false,
            },);
            scanInline({
              line: '{{a}',
              start: 0,
              lineNumber: 1,
              grammar: 'markdown',
              state: markdown,
              skipDelimiters: false,
            },);

            expect(mdx.braces,).toBe(1,);
            expect(markdown.braces,).toBe(0,);
          },
        },),
        it({
          name: 'COUNTS an open tag when its closing angle arrives, closes it on a closing tag, and counts neither a '
            + 'void element nor a self-closing tag',
          fn: async () => {
            /**
             State after one tag opens and two are written that open nothing.
             */
            const state = freshState();

            scanInline({
              line: '<div>cat<br><Cat name="mew" /><b>x</b>',
              start: 0,
              lineNumber: 1,
              grammar: 'mdx',
              state,
              skipDelimiters: false,
            },);

            expect(state,).toEqual({ ...freshState(), tags: 1, },);
          },
        },),
        it({
          name: 'KEEPS a tag begun and not yet closed pending to the next line',
          fn: async () => {
            /**
             State after a line that begins a tag and leaves it open.
             */
            const state = freshState();

            scanInline({
              line: '<Cat name="mew"',
              start: 0,
              lineNumber: 1,
              grammar: 'mdx',
              state,
              skipDelimiters: false,
            },);
            expect(state.pendingTag,).toBe('opening',);

            scanInline({
              line: '>',
              start: 0,
              lineNumber: 2,
              grammar: 'mdx',
              state,
              skipDelimiters: false,
            },);
            expect(state,).toEqual({ ...freshState(), tags: 1, },);
          },
        },),
        it({
          name: 'OPENS A TAG EXACTLY WHERE THE STRICT GRAMMAR READS ITS NAME, over every spelling of the differential '
            + 'set, so a name starting with `$`, `_` or a letter of another script opens one and a name holding a '
            + 'character the grammar refuses opens none',
          fn: async () => {
            /**
             Spellings on which the count and the grammar disagree, each written as an opening tag its line
             closes.
             */
            const disagreeing = tagNameSpellings()
              .filter(function disagrees(name,): boolean {
                /**
                 State after the line holding the tag.
                 */
                const state = freshState();
                scanInline({
                  line: `<${name} n="1">`,
                  start: 0,
                  lineNumber: 1,
                  grammar: 'mdx',
                  state,
                  skipDelimiters: false,
                },);
                return grammarAcceptsDocument({ document: `<${name} n="1"/>\n`, },) !== (state.tags === 1);
              },);
            expect(disagreeing,).toEqual([],);
          },
        },),
        it({
          name: 'READS A TAG WHOSE NAME OR SEPARATOR REACHES THE END OF ITS LINE as the strict grammar reads it, the '
            + 'line ending stepped over inside the tag: an opener goes on to its bracket on the next line, and a '
            + 'closer begun with `</` at a line\'s end closes on the next',
          fn: async () => {
            /**
             Openers split at a line's end, each closing its angle on its second line.
             */
            const splits: readonly (readonly [string, string,])[] = [
              [
                '<Paw',
                '>',
              ],
              [
                '<Paw.',
                'Cat>',
              ],
              [
                '<Paw:',
                'cat>',
              ],
            ];
            expect(splits.map(function tagsAfter([first, second,],): number {
              /**
               State after both lines.
               */
              const state = freshState();
              for (const [position, line,] of [first, second,].entries()) {
                scanInline({
                  line,
                  start: 0,
                  lineNumber: position + 1,
                  grammar: 'mdx',
                  state,
                  skipDelimiters: false,
                },);
              }
              return state.tags;
            },),).toEqual([
              1,
              1,
              1,
            ],);

            /**
             State holding one open tag, then reading a closer split after its slash.
             */
            const closing = { ...freshState(), tags: 1, };
            for (const [position, line,] of ['</', 'Paw>',].entries()) {
              scanInline({
                line,
                start: 0,
                lineNumber: position + 1,
                grammar: 'mdx',
                state: closing,
                skipDelimiters: false,
              },);
            }
            expect(closing,).toEqual(freshState(),);
          },
        },),
        it({
          name: 'NAMES the place of the item that passes its bound, with the bound of its own count',
          fn: async () => {
            /**
             State with brackets at the bound already.
             */
            const brackets = { ...freshState(), brackets: 256, };
            /**
             State with delimiters at the bound already.
             */
            const delimiters = { ...freshState(), delimiters: 1_024, };

            expect(scanInline({
              line: 'cat [',
              start: 0,
              lineNumber: 7,
              grammar: 'markdown',
              state: brackets,
              skipDelimiters: false,
            },),).toEqual({
              kind: 'beyond',
              measure: 'brackets',
              bound: 256,
              line: 7,
              column: 5,
            },);
            expect(scanInline({
              line: 'cat *',
              start: 0,
              lineNumber: 8,
              grammar: 'markdown',
              state: delimiters,
              skipDelimiters: false,
            },),).toEqual({
              kind: 'beyond',
              measure: 'emphasis delimiters',
              bound: 1_024,
              line: 8,
              column: 5,
            },);
          },
        },),
        it({
          name: 'STARTS reading at the position it is given, after the markers of the line',
          fn: async () => {
            /**
             State after a line whose first characters are skipped.
             */
            const state = freshState();

            scanInline({
              line: '[[ [',
              start: 2,
              lineNumber: 1,
              grammar: 'markdown',
              state,
              skipDelimiters: false,
            },);

            expect(state.brackets,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Nesting inline count tests
