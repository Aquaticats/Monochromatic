'use strict';

const {
  linkSync,
  lstatSync,
  readFileSync,
  renameSync,
  rmdirSync,
  unlinkSync,
  writeFileSync,
} = require('node:fs',);
const { randomUUID, } = require('node:crypto',);
const { setTimeout: sleep, } = require('node:timers/promises',);
const { basename, dirname, resolve, } = require('node:path',);

/** Match upstream proper-lockfile's default when caller passes no `retries`. */
const DEFAULT_RETRIES = 0;
/** Match upstream's exponential-backoff factor when caller passes no override. */
const DEFAULT_FACTOR = 2;
/** Match upstream's first-retry wait in ms when caller passes no override. */
const DEFAULT_MIN_TIMEOUT = 100;
/** Match upstream's ceiling on a single retry wait in ms. */
const DEFAULT_MAX_TIMEOUT = 10000;
/**
 * How long a lock nobody can be named for stands before it is taken over, when
 * the caller passed no `stale` of its own.
 *
 * This bound covers the only locks a holder cannot be named for: an empty lock
 * directory left by an earlier version of this shim, and a lock file whose
 * record a machine crash emptied before its data reached the disk. Neither names
 * a process, so neither can be proven dead, and both are aged out instead.
 * Ten seconds is upstream proper-lockfile's own default `stale` bound.
 */
const DEFAULT_UNOWNED_LOCK_GRACE_MS = 10000;
/**
 * Shortest interval between two ownership checks of a held lock, in ms.
 *
 * Upstream rechecks at `stale / 2`; the floor keeps a caller that passes a tiny
 * `stale` from turning the check into a busy loop.
 */
const MIN_COMPROMISE_CHECK_INTERVAL_MS = 100;
/** Marker telling a record this shim wrote from content anything else wrote. */
const RECORD_FORMAT = 'shim-proper-lockfile/1';
/** Process states that mean the process occupies no instruction any more. */
const DEAD_PROCESS_STATES = new Set(['X', 'x', 'Z',],);
/** Suffix of the record file written to disk temp-for-link; see `createExclusiveFile`. */
const TEMPORARY_SUFFIX = '.tmp';

/**
 * Compute the path of the lock file that guards `target`: a `.<basename>.lock`
 * sibling, the same place the previous version of this shim put its lock
 * directory, so an operator watching for a stuck lock looks in the same spot.
 *
 * @param {string} target
 * @returns {string}
 */
function lockPathFor(target,) {
  const absolute = resolve(target,);
  return resolve(dirname(absolute,), `.${basename(absolute,)}.lock`,);
}

/**
 * Path of the takeover record that arbitrates which caller may replace one dead
 * lock. A sibling of the lock file, so it shares the lock's directory and can be
 * created by the same atomic step.
 *
 * @param {string} lockPath
 * @returns {string}
 */
function takeoverPathFor(lockPath,) {
  return `${lockPath}.takeover`;
}

//region Process identity: which process holds a lock, told apart from a later one

let probedProcStat;
let cachedBootIdentity;
let cachedOwnStartTicks;
let temporaryCounter = 0;

/**
 * Whether this platform's `/proc` describes processes, probed once. Without it a
 * lock is never proven dead, only a pid's absence is used.
 *
 * @returns {boolean}
 */
function procStatAvailable() {
  if (probedProcStat === undefined) {
    try {
      readFileSync('/proc/self/stat', { encoding: 'utf8', },);
      probedProcStat = true;
    }
    catch (error) {
      // A platform without `/proc` is answered by `process.kill` alone; the pid
      // is all the proof it has.
      void error;
      probedProcStat = false;
    }
  }
  return probedProcStat;
}

/**
 * Identity of this boot, so a pid and start tick recorded before a reboot do not
 * name a process started after it.
 *
 * @returns {string | undefined} The boot id, or `undefined` when the platform does not publish one
 */
function bootIdentity() {
  if (cachedBootIdentity === undefined) {
    try {
      cachedBootIdentity = readFileSync('/proc/sys/kernel/random/boot_id', { encoding: 'utf8', },).trim();
    }
    catch (error) {
      // A site that publishes no boot id keeps pid-plus-start-tick identity and
      // loses only the reboot discrimination.
      void error;
      cachedBootIdentity = '';
    }
  }
  return cachedBootIdentity === '' ? undefined : cachedBootIdentity;
}

