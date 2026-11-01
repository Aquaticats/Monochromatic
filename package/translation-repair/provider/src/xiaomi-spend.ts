import { runSpendUsd, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Xiaomi spend
// WHAT THIS RUN HAS SPENT ON XIAOMI, as its meter reads it. XIAOMI DOCUMENTS
// NO BALANCE OR BILLING ENDPOINT (the API reference, read 2026-10-22; the
// console shows usage only), so there is no balance to read and no credit to
// configure, unlike Bedrock's ledger-backed meter. The meter answers wet and
// reports the priced spend of this process (`run-spend-meter.ts`, fed by every
// SPEND line the client writes) as its field, so the `METERS` line shows
// `xiaomi=wet xiaomiUsdSpent=...`.
//
// EXHAUSTION IS LEARNED FROM A PAYMENT REFUSAL, not from this meter: Xiaomi's
// error list names `402 - Insufficient Balance` ("Insufficient account balance",
// `https://mimo.mi.com/docs/en-US/api/guidance/error-codes`, read 2026-10-22),
// which the router passes to `markRefused` as `paymentRequired` and the budget
// layer holds dry until the meter's level moves (`provider-budget.ts`). The
// level here is the run's spend, so a call that settles after the refusal
// moves it and lets one more call through to meet the refusal again: a limit
// of this meter, recorded in `doc/decision/translation-repair-direct-providers.md`.

/**
 What this process has spent on Xiaomi.

 @example
 ```ts
 const spend: XiaomiSpend = { spentUsd: 0.0042, };
 ```
 */
export type XiaomiSpend = {
  /**
   USD the SPEND lines of this process's Xiaomi calls priced, a floor over
   the calls whose usage block arrived.
   */
  readonly spentUsd: number;
};

/**
 Reads the spend of this process on Xiaomi so far.

 @returns The priced spend

 @example
 ```ts
 const { spentUsd, } = xiaomiSpendNow();
 ```
 */
export function xiaomiSpendNow(): XiaomiSpend {
  return { spentUsd: runSpendUsd({ provider: 'xiaomi', }), };
}

//endregion Xiaomi spend
