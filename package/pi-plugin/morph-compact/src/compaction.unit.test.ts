import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { chooseCompressionRatio, } from './compaction.ts';

/** Context window size used in ratio tests. */
const CONTEXT_WINDOW = 100_000;

await describe({
  name: '',
  children: [
    describe({
      name: chooseCompressionRatio.name,
      children: [
        it({
          name: 'returns critical ratio above 80% usage',
          fn: async () => {
            const ratio = chooseCompressionRatio({
              tokens: 90_000,
              contextWindow: CONTEXT_WINDOW,
            },);
            expect(ratio,).toBe(0.3,);
          },
        },),
        it({
          name: 'returns high ratio above 60% usage',
          fn: async () => {
            const ratio = chooseCompressionRatio({
              tokens: 70_000,
              contextWindow: CONTEXT_WINDOW,
            },);
            expect(ratio,).toBe(0.4,);
          },
        },),
        it({
          name: 'returns moderate ratio below 60% usage',
          fn: async () => {
            const ratio = chooseCompressionRatio({
              tokens: 30_000,
              contextWindow: CONTEXT_WINDOW,
            },);
            expect(ratio,).toBe(0.5,);
          },
        },),
        it({
          name: 'defaults to high ratio when context usage is undefined',
          fn: async () => {
            const ratio = chooseCompressionRatio(undefined,);
            expect(ratio,).toBe(0.4,);
          },
        },),
        it({
          name: 'defaults to high ratio when tokens are null',
          fn: async () => {
            const ratio = chooseCompressionRatio({
              tokens: null,
              contextWindow: CONTEXT_WINDOW,
            },);
            expect(ratio,).toBe(0.4,);
          },
        },),
        it({
          name: 'uses critical ratio at 80% boundary',
          fn: async () => {
            const ratio = chooseCompressionRatio({
              tokens: 80_001,
              contextWindow: CONTEXT_WINDOW,
            },);
            expect(ratio,).toBe(0.3,);
          },
        },),
        it({
          name: 'uses high ratio at 60% boundary',
          fn: async () => {
            const ratio = chooseCompressionRatio({
              tokens: 60_001,
              contextWindow: CONTEXT_WINDOW,
            },);
            expect(ratio,).toBe(0.4,);
          },
        },),
      ],
    },),
  ],
},);