/**
 * State and start tick of one `/proc/<pid>/stat`.
 *
 * Field 3 is the process state and field 22 its start in clock ticks since the
 * boot. Two processes with one pid differ at field 22, which is what makes a
 * record a proof of identity rather than a name.
 *
 * @param {string} stat - Contents of `/proc/<pid>/stat`
 * @returns {{ state: string, startTicks: number | null } | null} The identity, or `null` when the text is not a stat file
 */
function identityFromStat(stat,) {
  // The command field can hold spaces and parentheses, so the fields that matter
  // start after the last ')'.
  const close = stat.lastIndexOf(')',);
  if (close === -1)
    return null;
  const fields = stat.slice(close + 2,).split(' ',);
  const startTicks = Number(fields[19],);
  return { state: fields[0], startTicks: Number.isFinite(startTicks,) ? startTicks : null, };
}

/**
 * Read one process's state and start tick.
 *
 * @param {number} pid
 * @returns {{ state: string, startTicks: number | null } | null | undefined} The identity; `undefined` when no such process exists, `null` when it exists but cannot be read
 */
function processIdentityOf(pid,) {
  if (!Number.isInteger(pid,) || pid <= 0)
    return undefined;
  if (!procStatAvailable()) {
    try {
      process.kill(pid, 0,);
      return { state: '?', startTicks: null, };
    }
    catch (error) {
      // EPERM means the process is there and belongs to someone else.
      if (error.code === 'EPERM')
        return { state: '?', startTicks: null, };
      return undefined;
    }
  }
  let stat;
  try {
    stat = readFileSync(`/proc/${pid}/stat`, { encoding: 'utf8', },);
  }
  catch (error) {
    if (error.code === 'ENOENT' || error.code === 'ESRCH')
      return undefined;
    // EACCES under `hidepid`: the process may well be alive, so its lock is
    // never taken over.
    void error;
    return null;
  }
  return identityFromStat(stat,);
}

/**
 * This process's own start tick, read once and reused for every lock it takes.
 *
 * @returns {number | null}
 */
function ownStartTicks() {
  if (cachedOwnStartTicks === undefined) {
    try {
      cachedOwnStartTicks = identityFromStat(readFileSync('/proc/self/stat', { encoding: 'utf8', },),)?.startTicks ?? null;
    }
    catch (error) {
      // Without `/proc` a record carries the pid alone, and this shim then says so
      // in what it proves.
      void error;
      cachedOwnStartTicks = null;
    }
  }
  return cachedOwnStartTicks;
}

/**
 * The record this process writes into a lock file it takes.
 *
 * The nonce is what makes one holding distinct from another holding by the same
 * process: a release callback left over from an earlier holding sees a different
 * nonce in the lock it did not write, so it removes nothing.
 *
 * @returns {{ format: string, pid: number, startTicks: number | null, boot: string | null, nonce: string }}
 */
function selfRecord() {
  return {
    format: RECORD_FORMAT,
    pid: process.pid,
    startTicks: ownStartTicks(),
    boot: bootIdentity() ?? null,
    nonce: randomUUID(),
  };
}

/**
 * Parse the record of a lock file.
 *
 * @param {string} text
 * @returns {{ pid: number, startTicks: number | null, boot: string | null, nonce: string | null } | undefined} The record, or `undefined` when this shim did not write the text
 */
function parseRecord(text,) {
  let parsed;
  try {
    parsed = JSON.parse(text,);
  }
  catch (error) {
    // Anything that is not JSON is not a record this shim wrote.
    void error;
    return undefined;
  }
  if (parsed === null || typeof parsed !== 'object')
    return undefined;
  if (parsed.format !== RECORD_FORMAT)
    return undefined;
  if (!Number.isInteger(parsed.pid,) || parsed.pid <= 0)
    return undefined;
  return {
    pid: parsed.pid,
    startTicks: typeof parsed.startTicks === 'number' ? parsed.startTicks : null,
    boot: typeof parsed.boot === 'string' ? parsed.boot : null,
    nonce: typeof parsed.nonce === 'string' ? parsed.nonce : null,
  };
}

