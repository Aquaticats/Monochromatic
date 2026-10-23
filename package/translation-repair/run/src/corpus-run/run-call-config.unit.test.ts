/**
 Tests that the call configuration a pass stamps into its artifacts has stream
 bounds that can fire before the per-call deadline.

 WHY: the `zheermao22` artifact's `callConfig` read `perCallTimeoutMs: 360000,
 streamFirstByteMs: 600000, streamIdleMs: 600000`, so neither stream guard could
 fire (ledger B804). The configuration is built once, at module load, through
 a check that refuses a bound at or above the deadline; this case reads what it
 built.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  CLAUDE_COMPLETION_CAP,
  claudeCallDeadlineMs,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  RUN_CALL_CONFIG,
  RUN_PER_CALL_TIMEOUT_MS,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'RUN_CALL_CONFIG',
  concurrency: 1,
  children: [
    it({
      name: 'STAMPS STREAM BOUNDS SHORTER THAN THE PER-CALL DEADLINE, so each guard can fire before the deadline '
        + 'does',
      fn: async () => {
        await Promise.resolve();
        expect(RUN_CALL_CONFIG.perCallTimeoutMs,).toBe(RUN_PER_CALL_TIMEOUT_MS,);
        expect(RUN_CALL_CONFIG.streamFirstContentMs,).toBeLessThan(RUN_PER_CALL_TIMEOUT_MS,);
        expect(RUN_CALL_CONFIG.streamIdleMs,).toBeLessThan(RUN_PER_CALL_TIMEOUT_MS,);
      },
    },),

    it({
      name: 'NAMES THE FIRST BOUND FOR WHAT IT TIMES, text and not a byte, so a cohort stamped before the change '
        + 'is told from one stamped after',
      fn: async () => {
        await Promise.resolve();
        expect(Object.keys(RUN_CALL_CONFIG,).toSorted(),).toEqual([
          'claudeCallTimeoutMs',
          'perCallTimeoutMs',
          'streamFirstContentMs',
          'streamIdleMs',
        ],);
      },
    },),

    it({
      name: 'STAMPS THE DEADLINE A CLAUDE CALL RUNS UNDER BESIDE THE ONE EVERY OTHER CALL RUNS UNDER (F2 of the '
        + 're-check of the second fix round), from the same constants the client reads, so a cohort pooled by '
        + 'this stamp does not read a Claude timeout line as a cut at 360,000 ms',
      fn: async () => {
        await Promise.resolve();
        expect(RUN_CALL_CONFIG.claudeCallTimeoutMs,).toBe(claudeCallDeadlineMs({
          maxTokens: CLAUDE_COMPLETION_CAP,
        },),);
        expect(RUN_CALL_CONFIG.claudeCallTimeoutMs,).toBe(460_000,);
        expect(RUN_CALL_CONFIG.perCallTimeoutMs,).toBe(RUN_PER_CALL_TIMEOUT_MS,);
      },
    },),
  ],
},);
