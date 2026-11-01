/**
 Credit-specific failure classification and the final provider-dispatch gate. @module
 */

/**
 Diagnostic markers tied to exhausted credits rather than generic payment or throttling errors.
 */
const CREDIT_EXHAUSTION_MARKERS = [
  'out of credits',
  'insufficient credits',
  'not enough credits',
  'credit balance is too low',
  'credit balance exhausted',
  'credits exhausted',
  'insufficient_credit',
  'insufficient_balance',
  'insufficient_quota',
  'insufficient_g1_credits_balance',
] as const;

/**
 Recognize supported exhausted-credit diagnostics from a failed provider outcome only.
 Never apply this classifier to successful review text or arbitrary HTTP 402 status alone.
 
 @param diagnostic - terminal provider failure text, including formatted JSON error bodies
 
 @returns whether the diagnostic provides credit-exhaustion evidence
 
 @example
 ```ts
 isAdvisorCreditExhaustion("402: You're out of credits");
 ```
 */
export function isAdvisorCreditExhaustion(diagnostic: string,): boolean {
  /**
   Provider casing is not part of the exhausted-credit contract.
   */
  const normalized = diagnostic.toLowerCase();
  return CREDIT_EXHAUSTION_MARKERS.some(function matches(marker: string,): boolean {
    return normalized.includes(marker,);
  },);
}

/**
 Reject a provider that became blocked while this attempt was preparing its request.
 
 @param provider - registered identity at the actual dispatch boundary
 
 @param blockedProviders - current operation ledger exclusions, not a cached selection-time copy
 
 @throws when an earlier attempt in this operation established exhausted credits
 
 @example
 ```ts
 assertAdvisorProviderAvailable({ provider, blockedProviders: ledger.snapshot().blockedProviders });
 ```
 */
export function assertAdvisorProviderAvailable({
  provider,
  blockedProviders,
}: {
  readonly provider: string;
  readonly blockedProviders: readonly string[];
},): void {
  if (blockedProviders.includes(provider,))
    throw new AdvisorProviderExcludedError(provider,);
}

/**
 Provider exclusion is distinct from a newly observed provider billing failure.
 */
export class AdvisorProviderExcludedError extends Error {
  /**
   Explain why no provider request was issued for this prepared candidate.
   
   @param provider - operation-local excluded identity
   
   @example
   ```ts
   new AdvisorProviderExcludedError('fixture');
   ```
   */
  constructor(provider: string,) {
    super(`advisor: provider ${provider} is blocked for this call after exhausted credits`,);
    this.name = 'AdvisorProviderExcludedError';
  }
}