/**
 * Whether two records describe the same holding.
 *
 * @param {{ pid: number, startTicks: number | null, boot: string | null, nonce: string | null } | undefined} left
 * @param {{ pid: number, startTicks: number | null, boot: string | null, nonce: string | null } | undefined} right
 * @returns {boolean}
 */
function sameRecord(left, right,) {
  if (left === undefined || right === undefined)
    return false;
  return left.pid === right.pid
    && left.startTicks === right.startTicks
    && left.boot === right.boot
    && left.nonce === right.nonce;
}

/**
 * Whether the process a record names is gone. This is the proof that a lock may
 * be taken over: the named process no longer exists, and a later process that
 * holds the same pid does not match the recorded start tick.
 *
 * @param {{ pid: number, startTicks: number | null, boot: string | null }} record
 * @returns {boolean}
 */
function holderIsGone(record,) {
  const currentBoot = bootIdentity();
  if (record.boot !== null && currentBoot !== undefined && record.boot !== currentBoot)
    return true;
  const identity = processIdentityOf(record.pid,);
  if (identity === undefined)
    return true;
  if (identity === null)
    return false;
  if (DEAD_PROCESS_STATES.has(identity.state,))
    return true;
  if (record.startTicks === null || identity.startTicks === null)
    return false;
  return identity.startTicks !== record.startTicks;
}

//endregion Process identity

//region Reading and writing lock files

/**
 * Create `path` holding `record`, atomically and completely: the record is
 * written to a temporary sibling first and then hard-linked onto `path`, so no
 * reader ever sees a lock file without its record, and of two callers racing for
 * one path exactly one wins.
 *
 * @param {{ path: string, record: object }} params
 * @returns {boolean} `true` when this call created the path, `false` when another caller holds it
 * @throws Rethrows every filesystem failure other than the path being taken
 */
function createExclusiveFile({ path, record, },) {
  temporaryCounter += 1;
  const temporary = `${path}.${process.pid}-${temporaryCounter}${TEMPORARY_SUFFIX}`;
  writeFileSync(temporary, `${JSON.stringify(record,)}\n`, { encoding: 'utf8', flag: 'wx', mode: 0o600, },);
  let created = true;
  try {
    linkSync(temporary, path,);
  }
  catch (error) {
    if (error.code !== 'EEXIST') {
      removeTemporaryRecord(temporary,);
      throw error;
    }
    created = false;
  }
  removeTemporaryRecord(temporary,);
  return created;
}

/**
 * Remove the temporary record a link attempt wrote.
 *
 * @param {string} temporary
 */
function removeTemporaryRecord(temporary,) {
  try {
    unlinkSync(temporary,);
  }
  catch (error) {
    // A leftover record costs one inode and must never replace the real answer of
    // the lock call, so its removal failure is dropped here on purpose.
    void error;
  }
}

/**
 * What is at a lock path, and whether its holder can be named.
 *
 * `held` and `gone` carry the record of the process that wrote the lock; the
 * other shapes are locks this shim did not write, which are judged by age
 * because nothing in them names a holder.
 *
 * @param {string} lockPath
 * @returns {{ shape: 'absent' | 'held' | 'gone' | 'unreadable' | 'directory' | 'other', ageMs?: number, record?: { pid: number, startTicks: number | null, boot: string | null }, code?: string }} The inspection
 */
function inspectLock(lockPath,) {
  const stats = lstatSync(lockPath, { throwIfNoEntry: false, },);
  if (stats === undefined)
    return { shape: 'absent', };
  const identity = { ino: stats.ino, dev: stats.dev, };
  const ageMs = Date.now() - stats.mtimeMs;
  if (stats.isDirectory())
    return { shape: 'directory', ageMs, ...identity, };
  if (!stats.isFile())
    return { shape: 'other', ageMs, ...identity, };
  let text;
  try {
    text = readFileSync(lockPath, { encoding: 'utf8', },);
  }
  catch (error) {
    if (error.code === 'ENOENT')
      return { shape: 'absent', };
    return { shape: 'unreadable', ageMs, code: error.code, ...identity, };
  }
  const record = parseRecord(text,);
  if (record === undefined)
    return { shape: 'unreadable', ageMs, ...identity, };
  return { shape: holderIsGone(record,) ? 'gone' : 'held', ageMs, record, ...identity, };
}

