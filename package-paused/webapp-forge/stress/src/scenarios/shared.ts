/**
 * Shared CLI-flag and timing helpers for stress scenarios.
 */

/**
 * Median percentile fraction.
 */
export const P50 = 0.5;

/**
 * 99th percentile fraction.
 */
export const P99 = 0.99;

/**
 * Decimal radix for `parseInt`.
 */
const DECIMAL_RADIX = 10;

/**
 * Reads a `--key=value` flag from `process.argv`.
 *
 * @param name - flag name without the leading dashes
 *
 * @returns flag value, or `undefined` if absent
 *
 * @example
 * ```ts
 * const out = getFlag('out');
 * ```
 */
export function getFlag(name: string,): string | undefined {
  /**
   * Literal `--name=` prefix used to identify the flag entry in argv.
   */
  const prefix = `--${name}=`;
  /**
   * First argv entry matching the prefix, or `undefined` when the flag is absent.
   */
  const argument = process.argv
    .find(function hasPrefix(entry,) {
    return entry.startsWith(prefix,);
  },);
  return argument?.slice(prefix.length,);
}

/**
 * Reads an integer flag with a default fallback.
 *
 * @param row - flag name and fallback
 *
 * @returns parsed integer
 *
 * @example
 * ```ts
 * const events = intFlag({ name: 'burst-events', fallback: 100 });
 * ```
 */
export function intFlag(row: {
  name: string;
  fallback: number;
},): number {
  /**
   * String form of the flag; `undefined` falls through to the default.
   */
  const raw = getFlag(row.name,);
  if (raw === undefined)
    return row.fallback;
  /**
   * Numeric interpretation; non-finite results (NaN, Infinity) also fall back.
   */
  const parsed = Number.parseInt(
    raw,
    DECIMAL_RADIX,
  );
  return Number.isFinite(parsed,) ? parsed : row.fallback;
}

export { wait, } from '@monochromatic-dev/async-time-core';
