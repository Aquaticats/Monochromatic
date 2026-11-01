import { BYTES_PER_MIB, } from '@monochromatic-dev/const-core/ts';
import {
  readFile,
  stat,
} from 'node:fs/promises';

/**
 Cap multiplier for {@link DEFAULT_MAX_HASH_SIZE_BYTES} expressed in mebibytes.
 Sixteen mebibytes is the default upper bound for files the cache will hash.
 
 Kept as a named constant rather than inlined into {@link DEFAULT_MAX_HASH_SIZE_BYTES}
 so a future tune (e.g. drop to 4 MiB on memory-constrained CI) lands in one place,
 and so the magnitude is greppable.
 */
const DEFAULT_MAX_HASH_SIZE_MIB = 16;

/**
 Default upper bound for files the cache will hash, in bytes.
 Files larger than this skip hashing and the caller fires unconditionally;
 the cap prevents pulling multi-GB files into memory just to compare digests.
 
 @example
 ```ts
 const cache = new HashCache({ maxHashSize: DEFAULT_MAX_HASH_SIZE_BYTES, });
 ```
 */
export const DEFAULT_MAX_HASH_SIZE_BYTES: number = DEFAULT_MAX_HASH_SIZE_MIB
  * BYTES_PER_MIB;

/**
 Real sentinel returned by {@link HashCache.hashFile} when a file exceeds the
 configured size cap and is deliberately not read into memory. A unique
 `Symbol` rather than `null` so the "too large to compare" outcome stays a
 distinct value (never collides with a real hex digest) without a nullish
 union; callers treat it as "fire without comparing content".
 
 @example
 ```ts
 const hash = await cache.hashFile(path,);
 if (hash === OVERSIZED) return true; // fire; do not compare
 ```
 */
export const OVERSIZED: unique symbol = Symbol('hash input exceeds size cap',);

/**
 Options for constructing a {@link HashCache}.
 */
export type HashCacheOptions = {
  /**
   Upper bound (bytes) for files {@link HashCache.hashFile} will read into memory.
   Files above this size return `null` so the caller can treat the event
   as "fire without comparing content" instead of blocking on a huge read.
   */
  readonly maxHashSize?: number;
};

/* oxlint-disable no-restricted-syntax/no-class -- per-instance mutable cache: one HashCache lives per `startWatchRestart()` call (concurrent watch sessions each need their own), state is `#private`-encapsulated, and the class is an exported library primitive consumers instantiate via `new`; a module-level container cannot model multiple concurrent instances. */
/**
 In-memory map of absolute path → sha256 hex of the last seen file content.
 
 Owned by `startWatchRestart`; passed via `WatchCtx.hashCache` to filters
 (specifically `contentHashFilter`) so the watcher's pre-populate phase
 (events before chokidar emits `ready`) and the live-compare phase
 (events after `ready`) operate on the same data.
 
 Storage is by absolute path; callers are expected to resolve once upstream
 (the watcher does this) so the cache never has to normalize and `get`/`set`
 are O(1) Map lookups.
 
 @example
 ```ts
 const cache = new HashCache();
 const hash = await cache.hashFile('/abs/path/to/index.ts');
 if (hash !== null) cache.set({ path: '/abs/path/to/index.ts', hash, });
 if (cache.get('/abs/path/to/index.ts') === hash) {
   // byte-identical write; skip restart
 }
 ```
 */
export class HashCache {
  /**
   Backing store: absolute path → sha256 hex.
   Class-private; consumers go through {@link get}/{@link has}/{@link set}/{@link delete}.
   */
  readonly #map: Map<string, string> = new Map<string, string>();

  /**
   Captured maxHashSize from the constructor options.
   Frozen at construction so existing entries cannot be invalidated mid-life by a config change.
   */
  readonly #maxHashSize: number;

  /**
   Constructs an empty cache.
   
   @param maxHashSize - upper bound (bytes); files above this size will read as `null` from {@link hashFile}.
     Defaults to {@link DEFAULT_MAX_HASH_SIZE_BYTES}.
   
   @example
   ```ts
   const cache = new HashCache({ maxHashSize: 1024 * 1024, });
   ```
   */
  constructor({ maxHashSize, }: HashCacheOptions = {},) {
    this.#maxHashSize = maxHashSize ?? DEFAULT_MAX_HASH_SIZE_BYTES;
  }