/**
 * Whether two inspections describe the same lock, so a takeover acts only on the
 * lock it judged dead.
 *
 * @param {{ shape: string, record?: object }} left
 * @param {{ shape: string, record?: object }} right
 * @returns {boolean}
 */
function sameLockState(left, right,) {
  if (left.shape === 'absent' || right.shape === 'absent')
    return false;
  if (left.record !== undefined && right.record !== undefined)
    return sameRecord(left.record, right.record,);
  return left.shape === right.shape;
}

/**
 * The record at a lock path, for the ownership questions that only compare records: what is holding, what is
 * releasing, and what holds the takeover record.
 * Cheaper than {@link inspectLock}, because it never asks the kernel whether a holder is alive.
 *
 * @param {string} lockPath
 * @returns {{ present: boolean, record?: { pid: number, startTicks: number | null, boot: string | null } }} What is at the path; `record` is present only when this shim wrote it
 */
function lockRecordAt(lockPath,) {
  const stats = lstatSync(lockPath, { throwIfNoEntry: false, },);
  if (stats === undefined)
    return { present: false, };
  if (!stats.isFile())
    return { present: true, };
  let text;
  try {
    text = readFileSync(lockPath, { encoding: 'utf8', },);
  }
  catch (error) {
    return { present: error.code !== 'ENOENT', };
  }
  return { present: true, record: parseRecord(text,), };
}

/**
 * Remove one path only if it is still the entry that was judged dead: move it
 * aside under a name of this caller's own, then check the entry that was actually
 * moved.
 *
 * Moving first is what makes the check worth making: the entry examined is the
 * entry acted on, so a lock that another caller replaced in the meantime is
 * moved back instead of deleted.
 * A moved entry that is not the judged one is put back with a hard link, which is
 * the one restore POSIX offers for anything that is not a directory; callers pass
 * only entries that are not directories.
 *
 * @param {{ path: string, expected: { ino?: number, dev?: number, record?: object } }} params
 * @returns {boolean} `true` when the judged entry was removed, `false` when the path no longer held it
 */
function removeJudgedEntry({ path, expected, },) {
  temporaryCounter += 1;
  const aside = `${path}.${process.pid}-${temporaryCounter}.removed`;
  try {
    renameSync(path, aside,);
  }
  catch (error) {
    // The path changed under this caller, so there is nothing it may remove.
    if (error.code === 'ENOENT')
      return false;
    throw error;
  }
  const moved = lstatSync(aside, { throwIfNoEntry: false, },);
  // Record inside the entry that was moved, when there is one to read.
  const movedRecord = moved === undefined || !moved.isFile()
    ? undefined
    : parseRecord(readFileSync(aside, { encoding: 'utf8', },),);
  const sameEntry = moved !== undefined && (moved.ino === expected.ino) && (moved.dev === expected.dev);
  const matched = sameEntry && (expected.record === undefined || sameRecord(movedRecord, expected.record,));
  if (!matched) {
    try {
      linkSync(aside, path,);
    }
    catch (error) {
      // Another caller arrived while this one held the entry aside. It owns the
      // lock now, and the caller whose entry could not be restored learns of it
      // from its own release or ownership check.
      void error;
    }
  }
  removeTemporaryRecord(aside,);
  return matched;
}

/**
 * Remove a lock path, whether it is a record file or a directory an earlier
 * version of this shim left.
 *
 * @param {string} lockPath
 * @param {{ shape: string }} inspection
 * @throws Error with `code === 'ELOCKED'` when the path is a directory holding entries, which this shim will not delete
 */
