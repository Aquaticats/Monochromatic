import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { NeuralWattAllowanceError, } from './neuralwatt-energy.ts';
import { retryAfterMsOf, } from './retry-stated-wait.ts';

/**
 How subscription reports spent allowance.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 How a malformed request is rejected, which Anthropic also used for an empty credit balance before it gave that a
 status of its own.
 */
const HTTP_BAD_REQUEST = 400;

/**
 Words Anthropic's older empty-balance reply carries, "Your credit balance is too low to access the Anthropic API",
 in the lower case the body is compared in.
 */
const LOW_CREDIT_WORDS = 'credit balance is too low';

/**
 How credit balance reports no funds.
 */
const HTTP_PAYMENT_REQUIRED = 402;

/**
 How a provider rejects the credential a request carried, expired or wrong.
 */
const HTTP_UNAUTHORIZED = 401;

/**
 Statuses that mean provider is out of budget rather than unwell.
 */
const BUDGET_REFUSAL_STATUSES: ReadonlySet<number> = new Set([
  HTTP_TOO_MANY_REQUESTS,
  HTTP_PAYMENT_REQUIRED,
],);

/**
 How OpenRouter opens a reply body that carries one upstream endpoint's own
 error instead of a verdict on our account.
 */
const UPSTREAM_ERROR_OPENING = '"message":"Provider returned error"';

/**
 Whether thrown failure is one model's upstream endpoint refusing the call,
 passed on by the aggregator with our account untouched.
 THE SIXTY-SECOND CLASS (XingZ612, 2026-09-19): Inception's shared pool
 rate-limited Mercury 2.5 and OpenRouter answered
 `429 {"error":{"message":"Provider returned error","code":429,"metadata":
 {"raw":"inception/mercury-2.5 is temporarily rate-limited upstream. ...",
 "provider_name":"Inception","provider_error_code":"rate_limit_exceeded",
 "limit_source":"upstream_provider_shared_pool", ...}}}`; read as the
 provider's budget refusal, every such reply held the whole of OpenRouter
 out for 60 s, the editors and refiners (OpenRouter-only seats) read
 unreachable, and the entry stopped three attempts running inside 34
 minutes with every meter wet. The limit is that one model's, so the seat
 is lost for the round and the provider stays where its meter puts it.

 @param error - whatever call threw

 @returns Whether one model's upstream endpoint, not the provider, refused

 @example
 ```ts
 if (isUpstreamModelRefusal({ error, },)) rl.warn(`${modelId}: upstream rate limit, seat lost this round`,);
 ```
 */
export function isUpstreamModelRefusal(
  { error, }: { readonly error: unknown; },
): boolean {
  if (!(error instanceof ProviderHttpError))
    return false;
  if (error.status !== HTTP_TOO_MANY_REQUESTS)
    return false;
  /**
   Body opening the reply carried.
   */
  const { bodyExcerpt, } = error;
  return bodyExcerpt.includes(UPSTREAM_ERROR_OPENING,);
}

/**
 Whether a failure is the older shape of Anthropic's empty-balance reply: status 400 with the low-credit sentence.

 ANTHROPIC'S ALONE: the sentence is its own wording, and another provider's 400 that quotes it (an aggregator
 relaying Anthropic's reply for one model, a body that echoes the request) is a bad request, not that provider's
 empty balance.

 @param provider - provider that answered

 @param status - HTTP status of the reply

 @param bodyExcerpt - opening of the reply body

 @returns Whether the balance, not the request, was refused

 @example
 ```ts
 if (isLowCreditReply({ provider, status, bodyExcerpt, },)) markBillingRefused();
 ```
 */
export function isLowCreditReply(
  {
    provider,
    status,
    bodyExcerpt,
  }: {
    readonly provider: ProviderName;
    readonly status: number;
    readonly bodyExcerpt: string;
  },
): boolean {
  return (provider === 'anthropic')
    && (status === HTTP_BAD_REQUEST)
    && bodyExcerpt
      .toLowerCase()
      .includes(LOW_CREDIT_WORDS,);
}

/**
 Whether thrown failure says provider is out of budget.

 A subscription reports exhaustion as rate limit,
 while credit balance reports it as payment due.
 Retry ladder already rides transient 429 responses;
 reaching router means ladder exhausted.
 A 429 the aggregator passes on from one model's upstream endpoint is that
 model's refusal, not the provider's (`isUpstreamModelRefusal`).

 @param error - whatever call threw

 @returns Whether other provider should be asked instead

 @example
 ```ts
 if (isBudgetRefusal({ error, },)) budgets.markRefused({ provider, },);
 ```
 */
