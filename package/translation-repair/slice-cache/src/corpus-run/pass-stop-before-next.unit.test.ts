/**
 Tests for the scheduler's stop rule: the soft time budget and the
 per-run spend ceiling, asked before each entry.

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
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  divertingConsoleLog,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { stopBeforeNextEntry, } from '../../dist/final/node/index.mjs';

/**
 A soft budget no case here reaches by time.
 */
const ROOMY_BUDGET_MS = 1_000_000;

/**
 Instant the injected wall clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 The clock time every line a case prints carries.
 */
const AT = 'at=2026-10-22T08:10:11.123Z';

/**
 Wall clock that reads the same instant every time it is asked.

 @returns Instant the line is printed at

 @example
 ```ts
 stopBeforeNextEntry({ elapsedMs, softBudgetMs, ceilingUsd, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

await describe({
  name: stopBeforeNextEntry.name,
  children: [
    it({
      name: 'LETS the next entry start while the run is inside both its time budget and its allowance',
      fn: async () => {
        resetRunSpend();
        noteRunSpend({
          provider: 'openrouter',
          costUsd: 4.5,
        },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
        },),).toBe(false,);
        resetRunSpend();
      },
    },),
    it({
      name: 'STOPS on the soft budget, whatever has been spent',
      fn: async () => {
        resetRunSpend();
        expect(stopBeforeNextEntry({
          elapsedMs: ROOMY_BUDGET_MS,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
        },),).toBe(true,);
      },
    },),
    it({
      name: 'STOPS once the run has spent its allowance on the metered provider, and NOT on what another '
        + 'provider spent (the owner\'s per-run ceiling of 2026-09-04)',
      fn: async () => {
        resetRunSpend();
        noteRunSpend({
          provider: 'hyper',
          costUsd: 100,
        },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
        },),).toBe(false,);
        noteRunSpend({
          provider: 'openrouter',
          costUsd: 20,
        },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
        },),).toBe(true,);
        resetRunSpend();
      },
    },),
    it({
      name: 'STOPS once the run has spent its allowance on Xiaomi, which no balance bounds and which takes the seat '
        + 'OpenRouter\'s ceiling used to bound, and LETS entries start below it',
      fn: async () => {
        resetRunSpend();
        noteRunSpend({
          provider: 'xiaomi',
          costUsd: 19.99,
        },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
        },),).toBe(false,);
        noteRunSpend({
          provider: 'xiaomi',
          costUsd: 0.01,
        },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
        },),).toBe(true,);
        resetRunSpend();
      },
    },),
    it({
      name: 'STOPS once the run has spent its allowance on DeepSeek, each provider against the ceiling alone, '
        + 'so spend split across providers below the ceiling each does not stop the run',
      fn: async () => {
        resetRunSpend();
        noteRunSpend({
          provider: 'xiaomi',
          costUsd: 12,
        },);
        noteRunSpend({
          provider: 'deepseek',
          costUsd: 12,
        },);
        noteRunSpend({
          provider: 'openrouter',
          costUsd: 12,
        },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
        },),).toBe(false,);
        noteRunSpend({
          provider: 'deepseek',
          costUsd: 8,
        },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
        },),).toBe(true,);
        resetRunSpend();
      },
    },),
    it({
      name: 'STOPS before the first entry under a ceiling of zero, which is how the guard is proven live',
      fn: async () => {
        resetRunSpend();
        expect(stopBeforeNextEntry({
          elapsedMs: 0,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 0,
        },),).toBe(true,);
      },
    },),
    it({
      name: 'ENDS THE SOFT BUDGET LINE WITH THE CLOCK TIME and no attempt, since the line is the one that says when the '
        + 'pass stopped starting entries and belongs to the pass, not to an attempt (ledger B960)',
      fn: async (ctx,) => {
        resetRunSpend();
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        expect(stopBeforeNextEntry({
          elapsedMs: ROOMY_BUDGET_MS,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
          wallClock: fixedWallClock,
        },),).toBe(true,);
        expect(printed.lines,).toEqual([`SOFT budget reached after 1000000ms; not starting new entries ${AT}`,],);
      },
    },),
    it({
      name: 'ENDS THE SPEND CEILING LINE WITH THE CLOCK TIME and no attempt, naming the provider whose spend reached the '
        + 'ceiling',
      fn: async (ctx,) => {
        resetRunSpend();
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        noteRunSpend({
          provider: 'openrouter',
          costUsd: 20,
        },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
          wallClock: fixedWallClock,
        },),).toBe(true,);
        resetRunSpend();
        expect(printed.lines.length,).toBe(1,);
        expect(printed.lines[0],).toContain('SPEND CEILING reached: 20 of 20 USD spent on openrouter this run',);
        expect(printed.lines[0]?.endsWith(` ${AT}`,),).toBe(true,);
        expect(printed.lines[0],).not.toContain('attempt=',);
      },
    },),
    it({
      name: 'PRINTS NO LINE WHEN THE NEXT ENTRY MAY START, so a pass inside its budget and its allowance stays silent',
      fn: async (ctx,) => {
        resetRunSpend();
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        expect(stopBeforeNextEntry({
          elapsedMs: 10,
          softBudgetMs: ROOMY_BUDGET_MS,
          ceilingUsd: 20,
          wallClock: fixedWallClock,
        },),).toBe(false,);
        expect(printed.lines,).toEqual([],);
      },
    },),
  ],
},);
