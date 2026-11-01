'use strict';

const { mkdirSync, rmdirSync, } = require('node:fs',);
const { setTimeout: sleep, } = require('node:timers/promises',);
const { dirname, basename, resolve, } = require('node:path',);

/** Match upstream proper-lockfile's default when caller passes no `retries`. */
const DEFAULT_RETRIES = 0;
/** Match upstream's exponential-backoff factor when caller passes no override. */
const DEFAULT_FACTOR = 2;
/** Match upstream's first-retry wait in ms when caller passes no override. */
const DEFAULT_MIN_TIMEOUT = 100;
/** Match upstream's ceiling on a single retry wait in ms. */
const DEFAULT_MAX_TIMEOUT = 10000;

/**
 * Compute the atomic-lock directory path for a target file: a
 * `.<basename>.lock` sibling. Upstream `proper-lockfile` writes a directory
 * with the same naming convention, so observers of `node_modules` neighbours
 * (other processes, file watchers) see no shape change.
 *
 * @param {string} target
 * @returns {string}
 */
function lockDirFor(target,) {
  const abs = resolve(target,);
  return resolve(dirname(abs,), `.${basename(abs,)}.lock`,);
}

/**
 * Acquire the lock by atomic `mkdirSync`. EEXIST means another holder owns
 * the lock and the caller must retry or surface ELOCKED.
 *
 * @param {string} target
 * @returns {() => void}
 * @throws Error with `code === 'ELOCKED'` when the lock is already held;
 *   rethrows any other filesystem error unchanged.
 * @example
 *   const release = acquire('/tmp/auth.json');
 *   // ... critical section ...
 *   release();
 */
function acquire(target,) {
  const dir = lockDirFor(target,);
  try {
    mkdirSync(dir,);
  }
  catch (err) {
    if (err.code === 'EEXIST') {
      const elocked = new Error(`Lock file is already being held (lock: ${dir})`,);
      elocked.code = 'ELOCKED';
      throw elocked;
    }
    throw err;
  }
  const state = { released: false, };
  return function release() {
    if (state.released)
      return;
    state.released = true;
    try {
      rmdirSync(dir,);
    }
    catch {
      // Tolerate disappearance; a stale-cleanup sweep or a parallel holder
      // running through its own release path may have removed the directory.
    }
  };
}

/**
 * Sync lock. Throws ELOCKED on first conflict; callers handle retries
 * themselves. `@earendil-works/pi-coding-agent`'s `auth-storage.js` (lines 32
 * to 54) and `settings-manager.js` (lines 38 to 60) both wrap this in their
 * own sync retry loop.
 *
 * @param {string} target
 * @param {object} [_options] Ignored. Upstream callers always pass
 *   `{ realpath: false }`, so the shim treats the target as already-resolved
 *   without invoking `fs.realpath`.
 * @returns {() => void}
 * @example
 *   const release = lockSync('/tmp/auth.json', { realpath: false });
 *   // ... critical section ...
 *   release();
 */
function lockSync(target, _options,) {
  return acquire(target,);
}

/**
 * Coerce upstream's union (`number | object | undefined`) into a uniform
 * retry config object with default values applied. Mirrors the shape of
 * `node-retry`'s options object, which proper-lockfile forwards to.
 *
 * @param {number | object | null | undefined} retries
 * @returns {{ retries: number, factor: number, minTimeout: number, maxTimeout: number }}
 */
function normaliseRetries(retries,) {
  if (typeof retries === 'number') {
    return {
      retries,
      factor: DEFAULT_FACTOR,
      minTimeout: DEFAULT_MIN_TIMEOUT,
      maxTimeout: DEFAULT_MAX_TIMEOUT,
    };
  }
  if (retries === null || retries === undefined) {
    return {
      retries: DEFAULT_RETRIES,
      factor: DEFAULT_FACTOR,
      minTimeout: DEFAULT_MIN_TIMEOUT,
      maxTimeout: DEFAULT_MAX_TIMEOUT,
    };
  }
  return {
    retries: retries.retries ?? DEFAULT_RETRIES,
    factor: retries.factor ?? DEFAULT_FACTOR,
    minTimeout: retries.minTimeout ?? DEFAULT_MIN_TIMEOUT,
    maxTimeout: retries.maxTimeout ?? DEFAULT_MAX_TIMEOUT,
  };
}

/**
 * Recursive retry loop bounded by `config.retries`. Each ELOCKED waits
 * `timeout` ms (capped by `maxTimeout`) before the next attempt; the wait
 * grows by `factor`. Upstream's `randomize` option is ignored; the workspace
 * does not run concurrent pi instances, so deterministic backoff is enough.
 *
 * @param {{ target: string, config: { retries: number, factor: number, minTimeout: number, maxTimeout: number }, attempt: number, timeout: number }} params
 * @returns {Promise<() => void>}
 */
async function attemptLock({ target, config, attempt, timeout, },) {
  const maxAttempts = config.retries + 1;
  try {
    return acquire(target,);
  }
  catch (err) {
    if (err.code === 'ELOCKED' && attempt < maxAttempts) {
      await sleep(timeout,);
      return attemptLock({
        target,
        config,
        attempt: attempt + 1,
        timeout: Math.min(timeout * config.factor, config.maxTimeout,),
      },);
    }
    throw err;
  }
}

/**
 * Async lock with internal retry. Upstream's `stale`, `onCompromised`, and
 * `realpath` options are accepted (the second parameter is a flat record)
 * but ignored; only `retries` shapes behavior. The returned release callback
 * is the sync function produced by `acquire`, so `await release()` in async
 * callers resolves immediately because `await undefined` is non-blocking.
 *
 * @param {string} target
 * @param {object} [options]
 * @returns {Promise<() => void>}
 * @example
 *   const release = await lock('/tmp/auth.json', {
 *     retries: { retries: 10, factor: 2, minTimeout: 100, maxTimeout: 10000 },
 *   });
 *   // ... critical section ...
 *   await release();
 */
async function lock(target, options,) {
  const { retries, } = options ?? {};
  const config = normaliseRetries(retries,);
  return attemptLock({
    target,
    config,
    attempt: 1,
    timeout: config.minTimeout,
  },);
}

module.exports = lock;
module.exports.lock = lock;
module.exports.lockSync = lockSync;
