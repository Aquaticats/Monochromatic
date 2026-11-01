import { runSpendUsd, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Claude spend
// WHAT THIS RUN HAS SPENT ON THE CLAUDE KEY, as its meter reads it, and whether the key's billing has refused a call.
// ANTHROPIC'S MESSAGES API HAS NO BALANCE OR BUDGET ENDPOINT FOR A PLAIN API KEY (the organisation's cost report needs
// an admin key this route does not hold), so there is no balance to read: the meter reports the priced spend of this
// process (`run-spend-meter.ts`, fed by every SPEND line the client writes) as its field, so the `METERS` line shows
// `anthropic=wet anthropicUsdSpent=...`. The owner's $200 a month has no hard cap in this code ("no hard monthly cap",
// 2026-10-22); the field is how a run's own spend is read against it.
//
// EXHAUSTION IS LEARNED FROM A BILLING ERROR, which the client records on itself (`claude-client.ts`): status 402
// `billing_error`, or the older status 400 whose message says the credit balance is too low. The flag is the
// client's, so it does not move when a call already in flight settles, and the meter reads dry for the rest of the
// run (`claudeIsDry` in `budget-routing.ts`). A top-up is seen on the next launch.

/**
 What this process has spent on the Claude key, and whether its billing has refused a call.

 @example
 ```ts
 const spend: ClaudeSpend = { spentUsd: 0.0042, billingRefused: false, };
 ```
 */
export type ClaudeSpend = {
  /**
   USD the SPEND lines of this process's Claude calls priced: the calls whose usage block arrived at what it reported,
   and each attempt that delivered a stream and failed at its bound (`claude-abandoned-spend.ts`).
   */
  readonly spentUsd: number;

  /**
   Whether a call of this client met a billing error, which holds the provider dry for the rest of the run.
   */
  readonly billingRefused: boolean;
};

/**
 Reads the spend of this process on the Claude key so far.

 @param billingRefused - whether the reading client has met a billing error

 @returns The priced spend and the flag

 @example
 ```ts
 const { spentUsd, } = claudeSpendNow({ billingRefused: false, },);
 ```
 */
export function claudeSpendNow(
  { billingRefused, }: { readonly billingRefused: boolean; },
): ClaudeSpend {
  return {
    spentUsd: runSpendUsd({ provider: 'anthropic', },),
    billingRefused,
  };
}

//endregion Claude spend