export function isBudgetRefusal(
  { error, }: { readonly error: unknown; },
): boolean {
  // NEURALWATT'S OWN LEDGER REFUSING BEFORE A REQUEST LEAVES (2026-10-20): the
  // day's free energy cannot take the call, so the next provider is asked. NOT
  // A PAYMENT REFUSAL: nothing is owed, and the energy returns when the account's day starts again, at the time of day
  // the run decided at launch (`neuralwatt-day-anchor.ts`), not at 00:00 UTC.
  if (error instanceof NeuralWattAllowanceError)
    return true;
  if (!(error instanceof ProviderHttpError))
    return false;
  if (isUpstreamModelRefusal({ error, },))
    return false;
  return BUDGET_REFUSAL_STATUSES.has(error.status,) || isLowCreditReply(error,);
}

/**
 Whether thrown failure is a refusal about this one call and not about the
 provider, so no hold on the provider should follow it.

 NEURALWATT'S LEDGER REFUSES A CALL WHOSE ESTIMATE DOES NOT FIT BEFORE ANY
 REQUEST LEAVES (2026-10-20). A large prompt can be refused on a day that
 still has room for a hundred small ones; holding the whole provider out for
 the backoff would send those small calls to the providers behind it while the
 free energy sits unspent, which the owner ruled against ("Use the neuralwatt
 free energy front and center."). The ledger's own meter already reads the
 provider dry when it is, so nothing is lost by holding nothing.

 @param error - whatever call threw

 @returns Whether the call alone was refused

 @example
 ```ts
 if (!isCallSizedRefusal({ error, },)) await budgets.markRefused({ provider, signal, },);
 ```
 */
export function isCallSizedRefusal(
  { error, }: { readonly error: unknown; },
): boolean {
  return error instanceof NeuralWattAllowanceError;
}

/**
 Whether thrown failure is a provider rejecting the credential its request
 carried. The credential is the provider's, so the refusal says nothing of the
 model asked and holds the whole provider out for the run (`credential-hold.ts`).
 Only 401 is read, since 403 carries too many other meanings to name a credential.

 @param error - whatever call threw

 @returns Whether the provider rejected its credential

 @example
 ```ts
 if (isCredentialRefusal({ error, },)) credentialHolds.hold({ provider, },);
 ```
 */
export function isCredentialRefusal(
  { error, }: { readonly error: unknown; },
): boolean {
  if (!(error instanceof ProviderHttpError))
    return false;
  return error.status === HTTP_UNAUTHORIZED;
}

/**
 Whether thrown failure says the provider's balance cannot pay for the call.
 THE FOURTEENTH CLASS (hakureico, 2026-09-07): OpenRouter at 0.01 USD read
 wet and answered every call `402: This request requires more credits, or
 fewer max_tokens. You requested up to 131072 tokens, but can only afford
 2411`; held out for the 60 s rate-limit backoff, it came back every minute
 to the same wall, and the seat wait chose its short hold over Hyper's long
 one. A payment refusal is a statement about the balance, not about the
 minute, so the budget layer reads the provider dry until its meter moves.

 @param error - whatever call threw

 @returns Whether the provider's balance refused the call

 @example
 ```ts
 await budgets.markRefused({ provider, signal, paymentRequired: isPaymentRefusal({ error, },), },);
 ```
 */
export function isPaymentRefusal(
  { error, }: { readonly error: unknown; },
): boolean {
  if (!(error instanceof ProviderHttpError))
    return false;
  return (error.status === HTTP_PAYMENT_REQUIRED) || isLowCreditReply(error,);
}

/**
 Wait a refusal names for its provider's return, zero when it names none or
 when the failure is not a provider reply.

 Hyper's daily limit answers "You've hit your daily rate limit. Please try
 again in 2h25m18s" (Huasheng, 2026-09-07); a hold shorter than that walks
 straight back into the same wall, which the router did every 60 s for
 2h53m.

 @param error - whatever call threw

 @returns Milliseconds the provider asked us to stay away

 @example
 ```ts
 await budgets.markRefused({ provider, signal, statedWaitMs: statedWaitMsOf({ error, },), },);
 ```
 */
export function statedWaitMsOf({ error, }: { readonly error: unknown; },): number {
  if (!(error instanceof ProviderHttpError))
    return 0;
  // THE LARGER OF THE TWO, since a provider states its wait in one place: Hyper in the body, Anthropic in the header.
  return Math.max(
    retryAfterMsOf({ bodyText: error.bodyExcerpt, },),
    error.retryAfterMs,
  );
}