function removeLockPath(lockPath, inspection,) {
  try {
    if (inspection.shape === 'directory')
      rmdirSync(lockPath,);
    else
      unlinkSync(lockPath,);
  }
  catch (error) {
    if (error.code === 'ENOENT')
      return;
    if (error.code === 'ENOTEMPTY' || error.code === 'EEXIST') {
      const blocked = new Error(
        `A lock directory this shim did not write holds entries and was not removed (lock: ${lockPath});`
        + ' remove it by hand once no pi run is using it',
      );
      blocked.code = 'ELOCKED';
      throw blocked;
    }
    throw error;
  }
}

/**
 * The error every caller of a lock this shim will not take already expects.
 *
 * @param {string} lockPath
 * @returns {Error} An error with `code === 'ELOCKED'` naming the lock
 */
function lockedError(lockPath,) {
  const error = new Error(`Lock file is already being held (lock: ${lockPath})`,);
  error.code = 'ELOCKED';
  return error;
}

//endregion Reading and writing lock files

//region Taking over a dead holder's lock

/**
 * How old a lock nobody can be named for must be before it is taken over.
 *
 * @param {{ stale?: number } | undefined} options
 * @returns {number} Milliseconds
 */
function unownedLockGraceMs(options,) {
  const stale = options?.stale;
  if (typeof stale === 'number' && stale > 0)
    return stale;
  return DEFAULT_UNOWNED_LOCK_GRACE_MS;
}

/**
 * Whether the lock an inspection found may be taken over: a lock whose recorded
 * holder is proven gone at once, and a lock nobody can be named for only once it
 * is older than the grace, because a live holder may still be inside the step
 * that names it.
 *
 * @param {{ shape: string, ageMs?: number }} inspection
 * @param {{ stale?: number } | undefined} options
 * @returns {boolean}
 */
function mayTakeOver(inspection, options,) {
  if (inspection.shape === 'gone')
    return true;
  if (inspection.shape === 'held')
    return false;
  return (inspection.ageMs ?? 0) > unownedLockGraceMs(options,);
}

/**
 * Claim the right to replace one dead lock. Of two callers that find the same
 * dead lock in the same instant, exactly one creates this record, so exactly one
 * proceeds to remove the dead lock and only that one can end up holding it.
 *
 * @param {{ takeoverPath: string, record: object }} params
 * @returns {boolean} `true` when this caller may proceed
 */
function claimTakeover({ takeoverPath, record, },) {
  if (createExclusiveFile({ path: takeoverPath, record, },))
    return true;
  const existing = inspectLock(takeoverPath,);
  const abandoned = existing.shape === 'gone'
    || (existing.shape !== 'held' && existing.shape !== 'absent'
      && (existing.ageMs ?? 0) > DEFAULT_UNOWNED_LOCK_GRACE_MS);
  if (!abandoned)
    return false;
  // A taker that died while arbitrating leaves the record behind, and clearing it
  // is what lets the caller's own next attempt arbitrate. The clear is
  // conditional on the entry judged abandoned, so it can never take away the
  // record a live taker wrote in the meantime.
  if (existing.shape === 'directory')
    removeLockPath(takeoverPath, existing,);
  else if (!removeJudgedEntry({ path: takeoverPath, expected: existing, },))
    return false;
  return createExclusiveFile({ path: takeoverPath, record, },);
}

/**
 * Drop the takeover record this caller created, once the lock it was arbitrating
 * is settled.
 *
 * @param {{ takeoverPath: string, record: object }} params
 */
function dropTakeover({ takeoverPath, record, },) {
  const current = lockRecordAt(takeoverPath,);
  if (!sameRecord(current.record, record,))
    return;
  try {
    unlinkSync(takeoverPath,);
  }
  catch (error) {
    // An abandoned takeover record is cleared by the next taker once its own
    // record reads as gone, and the lock it guarded is already settled.
    void error;
  }
}

/**
 * Replace one lock whose holder is proven gone, under the takeover record that
 * makes this caller the only one attempting it.
 *
 * @param {{ lockPath: string, inspection: object, record: object }} params
 * @throws Error with `code === 'ELOCKED'` when another taker is at work, when the lock changed under this caller, or when another caller took the freed lock first
 */
