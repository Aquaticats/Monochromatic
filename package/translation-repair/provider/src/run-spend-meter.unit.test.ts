/**
 Tests for the process-wide run spend meter and the line that feeds it.

 Fixtures are cat-themed invention.

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
import { reportSpend, } from '../dist/final/node/index.mjs';

await describe({
  name: 'run spend meter',
  children: [
    it({
      name: 'ACCUMULATES per provider, reads back what was noted, and RESETS to nothing',
      fn: async () => {
        resetRunSpend();
        noteRunSpend({
          provider: 'openrouter',
          costUsd: 0.25,
        },);
        noteRunSpend({
          provider: 'openrouter',
          costUsd: 0.5,
        },);
        noteRunSpend({
          provider: 'hyper',
          costUsd: 2,
        },);
        expect(runSpendUsd({ provider: 'openrouter', },),).toBe(0.75,);
        expect(runSpendUsd({ provider: 'hyper', },),).toBe(2,);
        expect(runSpendUsd({ provider: 'synthetic', },),).toBe(0,);
        resetRunSpend();
        expect(runSpendUsd({ provider: 'openrouter', },),).toBe(0,);
        expect(runSpendUsd({ provider: 'hyper', },),).toBe(0,);
      },
    },),
    it({
      name: 'IS FED BY reportSpend from the same cost= its line carries, and NOT by a line that carries '
        + 'no cost, so the ceiling and the spend report can never disagree about what was counted',
      fn: async () => {
        resetRunSpend();
        reportSpend({
          provider: 'openrouter',
          label: 'cat/whiskers',
          extracted: {
            text: 'The cat approved this rendering.',
            usage: {
              prompt_tokens: 8,
              completion_tokens: 2,
            },
          },
          costUsd: 0.125,
        },);
        reportSpend({
          provider: 'openrouter',
          label: 'cat/whiskers',
          extracted: { text: 'The cat approved this rendering.', },
        },);
        reportSpend({
          provider: 'synthetic',
          label: 'hf:cat/whiskers',
          extracted: {
            text: 'The cat approved this rendering.',
            usage: {
              prompt_tokens: 8,
              completion_tokens: 2,
            },
          },
        },);
        expect(runSpendUsd({ provider: 'openrouter', },),).toBe(0.125,);
        expect(runSpendUsd({ provider: 'synthetic', },),).toBe(0,);
        resetRunSpend();
      },
    },),
  ],
},);
