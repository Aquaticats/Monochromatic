/**
 One-pass candidate selection with call-local provider exclusions and soft provider diversity. @module
 */

/**
 No remaining eligible candidate, distinct from failed selection or invalid configuration.
 */
export const NO_ADVISOR_CANDIDATE: unique symbol = Symbol('advisor/no-untried-candidate');

/**
 Identity needed for provider-level exclusion without inspecting prompt content.
 */
export type AdvisorCandidateIdentity = {
  /**
   Canonical endpoint slug.
   */
  readonly model: string;
  /**
   Registered provider, not the underlying model vendor.
   */
  readonly provider: string;
};

/**
 Choose the next unused candidate, preferring providers not already represented by running reviews.
 
 @param candidates - currently eligible candidates in existing default preference order
 
 @param attemptedModels - models already selected in this operation
 
 @param blockedProviders - credit exclusions read from the operation ledger just in time
 
 @param runningProviders - provider identities already in flight
 
 @returns next eligible candidate or exhaustion sentinel
 
 @example
 ```ts
 const candidate = nextAdvisorCandidate({ candidates, attemptedModels, blockedProviders, runningProviders });
 ```
 */
export function nextAdvisorCandidate<const Candidate extends AdvisorCandidateIdentity>({
  candidates,
  attemptedModels,
  blockedProviders,
  runningProviders,
}: {
  readonly candidates: readonly Candidate[];
  readonly attemptedModels: readonly string[];
  readonly blockedProviders: readonly string[];
  readonly runningProviders: readonly string[];
},): Candidate | typeof NO_ADVISOR_CANDIDATE {
  /**
   Filter every model of a blocked provider, rather than only the failed endpoint.
   */
  const remaining = candidates.filter(function untried(candidate: Candidate,): boolean {
    return (!attemptedModels.includes(candidate.model,)) && (!blockedProviders.includes(candidate.provider,));
  },);
  return remaining.find(function differentProvider(candidate: Candidate,): boolean {
    return !runningProviders.includes(candidate.provider,);
  },) ?? remaining[0]
    ?? NO_ADVISOR_CANDIDATE;
}
