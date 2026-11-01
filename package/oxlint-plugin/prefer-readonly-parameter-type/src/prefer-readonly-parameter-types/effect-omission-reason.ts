/**
 Bounded reason categories persisted for a source whose callables were left out of the effect index.

 Split from `effect-cache-envelope.ts` so the record of an omission inside an installed package can
 validate its categories without importing the envelope that contains it.

 @module
 */

/**
 Bounded reason category for a direct-summary construction failure.
 */
export const DIRECT_SUMMARY_FAILURE_REASON = 'direct-summary-construction-failed';

/**
 Bounded persisted reason for source-local summary coverage.
 */
export type EffectSummaryOmissionReason = typeof DIRECT_SUMMARY_FAILURE_REASON;

/**
 Tests whether unknown value is one bounded omission reason category.

 @param value - Parsed reason candidate.

 @returns whether candidate names supported category.

 @example
 ```ts
 isOmissionReason('direct-summary-construction-failed');
 ```
 */
export function isOmissionReason(value: unknown,): value is EffectSummaryOmissionReason {
  return value === DIRECT_SUMMARY_FAILURE_REASON;
}

/**
 Tests whether every entry of a parsed list is a known omission reason category.
 
 @param values - Parsed list candidate.
 
 @returns whether each entry names a supported category.
 
 @example
 ```ts
 areOmissionReasons(['direct-summary-construction-failed']);
 ```
 */
export function areOmissionReasons(values: readonly unknown[],): values is readonly EffectSummaryOmissionReason[] {
  return values.every(function knownReason(value,): boolean {
    return isOmissionReason(value,);
  },);
}
