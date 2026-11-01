import {
  PROVIDER_ORDER,
  type ProviderName,
  providerRecord,
} from './provider-name.ts';

//region Run spend meter
// WHAT THIS PROCESS HAS SPENT SO FAR, per provider, in the USD the wire
// reported.
//
// WHY A METER IN THE PROCESS AND NOT ONLY IN THE LOG. `spend-line.ts` writes
// every call's cost where a reader can total it afterwards, and
// `corpus-run/spend-report.ts` is that reader. Neither can stop anything: the
// owner asked on 2026-09-04 for a per-run ceiling that refuses to START new
// entries once a run has spent its allowance, and a ceiling needs the running
// total while the run is still running. A meter the provider serves would do,
// but the OpenRouter credits meter is a balance, which auto top-up refills, so
// it cannot say what THIS run spent.
//
// PROCESS-WIDE STATE, DELIBERATELY. A run is a process, every call in it
// reports through `reportSpend`, and the ceiling is asked from one place. The
// mutable record is the whole of the state and nothing else reads it; tests
// reset it between cases.
//
// USD ONLY. Synthetic bills nothing per call and Hyper's credits are its own
// unit, so only calls that carried a `cost=` count here, and the total is a
// floor for the same reason the spend report's is: a cut stream reports no
// cost.

/**
 Running total per provider, in USD the wire reported for this process's
 calls.
 */
const spentUsd: Record<ProviderName, number> = providerRecord({
  of: function nothingYet(): number {
    return 0;
  },
},);

/**
 Adds one call's reported cost to its provider's running total.

 @param provider - meter the call drew on

 @param costUsd - USD the wire reported for the call

 @example
 ```ts
 noteRunSpend({ provider: 'openrouter', costUsd: 0.0042, },);
 ```
 */
export function noteRunSpend(
  {
    provider,
    costUsd,
  }: {
    readonly provider: ProviderName;
    readonly costUsd: number;
  },
): void {
  spentUsd[provider] += costUsd;
}

/**
 Reads what this process has spent on one provider so far.

 @param provider - meter to read

 @returns USD reported on that provider's calls since the process began, or
 since the last reset

 @example
 ```ts
 const soFar = runSpendUsd({ provider: 'openrouter', },);
 ```
 */
export function runSpendUsd({ provider, }: { readonly provider: ProviderName; },): number {
  return spentUsd[provider];
}

/**
 Zeroes every provider's running total.

 FOR TESTS, which share one process across cases; a run never resets.

 @example
 ```ts
 resetRunSpend();
 ```
 */
export function resetRunSpend(): void {
  for (const provider of PROVIDER_ORDER)
    spentUsd[provider] = 0;
}

//endregion Run spend meter
