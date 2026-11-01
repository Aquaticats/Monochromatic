/**
 * Tier-promotion thresholds and the pure transition predicate.
 *
 * Split from `composer.ts` so the state-machine module stays under the
 * line cap. `decideTierTransition` is a pure function over the observed
 * signals; the composer applies whatever transition it returns.
 */

import { BYTES_PER_KIB, } from '@monochromatic-dev/const-core/ts';

/**
 * Tier-2 threshold in kibibytes.
 */
const TIER_2_THRESHOLD_KIB = 8;

/**
 * Tier-3 threshold in kibibytes.
 */
const TIER_3_THRESHOLD_KIB = 1_024;

/**
 * Body-size threshold (chars) to promote from tier 1 to tier 2.
 */
export const TIER_2_THRESHOLD: number = TIER_2_THRESHOLD_KIB * BYTES_PER_KIB;

/**
 * Body-size threshold (chars) to promote from tier 2 to tier 3.
 */
export const TIER_3_THRESHOLD: number = TIER_3_THRESHOLD_KIB * BYTES_PER_KIB;

/**
 * Outcome of a single tier-promotion check.
 */
export type TierTransition =
  | { readonly kind: 'none'; }
  | { readonly kind: 'to-tier-2'; }
  | { readonly kind: 'to-tier-3'; };

/**
 * Pure decision: given the current tier and observed signals, what
 * transition (if any) should fire on the next promotion-check tick?
 *
 * Rules:
 *
 * - tier 1 promotes to tier 2 when `length >= TIER_2_THRESHOLD`.
 * - tier 2 promotes to tier 3 when `length >= TIER_3_THRESHOLD`,
 *   `tier3Active` is false, and we are not in edit mode.
 * - All other states return `none`. Promotion is one-way: tier-3 stays
 *   tier-3 even if the body shrinks back below the threshold.
 *
 * @param input - current tier + size + mode signals
 *
 * @returns transition to apply
 *
 * @example
 * ```ts
 * decideTierTransition({ tier: 1, length: 9000, tier3Active: false, inEditMode: false });
 * // { kind: 'to-tier-2' }
 * ```
 */
export function decideTierTransition(
  input: {
    /* oxlint-disable eslint/no-magic-numbers -- tier discriminant union */
    readonly tier: 1 | 2 | 3;
    /* oxlint-enable eslint/no-magic-numbers */
    readonly length: number;
    readonly tier3Active: boolean;
    readonly inEditMode: boolean;
  },
): TierTransition {
  if ((input.tier
    === 1) && (input.length
      >= TIER_2_THRESHOLD))
    return { kind: 'to-tier-2', };
  if (
    (input.tier
      === 2)
    && (input.length
      >= TIER_3_THRESHOLD)
      && (!input.tier3Active)
      && (!input.inEditMode)
  ) {
    return { kind: 'to-tier-3', };
  }
  return { kind: 'none', };
}
