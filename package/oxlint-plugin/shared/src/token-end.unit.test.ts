import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { tokenEnd, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: tokenEnd.name,
      children: [
        it({
          name: 'stops at the first whitespace after the token',
          fn: async () => {
            expect(tokenEnd({
              text: 'name - desc',
              start: 0,
            },),).toBe(4,);
          },
        },),
        it({
          name: 'returns the length when the text carries no whitespace',
          fn: async () => {
            expect(tokenEnd({
              text: 'name',
              start: 0,
            },),).toBe(4,);
          },
        },),
        it({
          name: 'returns the start when the text starts with whitespace',
          fn: async () => {
            expect(tokenEnd({
              text: ' name',
              start: 0,
            },),).toBe(0,);
          },
        },),
        it({
          name: 'scans from the start index past earlier whitespace',
          fn: async () => {
            // Positive control for the start parameter: the same text answers
            // 0 from index 0, so only honouring `start` reaches the end.
            expect(tokenEnd({
              text: ' name',
              start: 0,
            },),).toBe(0,);
            expect(tokenEnd({
              text: ' name',
              start: 1,
            },),).toBe(5,);
          },
        },),
        it({
          name: 'answers past the end of the text',
          fn: async () => {
            expect(tokenEnd({
              text: 'name',
              start: 9,
            },),).toBe(4,);
          },
        },),
        it({
          name: 'answers zero for empty text',
          fn: async () => {
            expect(tokenEnd({
              text: '',
              start: 0,
            },),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
