/**
 Shared sentinel for a byte measurement that could not be obtained, kept
 distinct from a measured zero. A git store can legitimately weigh zero bytes
 (an unborn or empty repository), so returning `0` to mean "I could not
 measure" conflates the two and lets a failed probe masquerade as a real
 value. Functions that size object stores or packs return {@link Measured} and
 signal failure with {@link UNMEASURED}, the same pattern the probe layer uses
 with {@link NO_DEEPEN}, {@link NO_TREE0}, and {@link NO_CHURN}.
 
 @module
 */

/**
 Sentinel meaning a measurement could not be obtained, as opposed to a
 measured zero. Distinct identity (a `unique symbol`) so callers compare by
 reference and TypeScript narrows the union.
 
 @example
 ```ts
 const bytes = await objectsDirSize({ repoPath });
 if (bytes === UNMEASURED) return NO_DEEPEN;
 ```
 */
export const UNMEASURED: unique symbol = Symbol('git-clone-size/byte-measurement-unavailable',);

/**
 A byte measurement, or {@link UNMEASURED} when sizing failed. Callers narrow
 with {@link isMeasured} before doing arithmetic.
 
 @example
 ```ts
 function total(value: Measured): number {
   return isMeasured(value) ? value : 0;
 }
 ```
 */
export type Measured = number | typeof UNMEASURED;

/**
 Narrows a {@link Measured} to a concrete byte count, excluding the
 {@link UNMEASURED} sentinel. A type guard so the failure branch stays explicit
 at every call site rather than collapsing into a fabricated zero.
 
 @param value - measurement to test
 
 @returns whether `value` is a real number rather than {@link UNMEASURED}
 
 @example
 ```ts
 const size = await dirSize({ path });
 if (!isMeasured(size)) throw new Error('store unmeasurable');
 const half = size / 2;
 ```
 */
export function isMeasured(value: Measured,): value is number {
  return value !== UNMEASURED;
}
