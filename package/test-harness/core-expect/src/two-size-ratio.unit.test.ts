/** Two-size timing measurement and the linear-growth assertion. @module */
import { describe, expect as verify, it, } from '@monochromatic-dev/test-harness-core/ts';
import {
  expectLinearTimeGrowth,
  measureTwoSizeRatio,
  TwoSizeRatioError,
  TwoSizeRatioInputError,
} from '@monochromatic-dev/test-harness-core-expect';

await describe({
  name: 'two-size ratio',
  children: [
    it({
      name: 'measures best and worst rounds at both sizes',
      fn: async () => {
        const measurement = measureTwoSizeRatio({
          grow: (size,) => 'a'.repeat(size,),
          run: (text,) => {
            for (const char of text) void char;
          },
          size: 20_000,
        },);
        verify(measurement.base.size,).toEqual(20_000,);
        verify(measurement.doubled.size,).toEqual(40_000,);
        verify(measurement.base.worstMs >= measurement.base.bestMs,).toBe(true,);
        verify(measurement.doubled.worstMs >= measurement.doubled.bestMs,).toBe(true,);
        verify(measurement.ratio > 0,).toBe(true,);
      },
    },),
    it({
      name: 'passes one-pass work whose time doubles with the input',
      fn: async () => {
        const measurement = expectLinearTimeGrowth({
          grow: (size,) => 'a'.repeat(size,),
          run: (text,) => {
            let seen = 0;
            for (const char of text) seen += char.length;
            return seen;
          },
          size: 50_000,
        },);
        verify(measurement.ratio < 3,).toBe(true,);
      },
    },),
    it({
      name: 'throws on quadratic work and carries the measurement',
      fn: async () => {
        let thrown: unknown;
        try {
          expectLinearTimeGrowth({
            grow: (size,) => 'a'.repeat(size,),
            run: (text,) => {
              let ops = 0;
              for (const outer of text) {
                for (const inner of text) {
                  ops += outer.length + inner.length;
                }
              }
              return ops;
            },
            size: 2_000,
          },);
        } catch (error) {
          thrown = error;
        }
        verify(thrown instanceof TwoSizeRatioError,).toBe(true,);
        const ratioError = thrown as TwoSizeRatioError;
        verify(ratioError.name,).toEqual('TwoSizeRatioError',);
        verify(ratioError.measurement.ratio > ratioError.maxRatio,).toBe(true,);
        verify(ratioError.message.includes('positive control',),).toBe(true,);
      },
    },),
    it({
      name: 'warms up at the doubled size and discards those rounds',
      fn: async () => {
        const calls: number[] = [];
        measureTwoSizeRatio({
          grow: (size,) => 'a'.repeat(size,),
          run: (text,) => {
            calls.push(text.length,);
            let seen = 0;
            for (const char of text) seen += char.length;
            return seen;
          },
          size: 50_000,
          repeats: 2,
        },);
        verify(calls,).toEqual([100_000, 100_000, 50_000, 100_000, 50_000, 100_000,],);
      },
    },),
    it({
      name: 'respects a caller-supplied max ratio',
      fn: async () => {
        let thrown: unknown;
        try {
          expectLinearTimeGrowth({
            grow: (size,) => 'a'.repeat(size,),
            run: (text,) => {
              let ops = 0;
              for (const outer of text) {
                for (const inner of text) {
                  ops += outer.length + inner.length;
                }
              }
              return ops;
            },
            size: 2_000,
            maxRatio: 1.01,
          },);
        } catch (error) {
          thrown = error;
        }
        verify(thrown instanceof TwoSizeRatioError,).toBe(true,);
        verify((thrown as TwoSizeRatioError).maxRatio,).toEqual(1.01,);
      },
    },),
    it({
      name: 'rejects a base timing below the readable floor instead of flooring it',
      fn: async () => {
        let thrown: unknown;
        try {
          measureTwoSizeRatio({
            grow: (size,) => size,
            run: () => undefined,
            size: 1,
            repeats: 1,
          },);
        } catch (error) {
          thrown = error;
        }
        verify(thrown instanceof TwoSizeRatioInputError,).toBe(true,);
      },
    },),
    it({
      name: 'rejects zero rounds, which would pass vacuously on a NaN ratio',
      fn: async () => {
        let thrown: unknown;
        try {
          expectLinearTimeGrowth({
            grow: (size,) => 'a'.repeat(size,),
            run: (text,) => text.length,
            repeats: 0,
          },);
        } catch (error) {
          thrown = error;
        }
        verify(thrown instanceof TwoSizeRatioInputError,).toBe(true,);
      },
    },),
    it({
      name: 'rejects a thenable run, whose work outlives the timing call',
      fn: async () => {
        let thrown: unknown;
        try {
          measureTwoSizeRatio({
            grow: (size,) => 'a'.repeat(size,),
            run: async (text,) => text.length,
            size: 64,
          },);
        } catch (error) {
          thrown = error;
        }
        verify(thrown instanceof TwoSizeRatioInputError,).toBe(true,);
      },
    },),
    it({
      name: 'rejects a NaN bound, whose comparison would pass anything',
      fn: async () => {
        let thrown: unknown;
        try {
          expectLinearTimeGrowth({
            grow: (size,) => 'a'.repeat(size,),
            run: (text,) => text.length,
            size: 64,
            maxRatio: Number.NaN,
          },);
        } catch (error) {
          thrown = error;
        }
        verify(thrown instanceof TwoSizeRatioInputError,).toBe(true,);
      },
    },),
  ],
},);
