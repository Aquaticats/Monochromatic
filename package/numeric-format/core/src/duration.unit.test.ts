/**
 Tests for `formatDuration`.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { formatDuration, } from '@monochromatic-dev/numeric-format-core';

await describe({
  name: formatDuration.name,
  children: [
    //region Sub-10ms branch: one decimal place

    it({
      name: 'renders 0 with one decimal as ms',
      fn: async ({ expect, },) => {
        expect(formatDuration(0,),).toBe('0.0ms',);
      },
    },),

    it({
      name: 'renders sub-millisecond with one decimal',
      fn: async ({ expect, },) => {
        expect(formatDuration(0.34,),).toBe('0.3ms',);
        expect(formatDuration(0.05,),).toBe('0.1ms',);
      },
    },),

    it({
      name: 'renders single-digit ms with one decimal',
      fn: async ({ expect, },) => {
        expect(formatDuration(1,),).toBe('1.0ms',);
        expect(formatDuration(2.4,),).toBe('2.4ms',);
        expect(formatDuration(9.9,),).toBe('9.9ms',);
      },
    },),

    //endregion Sub-10ms branch: one decimal place

    //region 10-999ms branch: whole ms

    it({
      name: 'renders 10ms at lower boundary as whole ms',
      fn: async ({ expect, },) => {
        expect(formatDuration(10,),).toBe('10ms',);
      },
    },),

    it({
      name: 'renders mid-range as whole ms',
      fn: async ({ expect, },) => {
        expect(formatDuration(51,),).toBe('51ms',);
        expect(formatDuration(123.7,),).toBe('124ms',);
      },
    },),

    it({
      name: 'renders 999ms at upper boundary as whole ms',
      fn: async ({ expect, },) => {
        expect(formatDuration(999,),).toBe('999ms',);
      },
    },),

    //endregion 10-999ms branch: whole ms

    //region 1000ms+ branch: seconds with one decimal

    it({
      name: 'renders 1000ms at boundary as seconds',
      fn: async ({ expect, },) => {
        expect(formatDuration(1_000,),).toBe('1.0s',);
      },
    },),

    it({
      name: 'renders multi-second durations with one decimal',
      fn: async ({ expect, },) => {
        expect(formatDuration(1_234,),).toBe('1.2s',);
        expect(formatDuration(15_300,),).toBe('15.3s',);
      },
    },),
    //endregion 1000ms+ branch: seconds with one decimal
  ],
},);
