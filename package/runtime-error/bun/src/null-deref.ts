/**
 Intentionally triggers `TypeError: null is not an object (evaluating
 'value.property')`, the wording Bun 1.4.4 uses, by accessing a property on a
 `null` value. The double-cast bypasses TypeScript's static null check so the
 error surfaces at runtime rather than compile time.
 */
export {};

/* oxlint-disable no-unsafe-type-assertion -- intentional unsafe cast to provoke a runtime TypeError */
/**
 Null value cast to an object type to bypass static null checks.
 */
const value = null as unknown as Record<string, unknown>;
/* oxlint-enable no-unsafe-type-assertion */

console.log(value.property,);
