/**
 Tests the readings of one line of straight single marks (n159): a closing mark pairs with the innermost opening mark that waits,
 a listed elision never waits behind another opening mark, a quoted number leaves two readings, and the reading with the most
 pairs wins, a tie keeping the first (the elision).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bestPairsOf,
  distinctDepths,
  EMPTY_LINE,
  withMark,
  withQuotedNumber,
} from '../dist/final/node/index.mjs';

/**
 The reading of a line after one opening mark at the start.
 */
const OPENED = withMark({
  state: EMPTY_LINE,
  at: 0,
  shape: 'opening',
  elision: false,
},);

await describe({
  name: 'nested-single-line-readings (n159)',
  children: [
    describe({
      name: withMark.name,
      children: [
        it({
          name: 'WAITS AT AN OPENING MARK AND PAIRS A CLOSING MARK WITH THE INNERMOST ONE THAT WAITS',
          fn: async () => {
            /**
             The line after two opening marks.
             */
            const twoOpened = withMark({
              state: OPENED,
              at: 4,
              shape: 'opening',
              elision: false,
            },);
            expect(twoOpened.waiting,).toEqual([0, 4,],);
            expect(withMark({
              state: twoOpened,
              at: 9,
              shape: 'closing',
              elision: false,
            },),).toEqual({
              waiting: [0,],
              paired: [{ open: 4, close: 9, },],
            },);
          },
        },),
        it({
          name: 'NEVER LETS A LISTED ELISION WAIT BEHIND ANOTHER OPENING MARK, but lets it open on an empty line',
          fn: async () => {
            expect(withMark({
              state: OPENED,
              at: 5,
              shape: 'opening',
              elision: true,
            },),).toEqual(OPENED,);
            expect(withMark({
              state: EMPTY_LINE,
              at: 5,
              shape: 'opening',
              elision: true,
            },).waiting,).toEqual([5,],);
          },
        },),
        it({
          name: 'LEAVES THE LINE AS IT IS FOR A CLOSING MARK WITH NONE WAITING AND FOR A MARK THAT IS NEITHER',
          fn: async () => {
            expect(withMark({
              state: EMPTY_LINE,
              at: 3,
              shape: 'closing',
              elision: false,
            },),).toEqual(EMPTY_LINE,);
            expect(withMark({
              state: EMPTY_LINE,
              at: 3,
              shape: 'neither',
              elision: false,
            },),).toEqual(EMPTY_LINE,);
          },
        },),
      ],
    },),
    describe({
      name: withQuotedNumber.name,
      children: [
        it({
          name: 'KEEPS TWO READINGS WHEN A MARK WAITS, the elision first, and one when none waits',
          fn: async () => {
            expect(withQuotedNumber({
              states: [OPENED,],
              open: 5,
              close: 8,
              closeShape: 'closing',
            },),).toEqual([
              {
                waiting: [],
                paired: [{ open: 0, close: 8, },],
              },
              {
                waiting: [0,],
                paired: [{ open: 5, close: 8, },],
              },
            ],);
            expect(withQuotedNumber({
              states: [EMPTY_LINE,],
              open: 5,
              close: 8,
              closeShape: 'closing',
            },),).toEqual([
              {
                waiting: [],
                paired: [{ open: 5, close: 8, },],
              },
            ],);
          },
        },),
        it({
          name: 'KEEPS THE OPENING MARK WAITING, the number its own quotation, WHEN THE MARK AFTER THE NUMBER IS NO CLOSING MARK',
          fn: async () => {
            expect(withQuotedNumber({
              states: [OPENED,],
              open: 5,
              close: 8,
              closeShape: 'neither',
            },),).toEqual([
              {
                waiting: [0,],
                paired: [{ open: 5, close: 8, },],
              },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: distinctDepths.name,
      children: [
        it({
          name: 'KEEPS THE READING WITH THE MOST PAIRS FOR EACH NUMBER OF WAITING MARKS, the first of equal ones',
          fn: async () => {
            /**
             A reading with one pair and nothing waiting.
             */
            const paired = {
              waiting: [],
              paired: [{ open: 1, close: 2, },],
            };
            expect(distinctDepths({
              states: [
                EMPTY_LINE,
                paired,
                {
                  ...paired,
                  paired: [{ open: 3, close: 4, },],
                },
              ],
            },),).toEqual([paired,],);
          },
        },),
      ],
    },),
    describe({
      name: bestPairsOf.name,
      children: [
        it({
          name: 'TAKES THE PAIRS OF THE READING WITH THE MOST PAIRS, and of equal ones the first, which keeps an elision',
          fn: async () => {
            expect(bestPairsOf({
              states: [
                {
                  waiting: [],
                  paired: [{ open: 0, close: 1, },],
                },
                {
                  waiting: [],
                  paired: [{ open: 2, close: 3, },],
                },
              ],
            },),).toEqual([{ open: 0, close: 1, },],);
            expect(bestPairsOf({
              states: [
                {
                  waiting: [],
                  paired: [{ open: 0, close: 1, },],
                },
                {
                  waiting: [],
                  paired: [
                    { open: 2, close: 3, },
                    { open: 4, close: 5, },
                  ],
                },
              ],
            },),).toEqual([
              { open: 2, close: 3, },
              { open: 4, close: 5, },
            ],);
          },
        },),
      ],
    },),
  ],
},);
