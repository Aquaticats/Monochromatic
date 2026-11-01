import type { QuotaSnapshot, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Unconfigured Synthetic quota
// SPLIT OUT OF `run-config.ts` at its line budget.

/* oxlint-disable require-await, typescript/require-await -- compatibility meter contract is asynchronous */
/**
 Synthetic quota shape used only when Hyper is sole configured provider.

 Router receives absent Synthetic as dry directly;
 this compatibility method keeps client surface stable for observational callers.

 @returns Explicitly exhausted synthetic quota

 @example
 ```ts
 const quota = await unconfiguredSyntheticQuota();
 ```
 */
export async function unconfiguredSyntheticQuota(): Promise<QuotaSnapshot> {
  return {
    fiveHour: {
      remaining: 0,
      max: 0,
      limited: true,
      nextTickAt: '',
    },
    weekly: {
      percentRemaining: 0,
      nextRegenAt: '',
    },
  };
}
/* oxlint-enable require-await, typescript/require-await */

//endregion Unconfigured Synthetic quota
