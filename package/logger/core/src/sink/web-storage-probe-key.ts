/**
 Names the key one web storage probe owns for its whole write-read-remove cycle. @module

 A probe that elects a sink has to prove the store round-trips a write, and the
 only way to prove that is to write one. `localStorage` is shared by every tab
 of the origin and exposes no compare-and-set, so two tabs probing one fixed key
 overwrite and remove each other's value: the tab that reads last sees a foreign
 value or nothing, reports the backend unavailable, and loses its sink for the
 whole run. A unique value alone cannot fix that, because the collision that
 matters is the key, not the bytes.

 Every probe therefore names a key no other probe can name, and removes it when
 it is done, so neither a concurrent probe nor an earlier one leaves residue.
 */

/**
 Prefix of every probe key, so a leftover key from a crashed probe is
 recognizable in storage inspection and never matches a log key.
 */
const PROBE_KEY_BASE = '__monochromatic_verify__';

/**
 Radix of the random suffix, so `Number.prototype.toString` yields compact
 alphanumerics.
 */
const SUFFIX_RADIX = 36;

/**
 Length of the random suffix. Eight base-36 characters separate two probes
 started in the same millisecond while keeping the key short.
 */
const SUFFIX_LENGTH = 8;

/**
 Builds the key one storage probe owns until it removes that key again.

 @returns Probe key unique to one verification call.

 @example
 ```ts
 const testKey = createWebStorageProbeKey();
 globalThis.localStorage.setItem(testKey, 'test');
 ```
 */
export function createWebStorageProbeKey(): string {
  /**
   Same-millisecond probes another tab started, kept apart by the random suffix.
   */
  const suffix = Math.random()
    .toString(SUFFIX_RADIX,)
    .slice(
    2,
    2 + SUFFIX_LENGTH,
    )
    .padEnd(
    SUFFIX_LENGTH,
    '0',
    );
  return `${PROBE_KEY_BASE}-${Date.now()}-${suffix}`;
}
