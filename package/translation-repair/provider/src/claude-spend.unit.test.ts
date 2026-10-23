/**
 Tests for the reader of what this run has spent on the Claude key. The dollars are the process-wide run meter's and
 the billing flag is the client's own, so one test run's refusal never reaches another client.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  noteRunSpend,
  resetRunSpend,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { claudeSpendNow, } from '../dist/final/node/index.mjs';

await describe({
  name: claudeSpendNow.name,
  // SEQUENTIAL: the run spend meter is process-wide, so cases that write it must not overlap.
  concurrency: 1,
  children: [
    it({
      name: 'READS zero on a fresh meter and exactly what the run meter holds for the Claude key after spend is '
        + 'noted, and carries the billing flag it is given',
      fn: async () => {
        resetRunSpend();
        expect(claudeSpendNow({ billingRefused: false, },),).toEqual({ spentUsd: 0, billingRefused: false, },);
        noteRunSpend({ provider: 'anthropic', costUsd: 0.0042, },);
        expect(claudeSpendNow({ billingRefused: true, },).spentUsd,).toBe(runSpendUsd({ provider: 'anthropic', },),);
        expect(claudeSpendNow({ billingRefused: true, },).billingRefused,).toBe(true,);
        resetRunSpend();
      },
    },),
    it({
      name: 'COUNTS ONLY the Claude key: spend noted for another provider does not move it',
      fn: async () => {
        resetRunSpend();
        noteRunSpend({ provider: 'xiaomi', costUsd: 5, },);
        noteRunSpend({ provider: 'openrouter', costUsd: 7, },);
        expect(claudeSpendNow({ billingRefused: false, },).spentUsd,).toBe(0,);
        resetRunSpend();
      },
    },),
  ],
},);
