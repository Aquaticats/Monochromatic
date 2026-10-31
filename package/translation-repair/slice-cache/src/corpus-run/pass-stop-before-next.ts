import {
  lifecycleClockStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { runSpendUsd, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  SPEND_CEILING_PROVIDERS,
  spendCeilingNote,
  spendCeilingReached,
} from '@monochromatic-dev/translation-repair-run/ts';
import { printRunLine, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Stop before the next entry
// THE TWO REASONS A PASS STARTS NO FURTHER ENTRY, asked by the attempt queue
// before each one and never mid-entry: the soft time budget, and since
// 2026-09-04 the per-run spend ceiling, applied since 2026-10-22 to each provider that bills in USD on its own.
// Split out of `corpus-pass.ts` at the line cap, and because the pass had no
// unit test of its scheduler's stop rules while both rules were inline.

/**
 Decides whether the pass may start one more entry, and says why not when
 it may not.

 SOFT BUDGET FIRST, because it is the older rule and the cheaper check; a run
 past both prints the time reason, which is the one an operator planned.

 @param elapsedMs - time since the processing loop began, on `monotonicMs`,
 so setting the system clock neither spends the budget nor refunds it
 (ledger B78)

 @param softBudgetMs - time after which no new entry starts

 @param ceilingUsd - USD this run may spend on any one metered provider before
 no new entry starts

 @param wallClock - clock the time of the line it prints is read on, printed as the line's last field `at=`; the wall
 clock unless a test fixes it. The line belongs to the pass and carries no attempt (ledger B960)

 @returns Whether the queue must stop before its next attempt

 @example
 ```ts
 const stop = stopBeforeNextEntry({ elapsedMs: monotonicMs() - start, softBudgetMs, ceilingUsd, },);
 ```
 */
export function stopBeforeNextEntry(
  {
    elapsedMs,
    softBudgetMs,
    ceilingUsd,
    wallClock = wallClockNow,
  }: {
    readonly elapsedMs: number;
    readonly softBudgetMs: number;
    readonly ceilingUsd: number;
    readonly wallClock?: () => Date;
  },
): boolean {
  if (elapsedMs >= softBudgetMs) {
    printRunLine({
      line: `SOFT budget reached after ${String(elapsedMs,)}ms; not starting new entries ${
        lifecycleClockStamp({ wallClock, },)
      }`,
    },);
    return true;
  }

  for (const provider of SPEND_CEILING_PROVIDERS) {
    /**
     What this run has spent so far on this provider, never summed with another's.
     */
    const spentUsd = runSpendUsd({ provider, },);

    if (spendCeilingReached({
      spentUsd,
      ceilingUsd,
    },)) {
      printRunLine({
        line: `${
          spendCeilingNote({
            provider,
            spentUsd,
            ceilingUsd,
          },)
        } ${lifecycleClockStamp({ wallClock, },)}`,
      },);
      return true;
    }
  }
  return false;
}

//endregion Stop before the next entry
