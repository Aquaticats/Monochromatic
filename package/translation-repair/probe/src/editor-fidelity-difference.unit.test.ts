/**
 Tests for how the editor fidelity probe finds the defect in a damaged twin: where it and the reference stop agreeing
 from either end, and the whole digit run a differing digit belongs to. Texts are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  differingMiddle,
  digitRunAround,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'editor fidelity difference',
  children: [
    it({
      name: 'FINDS WHERE THE TEXTS STOP AGREEING from the start and from the end, and where the differing middle ends '
        + 'in each',
      fn: async () => {
        expect(differingMiddle({
          cleanText: 'the cat naps today',
          damagedText: 'the cat purrs today',
        },),).toStrictEqual({
          start: 8,
          damagedEnd: 12,
          cleanEnd: 11,
        },);
      },
    },),

    it({
      name: 'NEVER LETS THE SUFFIX OVERLAP THE PREFIX, so a repeated letter does not count twice: the extra letter of '
        + 'the damaged text is the whole middle',
      fn: async () => {
        expect(differingMiddle({
          cleanText: 'aaa',
          damagedText: 'aaaa',
        },),).toStrictEqual({
          start: 3,
          damagedEnd: 4,
          cleanEnd: 3,
        },);
      },
    },),

    it({
      name: 'WIDENS A DIFFERING LAST DIGIT BACK TO THE START OF ITS NUMBER and a differing first digit forward to its '
        + 'end, so a number is read whole and not by the one digit that differs',
      fn: async () => {
        expect(digitRunAround({
          text: 'in 2024 and',
          start: 6,
          end: 7,
        },),).toStrictEqual({
          start: 3,
          grown: 0,
        },);
        expect(digitRunAround({
          text: 'in 2024 and',
          start: 3,
          end: 4,
        },),).toStrictEqual({
          start: 3,
          grown: 3,
        },);
      },
    },),

    it({
      name: 'LEAVES A MIDDLE THAT IS NOT IN A NUMBER AS IT IS',
      fn: async () => {
        expect(digitRunAround({
          text: 'the cat naps',
          start: 4,
          end: 7,
        },),).toStrictEqual({
          start: 4,
          grown: 0,
        },);
      },
    },),
  ],
},);
