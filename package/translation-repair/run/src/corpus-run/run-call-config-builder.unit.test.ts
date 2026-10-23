/**
 Tests that the run's call configuration is built through the check that
 refuses stream bounds at or above the per-call deadline.

 WHY: the shipped configuration was inconsistent for three runs and nothing
 refused it (ledger B804). `RUN_CALL_CONFIG` is built once at module load by
 `runCallConfigOf`; a refusal there ends the run before it spends anything.
 These cases hand the builder the pair that shipped and read the refusal.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { StreamBoundsOutsideCallTimeoutError, } from '@monochromatic-dev/translation-repair-provider-stream';

import { runCallConfigOf, } from '../../dist/final/node/index.mjs';

await describe({
  name: runCallConfigOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'REFUSES THE PAIR THAT SHIPPED, both stream bounds at 600000 beside a deadline of 360000, by name',
      fn: async () => {
        await Promise.resolve();
        /**
         What the builder raised.
         */
        const refusal = caught(function shipped(): unknown {
          return runCallConfigOf({
            perCallTimeoutMs: 360_000,
            streamFirstContentMs: 600_000,
            streamIdleMs: 600_000,
          },);
        },);
        expect(refusal,).toBeInstanceOf(StreamBoundsOutsideCallTimeoutError,);
      },
    },),

    it({
      name: 'HANDS BACK THE THREE KNOBS UNCHANGED WHEN THEY AGREE',
      fn: async () => {
        await Promise.resolve();
        expect(runCallConfigOf({
          perCallTimeoutMs: 360_000,
          streamFirstContentMs: 120_000,
          streamIdleMs: 90_000,
        },),).toEqual({
          perCallTimeoutMs: 360_000,
          streamFirstContentMs: 120_000,
          streamIdleMs: 90_000,
          claudeCallTimeoutMs: 460_000,
        },);
      },
    },),

    it({
      name: 'STAMPS THE LARGER OF THE CALLER\'S DEADLINE AND THE CLAUDE DEADLINE as the one a Claude call runs under '
        + '(F2 '
        + 'of the re-check of the second fix round)',
      fn: async () => {
        await Promise.resolve();
        expect(runCallConfigOf({
          perCallTimeoutMs: 900_000,
          streamFirstContentMs: 120_000,
          streamIdleMs: 90_000,
        },).claudeCallTimeoutMs,).toBe(900_000,);
      },
    },),
  ],
},);