  /**
   Reads a file from disk and computes the sha256 hex digest of its bytes.
   
   Returns `null` when the file's size exceeds the configured `maxHashSize`;
   the caller treats `null` as "byte-equality unknown, fire anyway" so a 10 GB
   file does not block dev-server restarts on a 10-second read.
   
   The method does not consult or mutate the cache; orchestrators that want
   to record the hash call {@link set} themselves after a non-null return.
   Keeping read and store separate lets callers compose pre-populate
   (hash + store unconditionally) and live-compare (hash, compare to stored,
   decide whether to overwrite) without two different methods.
   
   @param absolutePath - path read directly; caller is expected to have resolved it
   
   @returns sha256 hex string, or {@link OVERSIZED} when the file is too large
   
   @example
   ```ts
   const hash = await cache.hashFile('/abs/path/to/index.ts');
   if (hash !== OVERSIZED) cache.set({ path, hash, });
   ```
   
   @throws Error when the file cannot be stat'd or read (missing, permission denied, ...).
     The watcher's event-handler layer decides whether to swallow ENOENT
     (legitimate race against `unlink`) or escalate; this method does not guess.
   */
  async hashFile(absolutePath: string,): Promise<string | typeof OVERSIZED> {
    /**
     File size pulled from `fstat` to gate the read against `#maxHashSize` before allocating bytes.
     */
    const { size, } = await stat(absolutePath,);
    if (size > this
      .#maxHashSize)
      return OVERSIZED;
    /**
     File bytes; Buffer extends Uint8Array, accepted directly by SubtleCrypto
     */
    const bytes = await readFile(absolutePath,);
    /**
     Raw SHA-256 digest as an `ArrayBuffer`; hex-encoded immediately below for the cache key.
     */
    const digest = await crypto.subtle
      .digest(
      'SHA-256',
      bytes,
      );
    return Buffer.from(digest,)
      .toString('hex',);
  }

  /* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors Map.get: this method forwards directly to the backing Map's `get`, whose `string | undefined` return (undefined when the key is absent) is the external API's shape, not modelling our own optionality. */
  /**
   Looks up the stored hash for a path.
   
   @param absolutePath - exact key passed to a prior {@link set}; no normalization happens here
   
   @returns sha256 hex, or `undefined` when the path has never been recorded
   
   @example
   ```ts
   const prior = cache.get('/abs/path/to/index.ts');
   ```
   */
  get(absolutePath: string,): string | undefined {
    return this.#map
      .get(absolutePath,);
  }
  /* oxlint-enable no-restricted-syntax/no-nullish-union */

  /**
   Reports whether a path has any recorded hash.
   
   The watcher relies on this to distinguish "post-`ready` first-seen event"
   (no entry; treat as a fresh file that should fire) from "post-`ready`
   already-known path" (compare hashes).
   
   @param absolutePath - exact key passed to a prior {@link set}; no normalization happens here
   
   @returns `true` when the cache has a hash for this path
   
   @example
   ```ts
   if (!cache.has(event.path)) {
     // first-seen file
   }
   ```
   */
  has(absolutePath: string,): boolean {
    return this.#map
      .has(absolutePath,);
  }

  /**
   Records a path → hash pair, overwriting any prior value.
   
   Destructured because positional args (`set(path, hash)`) make it possible
   to swap them silently; the destructured form makes the intent named at
   every call site and survives refactors.
   
   @param path - absolute path the hash describes (same key that {@link get} and {@link has} will use)
   
   @param hash - hex digest returned by {@link hashFile}
   
   @example
   ```ts
   const hash = await cache.hashFile(file);
   if (hash !== null) cache.set({ path: file, hash, });
   ```
   */
  set(
    {
      path,
      hash,
    }: {
      readonly path: string;
      readonly hash: string;
    },
  ): void {
    this.#map
      .set(
      path,
      hash,
      );
  }

  /**
   Removes a path's record. Driven by `unlink` events so a future `add`
   of the same path starts from a clean slate instead of matching the
   pre-deletion hash by accident.
   
   @param absolutePath - exact key passed to a prior {@link set}; no normalization happens here
   
   @returns `true` when an entry was removed, `false` when no entry existed
   
   @example
   ```ts
   cache.delete(event.path);
   ```
   */
  delete(absolutePath: string,): boolean {
    return this.#map
      .delete(absolutePath,);
  }

  /**
   Number of recorded paths.
   Exposes the Map's `size` so tests can verify state without poking the private field.
   
   @returns count of entries currently stored in the cache
   
   @example
   ```ts
   expect(cache.size,).toBe(0,);
   ```
   */
  get size(): number {
    return this.#map
      .size;
  }
}
/* oxlint-enable no-restricted-syntax/no-class */
