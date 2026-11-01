/**
 Tests for the computed line-structure predicate.

 Its thresholds were chosen from a corpus measurement rather than by taste,
 and they are measured on the SOURCE side. That is load-bearing: one entry's
 Chinese verse has a median node length of 22 and the English rendering of the
 same chunk has 99, so the same predicate reading the translation would never
 fire. The original's shape is what a repair must preserve.

 The case that fixes the thresholds is real: that entry's verse has a median block
 length of 22 while its prose chunks sit at 49 and 87. A threshold of 20 would
 have missed the verse, which is why the boundary is tested here directly.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildEditorAddendum,
  governedSliceIndices,
  isLineStructured,
} from '../dist/final/node/index.mjs';

/**
 Builds a slice from blocks.

 @param blocks - blank-line-separated blocks

 @returns Slice text

 @example
 ```ts
 const text = slice({ blocks: ['a', 'b',], },);
 ```
 */
function slice(
  {
    blocks,
  }: {
    readonly blocks: readonly string[];
  },
): string {
  return blocks.join('\n\n',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isLineStructured.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'recognizes verse: many short blocks, each a unit rather than a '
            + 'paragraph, which is the shape whose editor rewrote three lines into '
            + 'invented ones',
          fn: async () => {
            expect(isLineStructured({
              text: slice({
                blocks: [
                  '月光细长，铺满窗台；',
                  '风铃叮当，唤醒屋檐；',
                  '顽皮的猫跃上屋脊；',
                  '像夜里的影子，轻轻落下；',
                  '直到天色发白，才肯回家。',
                ],
              },),
            },),).toBe(true,);
          },
        },),

        it({
          name: 'recognizes the same verse written with CRLF endings, which is how the one such page in '
            + 'the pinned corpus is written: a splitter looking for two line feeds found no boundary in '
            + 'it, counted one block, and answered false, so no addendum, inheritance or line-count '
            + 'guard ever reached that page',
          fn: async () => {
            /**
             Verse as an LF page carries it.
             */
            const verse = slice({
              blocks: [
                '猫猫在窗台上',
                '看着外面的鸟',
                '尾巴轻轻摇',
                '阳光洒下来',
                '它闭上眼睛',
                '睡着了',
              ],
            },);
            // POSITIVE CONTROL: the LF form is line-structured, or the CRLF
            // assertion would be checking nothing.
            expect(isLineStructured({ text: verse, },),).toBe(true,);
            expect(isLineStructured({ text: verse.replaceAll('\n', '\r\n',), },),).toBe(true,);
          },
        },),
        it({
          name: 'does NOT fire on prose, however many paragraphs it has, since a '
            + 'paragraph is not a unit the editor must preserve line for line',
          fn: async () => {
            expect(isLineStructured({
              text: slice({
                blocks: Array.from(
                  { length: 8, },
                  function paragraph(): string {
                    return 'Mittens spent the whole afternoon on the windowsill, '
                      + 'watching the birds come and go and declining every '
                      + 'invitation to move somewhere less sunlit.';
                  },
                ),
              },),
            },),).toBe(false,);
          },
        },),

        it({
          name: 'refuses to judge a slice under five blocks, because a stanza and a '
            + 'couple of short paragraphs are indistinguishable at that size',
          fn: async () => {
            expect(isLineStructured({
              text: slice({ blocks: ['One.', 'Two.', 'Three.', 'Four.',], },),
            },),).toBe(false,);
          },
        },),

        it({
          name: 'admits a median of 22, the measured value of the verse this exists '
            + 'for, and REFUSES a median of 31. A threshold of 20 would have missed '
            + 'the real case, so the boundary is pinned rather than left implicit',
          fn: async () => {
            /**
             Five blocks of exactly 22 characters.
             */
            const atTwentyTwo = slice({
              blocks: Array.from(
                { length: 5, },
                function line(): string {
                  return 'x'.repeat(22,);
                },
              ),
            },);

            /**
             Five blocks of exactly 31 characters, one past the limit.
             */
            const atThirtyOne = slice({
              blocks: Array.from(
                { length: 5, },
                function line(): string {
                  return 'x'.repeat(31,);
                },
              ),
            },);

            expect(isLineStructured({ text: atTwentyTwo, },),).toBe(true,);
            expect(isLineStructured({ text: atThirtyOne, },),).toBe(false,);
          },
        },),

        it({
          name: 'reads a chat transcript as line-structured too, which is correct '
            + 'rather than incidental: each message is a unit, so one output line '
            + 'per input line is the right instruction there as well',
          fn: async () => {
            expect(isLineStructured({
              text: slice({
                blocks: [
                  '> Tried the new food today?',
                  '> Not really my thing.',
                  '> The sunny spot is better.',
                  '> Agreed, entirely.',
                  '> See you tomorrow!',
                ],
              },),
            },),).toBe(true,);
          },
        },),

        it({
          name: 'DOES NOT CALL A LOOSE LIST LINE-STRUCTURED, five short cat facts '
            + 'written as one list block whatever its items own spacing (ledger B68)',
          fn: async () => {
            expect(isLineStructured({
              text: '- Mittens naps.\n\n- Biscuit watches.\n\n- Patches hunts.\n\n- Tom sleeps.\n\n- Jerry yawns.\n',
            },),).toBe(false,);
          },
        },),

        it({
          name: 'DOES NOT CALL A SLICE THE GRAMMAR REFUSES LINE-STRUCTURED, though six short paragraphs '
            + 'follow its unclosed component, since the floor reads no blocks there (ledger B68)',
          fn: async () => {
            expect(isLineStructured({
              text: '<Cat unclosed\n\nA nap.\n\nA stretch.\n\nA meal.\n\nA purr.\n\nA nap again.\n',
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: buildEditorAddendum.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'speaks about the ORIGINAL, not the current text, because the '
            + 'predicate reads the source and the two disagree on exactly the case '
            + 'this exists for: one entry has 21 source blocks at median 22 against '
            + '18 target blocks at median 101, so an addendum asserting the CURRENT '
            + 'text is line-structured told the editor something untrue about the '
            + 'text in front of it',
          fn: async () => {
            /** Addendum the editor is handed for a governed slice. */
            const addendum = buildEditorAddendum({
              baseAddendum: '',
              lineStructured: true,
            },);

            expect(addendum.includes('ORIGINAL IS line-structured',),).toBe(true,);
            expect(addendum.includes('CURRENT TEXT IS line-structured',),).toBe(false,);
          },
        },),

        it({
          name: 'forbids the failure that was actually observed, not only reflow. '
            + 'The editor replaced three correct lines of one entry with invented text, '
            + 'one carrying a correct translation of a DIFFERENT line, and a rule '
            + 'that only preserved line counts would have permitted every one',
          fn: async () => {
            /** Rule text handed to a governed slice. */
            const addendum = buildEditorAddendum({
              baseAddendum: '',
              lineStructured: true,
            },);

            expect(addendum.includes('never invent a line',),).toBe(true,);
            expect(addendum.includes('content belonging to another',),).toBe(true,);
          },
        },),

        it({
          name: 'adds nothing for ordinary prose, so the rule reaches only the '
            + 'slices its chunk governs and cannot quietly govern the corpus',
          fn: async () => {
            expect(buildEditorAddendum({
              baseAddendum: '',
              lineStructured: false,
            },),).toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: governedSliceIndices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'governs EVERY slice carved from a line-structured chunk, including '
            + 'slices too small for the predicate to judge on their own. One entry '
            + 'measured the cost of the alternative: its verse chunk trips at 21 '
            + 'blocks, median 22, subdivides into seven slices, and only one of the '
            + 'seven still trips, so deciding per slice dropped the rule on six '
            + 'sevenths of the verse it exists for',
          fn: async () => {
            /**
             A verse chunk whole, as the aligner produces it.
             */
            const verseChunk = slice({
              blocks: [
                'Paws on the windowsill',
                'A tail curled tight',
                'Rain against the glass',
                'The kettle starts to sing',
                'Nobody comes home',
                'The cushion keeps its shape',
              ],
            },);

            /**
             Two of its slices; the second is a two-block fragment the predicate
             refuses to judge, which is the case that regressed.
             */
            const governed = governedSliceIndices({
              chunks: [{
                sourceText: verseChunk,
                slices: [
                  {
                    index: 4,
                    sourceText: verseChunk,
                  },
                  {
                    index: 5,
                    sourceText: 'Paws on the windowsill',
                  },
                ],
              },],
            },);

            expect(governed.has(4,),).toBe(true,);
            expect(governed.has(5,),).toBe(true,);
          },
        },),

        it({
          name: 'asks the predicate about the WHOLE chunk, so a fragment that would '
            + 'answer false alone still inherits true, proving the verdict is not '
            + 'silently recomputed per slice',
          fn: async () => {
            /**
             Two blocks: under the five-block floor, so judged alone this is false.
             */
            const fragment = slice({
              blocks: [
                'Paws on the windowsill',
                'A tail curled tight',
              ],
            },);

            expect(isLineStructured({ text: fragment, },),).toBe(false,);

            expect(
              governedSliceIndices({
                chunks: [{
                  sourceText: slice({
                    blocks: [
                      'Paws on the windowsill',
                      'A tail curled tight',
                      'Rain against the glass',
                      'The kettle starts to sing',
                      'Nobody comes home',
                    ],
                  },),
                  slices: [{
                    index: 9,
                    sourceText: fragment,
                  },],
                },],
              },).has(9,),
            ).toBe(true,);
          },
        },),

        it({
          name: 'governs nothing carved from a prose chunk, so inheritance widens '
            + 'reach only where the chunk earned it',
          fn: async () => {
            expect(
              governedSliceIndices({
                chunks: [{
                  sourceText: slice({
                    blocks: [
                      'The cat considered the windowsill at some length, weighing the sun against the draught.',
                      'Having decided, she then reconsidered, which is the privilege of cats and of committees.',
                      'The kettle boiled unattended, as kettles will when nobody in the house has hands.',
                      'By evening the cushion had taken her shape and refused, politely, to give it back.',
                      'This is the whole of the afternoon, and it was enough for everyone concerned.',
                    ],
                  },),
                  slices: [
                    {
                      index: 0,
                      sourceText: 'The kettle boiled unattended.',
                    },
                    {
                      index: 1,
                      sourceText: 'By evening the cushion had taken her shape.',
                    },
                    {
                      index: 2,
                      sourceText: 'This is the whole of the afternoon.',
                    },
                  ],
                },],
              },).size,
            ).toBe(0,);
          },
        },),

        it({
          name: 'governs a slice whose OWN original is line-structured even when its '
            + 'enclosing chunk is not, which is a stanza sitting inside a section '
            + 'whose prose dominates the chunk median. Chunk-only governance loses '
            + 'these: measured across the 92 entries at the pinned corpus commit it '
            + 'covers 195 slices against slice-only 55, yet four entries go '
            + 'BACKWARDS, one from 5 to 1 and three others from 1 to 0. The '
            + 'union cannot lose to either reading',
          fn: async () => {
            /**
             A chunk dominated by long prose blocks, so the chunk does not trip.
             */
            const proseHeavy = slice({
              blocks: [
                'The cat considered the windowsill at some length, weighing the sun against the draught.',
                'Having decided, she then reconsidered, which is the privilege of cats and of committees.',
                'The kettle boiled unattended, as kettles will when nobody in the house has hands.',
                'By evening the cushion had taken her shape and refused, politely, to give it back.',
                'This is the whole of the afternoon, and it was enough for everyone concerned.',
                'Paws on the windowsill',
                'A tail curled tight',
                'Rain against the glass',
                'The kettle starts to sing',
                'Nobody comes home',
              ],
            },);

            /**
             The stanza alone, which DOES trip on its own.
             */
            const stanza = slice({
              blocks: [
                'Paws on the windowsill',
                'A tail curled tight',
                'Rain against the glass',
                'The kettle starts to sing',
                'Nobody comes home',
              ],
            },);

            expect(isLineStructured({ text: proseHeavy, },),).toBe(false,);
            expect(isLineStructured({ text: stanza, },),).toBe(true,);

            expect(
              governedSliceIndices({
                chunks: [{
                  sourceText: proseHeavy,
                  slices: [{
                    index: 7,
                    sourceText: stanza,
                  },],
                },],
              },).has(7,),
            ).toBe(true,);
          },
        },),

        it({
          name: 'keeps chunks independent, so one verse section does not govern the '
            + 'prose sections beside it',
          fn: async () => {
            /**
             One verse chunk and one prose chunk, as a mixed entry produces.
             */
            const governed = governedSliceIndices({
              chunks: [
                {
                  sourceText: slice({
                    blocks: [
                      'Paws on the windowsill',
                      'A tail curled tight',
                      'Rain against the glass',
                      'The kettle starts to sing',
                      'Nobody comes home',
                    ],
                  },),
                  slices: [
                    {
                      index: 0,
                      sourceText: 'Paws on the windowsill',
                    },
                    {
                      index: 1,
                      sourceText: 'A tail curled tight',
                    },
                  ],
                },
                {
                  sourceText: slice({
                    blocks: [
                      'The cat considered the windowsill at some length, weighing the sun against the draught.',
                      'Having decided, she then reconsidered, which is the privilege of cats and of committees.',
                      'The kettle boiled unattended, as kettles will when nobody in the house has hands.',
                      'By evening the cushion had taken her shape and refused, politely, to give it back.',
                      'This is the whole of the afternoon, and it was enough for everyone concerned.',
                    ],
                  },),
                  slices: [
                    {
                      index: 2,
                      sourceText: 'The kettle boiled unattended, as kettles will.',
                    },
                    {
                      index: 3,
                      sourceText: 'By evening the cushion had taken her shape.',
                    },
                  ],
                },
              ],
            },);

            expect([...governed,].toSorted(function ascending(left, right,) {
              return left - right;
            },),).toEqual([0, 1,],);
          },
        },),
      ],
    },),
  ],
},);
