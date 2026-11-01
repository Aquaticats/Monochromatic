/**
 Available-usage snapshotting and aggregation without counting streaming updates twice. @module
 */
import type { Usage, } from '@earendil-works/pi-ai';
import type { ReadonlyDeep, } from 'type-fest';

//region Accounting

/**
 Copy provider-owned usage before retaining or publishing it.
 
 @param usage - latest provider counters
 
 @returns detached counter snapshot
 
 @example
 ```ts
 const snapshot = copyAdvisorUsage(message.usage);
 ```
 */
export function copyAdvisorUsage(usage: ReadonlyDeep<Usage>,): ReadonlyDeep<Usage> {
  return {
    ...usage,
    cost: { ...usage.cost, },
  };
}

/**
 Aggregate one latest usage snapshot per attempt, preserving optional subset counters.
 
 @param usages - available snapshots, with absent usage omitted
 
 @returns sum without adding reasoning or one-hour cache subsets to totals again
 
 @example
 ```ts
 const aggregate = aggregateAdvisorUsage([first.usage, second.usage]);
 ```
 */
export function aggregateAdvisorUsage(usages: readonly ReadonlyDeep<Usage>[],): ReadonlyDeep<Usage> {
  return usages.reduce(
    function addUsage(
      total: ReadonlyDeep<Usage>,
      usage: ReadonlyDeep<Usage>,
    ): ReadonlyDeep<Usage> {
      return {
        input: total.input + usage.input,
        output: total.output + usage.output,
        cacheRead: total.cacheRead + usage.cacheRead,
        cacheWrite: total.cacheWrite + usage.cacheWrite,
        totalTokens: total.totalTokens + usage.totalTokens,
        ...((total.reasoning === undefined) && (usage.reasoning === undefined)
          ? {} : { reasoning: (total.reasoning ?? 0) + (usage.reasoning ?? 0), }),
        ...((total.cacheWrite1h === undefined) && (usage.cacheWrite1h === undefined)
          ? {} : { cacheWrite1h: (total.cacheWrite1h ?? 0) + (usage.cacheWrite1h ?? 0), }),
        cost: {
          input: total.cost
            .input
            + usage.cost
              .input,
          output: total.cost
            .output
            + usage.cost
              .output,
          cacheRead: total.cost
            .cacheRead
            + usage.cost
              .cacheRead,
          cacheWrite: total.cost
            .cacheWrite
            + usage.cost
              .cacheWrite,
          total: total.cost
            .total
            + usage.cost
              .total,
        },
      };
    },
    {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 0,
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        total: 0,
      },
    },
  );
}

//endregion Accounting