function takeOver({ lockPath, inspection, record, },) {
  const takeoverPath = takeoverPathFor(lockPath,);
  if (!claimTakeover({ takeoverPath, record, },))
    throw lockedError(lockPath,);
  try {
    const current = inspectLock(lockPath,);
    if (!sameLockState(inspection, current,))
      throw lockedError(lockPath,);
    // A record file is removed only while it is still the entry judged dead, so a
    // lock another caller took in the meantime is put back rather than deleted.
    // A directory an earlier shim left has no record to compare, and its age was
    // the whole judgement.
    if (current.shape === 'directory')
      removeLockPath(lockPath, current,);
    else if (!removeJudgedEntry({ path: lockPath, expected: current, },))
      throw lockedError(lockPath,);
    if (!createExclusiveFile({ path: lockPath, record, },))
      throw lockedError(lockPath,);
  }
  finally {
    dropTakeover({ takeoverPath, record, },);
  }
}

//endregion Taking over a dead holder's lock

//region Holding, reporting a lost lock, and releasing

/**
 * Interval between two ownership checks of a held lock.
 *
 * @param {{ stale?: number, onCompromised?: (error: Error) => void } | undefined} options
 * @returns {number | undefined} Milliseconds, or `undefined` when the caller asked for no callback
 */
function compromiseCheckIntervalMs(options,) {
  if (typeof options?.onCompromised !== 'function')
    return undefined;
  const stale = options.stale;
  if (typeof stale !== 'number' || stale <= 0)
    return undefined;
  return Math.max(Math.floor(stale / 2,), MIN_COMPROMISE_CHECK_INTERVAL_MS,);
}

/**
 * Build the release callback of a lock this process now holds, and start
 * watching the lock while it is held so a holder learns through `onCompromised`
 * that its lock is no longer its own.
 *
 * @param {{ lockPath: string, record: object, options: { stale?: number, onCompromised?: (error: Error) => void } | undefined }} params
 * @returns {() => void} Release callback: removes the lock only while the record in it is this process's, tolerates the lock already being gone, and rethrows any other removal failure so a later call retries
 */
function beginHold({ lockPath, record, options, },) {
  const hold = { released: false, compromised: false, timer: undefined, };

  function stopWatch() {
    if (hold.timer !== undefined) {
      clearInterval(hold.timer,);
      hold.timer = undefined;
    }
  }

  function reportCompromise(reason,) {
    if (hold.compromised)
      return;
    hold.compromised = true;
    stopWatch();
    if (typeof options?.onCompromised !== 'function')
      return;
    const compromised = new Error(`${reason} (lock: ${lockPath})`,);
    compromised.code = 'ECOMPROMISED';
    options.onCompromised(compromised,);
  }

  function checkStillHeld() {
    const current = lockRecordAt(lockPath,);
    if (sameRecord(current.record, record,))
      return;
    hold.released = true;
    reportCompromise(current.present
      ? 'The lock this process held is now held by another process'
      : 'The lock this process held was removed',);
  }

  const intervalMs = compromiseCheckIntervalMs(options,);
  if (intervalMs !== undefined) {
    hold.timer = setInterval(checkStillHeld, intervalMs,);
    // The watch never keeps a process alive on its own.
    hold.timer.unref();
  }

  return function release() {
    if (hold.released)
      return;
    if (hold.compromised) {
      hold.released = true;
      return;
    }
    const current = lockRecordAt(lockPath,);
    if (!sameRecord(current.record, record,)) {
      hold.released = true;
      reportCompromise(current.present
        ? 'The lock this process held was taken over before it was released'
        : 'The lock this process held was removed before it was released',);
      return;
    }
    try {
      unlinkSync(lockPath,);
    }
    catch (error) {
      // Disappearance is the only removal failure to tolerate. EPERM, EACCES and
      // EBUSY leave the lock in place, so they surface here instead of reporting
      // a release that did not happen; `hold.released` stays false and the next
      // call retries.
      if (error.code !== 'ENOENT')
        throw error;
    }
    hold.released = true;
    stopWatch();
  };
}

/**
 * Acquire the lock, taking over a lock whose holder is proven gone.
 *
 * @param {string} target
 * @param {{ stale?: number, onCompromised?: (error: Error) => void } | undefined} options
 * @returns {() => void} Release callback
 * @throws Error with `code === 'ELOCKED'` when a live holder, or another taker of the same dead lock, holds it
 */
