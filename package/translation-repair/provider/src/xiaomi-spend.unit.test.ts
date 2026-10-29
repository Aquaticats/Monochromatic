/**
 Tests for the reader of what this run has spent on Xiaomi. The reading is the process-wide run meter and nothing else.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  noteRunSpend,
  resetRunSpend,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { xiaomiSpendNow, } from '../dist/final/node/index.mjs';

await describe({
  name: xiaomiSpendNow.name,
  // SEQUENTIAL: the run spend meter is process-wide, so cases that write it must not overlap.
  concurrency: 1,
  children: [
    it({
      name: 'READS zero on a fresh meter and exactly what the run meter holds for Xiaomi after spend is noted',
      fn: async () => {
        resetRunSpend();
        expect(xiaomiSpendNow().spentUsd,).toBe(0,);
        noteRunSpend({ provider: 'xiaomi', costUsd: 0.0042, },);
        expect(xiaomiSpendNow().spentUsd,).toBe(runSpendUsd({ provider: 'xiaomi', },),);
        expect(xiaomiSpendNow().spentUsd,).toBeCloseTo(0.0042, 12,);
        resetRunSpend();
      },
    },),
    it({
      name: 'COUNTS ONLY Xiaomi: spend noted for another provider does not move it',
      fn: async () => {
        resetRunSpend();
        noteRunSpend({ provider: 'deepseek', costUsd: 5, },);
        noteRunSpend({ provider: 'openrouter', costUsd: 7, },);
        expect(xiaomiSpendNow().spentUsd,).toBe(0,);
        resetRunSpend();
      },
    },),
  ],
},);
