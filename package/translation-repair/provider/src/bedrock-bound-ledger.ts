import type { BedrockServedId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { bedrockAttemptBound, } from './bedrock-cost.ts';
import type { BedrockLedger, } from './bedrock-ledger.ts';
import {
  reportSpend,
  type SpendReckoning,
} from './spend-line.ts';

//region Bedrock bound ledger
// AN ATTEMPT BEDROCK ACCEPTED IS SPEND WHETHER OR NOT ITS USAGE ARRIVED
// (ledger P1, 2026-09-28). The ledger is the only guard on the owner's card,
// and it noted completed calls with usage only: 873 Bedrock streams across the
// logs ended cut or overrun and none reached it, and a whole call whose stream
// carried no usage block would not have either (none of 190,009 logged calls
// did). Each such attempt is written at its bound, on the SPEND line and in
// the ledger, marked with why its figures are reckoned.

/**
 Writes one attempt at the most it could have been billed.

 @param servedId - model the attempt went to

 @param requestBodyBytes - size of the body the attempt sent

 @param maxTokens - `max_tokens` the body carried

 @param reckoning - why no reported usage prices it: abandoned before its
 end, or finished without a usage block

 @param ledger - the spend record the meter is read off

 @example
 ```ts
 await ledgerAtBound({ servedId, requestBodyBytes, maxTokens, reckoning: 'abandoned-bound', ledger, },);
 ```
 */
export async function ledgerAtBound(
  {
    servedId,
    requestBodyBytes,
    maxTokens,
    reckoning,
    ledger,
  }: {
    readonly servedId: BedrockServedId;
    readonly requestBodyBytes: number;
    readonly maxTokens: number;
    readonly reckoning: Extract<SpendReckoning, 'abandoned-bound' | 'unreported-bound'>;
    readonly ledger: BedrockLedger;
  },
): Promise<void> {
  /**
   The most the attempt could have been billed.
   */
  const bound = bedrockAttemptBound({
    servedId,
    requestBodyBytes,
    maxTokens,
  },);
  reportSpend({
    provider: 'bedrock',
    label: servedId,
    extracted: {
      text: '',
      usage: {
        prompt_tokens: bound.promptTokens,
        completion_tokens: bound.completionTokens,
      },
    },
    costUsd: bound.usd,
    estimated: reckoning,
  },);
  await ledger.note({
    at: new Date().toISOString(),
    model: servedId,
    usd: bound.usd,
    promptTokens: bound.promptTokens,
    completionTokens: bound.completionTokens,
    estimated: reckoning,
  },);
}

//endregion Bedrock bound ledger