function acquire(target, options,) {
  const lockPath = lockPathFor(target,);
  const record = selfRecord();
  if (createExclusiveFile({ path: lockPath, record, },))
    return beginHold({ lockPath, record, options, },);
  const inspection = inspectLock(lockPath,);
  if (inspection.shape === 'absent') {
    // The holder released between this caller's link and its look.
    if (createExclusiveFile({ path: lockPath, record, },))
      return beginHold({ lockPath, record, options, },);
    throw lockedError(lockPath,);
  }
  if (!mayTakeOver(inspection, options,))
    throw lockedError(lockPath,);
  takeOver({ lockPath, inspection, record, },);
  return beginHold({ lockPath, record, options, },);
}

//endregion Holding, reporting a lost lock, and releasing

/**
 * Sync lock. Throws ELOCKED on first conflict; callers handle retries
 * themselves. `@earendil-works/pi-coding-agent`'s `auth-storage.js` (lines 45
 * to 62) and `settings-manager.js` (lines 117 to 143) both wrap this in their
 * own sync retry loop.
 *
 * A dead holder's lock is taken over inside this call, so the caller's first
 * attempt already succeeds where a crashed run left the lock behind.
 *
 * @param {string} target
 * @param {{ realpath?: boolean, stale?: number, onCompromised?: (error: Error) => void }} [_options] `realpath` is ignored: every caller passes `false`, so the target is treated as already resolved. `stale` bounds how long a lock nobody can be named for stands. `onCompromised` is called when the held lock stops being this process's.
 * @returns {() => void}
 * @example
 *   const release = lockSync('/tmp/auth.json', { realpath: false });
 *   // ... critical section ...
 *   release();
 */
function lockSync(target, options,) {
  return acquire(target, options,);
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
 * grows by `factor`. Upstream's `randomize` option is ignored; the backoff is
 * deterministic, and callers that pass `retries: 0` keep their own loop as the
 * only retry.
 *
 * @param {{ target: string, options: object | undefined, config: { retries: number, factor: number, minTimeout: number, maxTimeout: number }, attempt: number, timeout: number }} params
 * @returns {Promise<() => void>}
 */
async function attemptLock({ target, options, config, attempt, timeout, },) {
  const maxAttempts = config.retries + 1;
  try {
    return acquire(target, options,);
  }
  catch (err) {
    if (err.code === 'ELOCKED' && attempt < maxAttempts) {
      await sleep(timeout,);
      return attemptLock({
        target,
        options,
        config,
        attempt: attempt + 1,
        timeout: Math.min(timeout * config.factor, config.maxTimeout,),
      },);
    }
    throw err;
  }
}

/**
 * Async lock with internal retry.
 *
 * `stale` is the bound on how long a lock nobody can be named for stands, and
 * the interval of the ownership checks of a held lock is half of it; a lock
 * whose recorded holder is proven gone is taken over without waiting for any
 * bound. `onCompromised` is called while the lock is held as soon as it stops
 * being this process's, and again at release if it was taken from this process
 * before release ran. `realpath` is ignored, as on the sync path.
 *
 * The returned release callback is the sync function `acquire` produced, so
 * `await release()` in async callers resolves immediately because `await
 * undefined` is non-blocking.
 *
 * @param {string} target
 * @param {{ retries?: number | object, stale?: number, onCompromised?: (error: Error) => void, realpath?: boolean }} [options]
 * @returns {Promise<() => void>}
 * @example
 *   const release = await lock('/tmp/auth.json', {
 *     retries: 0,
 *     stale: 30000,
 *     onCompromised: (error) => { compromised = error; },
 *   });
 *   // ... critical section ...
 *   await release();
 */
async function lock(target, options,) {
  const config = normaliseRetries(options?.retries,);
  return attemptLock({
    target,
    options,
    config,
    attempt: 1,
    timeout: config.minTimeout,
  },);
}

module.exports = lock;
module.exports.lock = lock;
module.exports.lockSync = lockSync;
