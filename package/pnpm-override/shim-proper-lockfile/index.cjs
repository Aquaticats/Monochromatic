'use strict';

const {
  linkSync,
  lstatSync,
  readFileSync,
  readdirSync,
  renameSync,
  statSync,
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
/** The kernel's per-process stat line, whose fields 3 and 22 are the state and the start tick. */
const PROC_SELF_STAT = '/proc/self/stat';
/** This boot's identity, so a record from an earlier boot is recognisable. */
const PROC_BOOT_ID = '/proc/sys/kernel/random/boot_id';
/** This process's PID namespace, whose inode is the same number for every process in it. */
const PROC_SELF_PID_NAMESPACE = '/proc/self/ns/pid';

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
let cachedOwnPidNamespace;
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
      readFileSync(PROC_SELF_STAT, { encoding: 'utf8', },);
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
      cachedBootIdentity = readFileSync(PROC_BOOT_ID, { encoding: 'utf8', },).trim();
    }
    catch (error) {
      // A site that publishes no boot id loses only the reboot discrimination, and
      // a lock from another boot then reads as one this shim cannot judge.
      void error;
      cachedBootIdentity = '';
    }
  }
  return cachedBootIdentity === '' ? undefined : cachedBootIdentity;
}

/**
 * Inode of a PID namespace, the number every process inside it reports for it.
 * Two processes see the same number exactly when they share a namespace, which is
 * what lets a reader know whether a recorded pid names anything it can look up.
 *
 * `/proc/<pid>/ns/pid` is a symbolic link into `nsfs`, and the link's own inode is
 * a per-process procfs number that changes with every process, so the target is
 * examined rather than the link.
 *
 * @param {string} path - `/proc/<pid>/ns/pid`
 * @returns {number | undefined} The namespace inode, or `undefined` when it cannot be read
 */
function pidNamespaceAt(path,) {
  try {
    return statSync(path, { throwIfNoEntry: false, },)?.ino;
  }
  catch (error) {
    void error;
    return undefined;
  }
}

/**
 * This process's own PID namespace, read once and reused for every lock it takes.
 *
 * @returns {number | undefined}
 */
function ownPidNamespace() {
  if (cachedOwnPidNamespace === undefined)
    cachedOwnPidNamespace = pidNamespaceAt(PROC_SELF_PID_NAMESPACE,) ?? 0;
  return cachedOwnPidNamespace === 0 ? undefined : cachedOwnPidNamespace;
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
 * Read one process's state and start tick, as this process's PID namespace shows it.
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
      cachedOwnStartTicks = identityFromStat(readFileSync(PROC_SELF_STAT, { encoding: 'utf8', },),)?.startTicks ?? null;
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
 * @returns {{ format: string, pid: number, startTicks: number | null, boot: string | null, ns: number | null, nonce: string }}
 */
function selfRecord() {
  return {
    format: RECORD_FORMAT,
    pid: process.pid,
    startTicks: ownStartTicks(),
    boot: bootIdentity() ?? null,
    ns: ownPidNamespace() ?? null,
    nonce: randomUUID(),
  };
}

/**
 * Parse the record of a lock file.
 *
 * @param {string} text
 * @returns {{ pid: number, startTicks: number | null, boot: string | null, ns: number | null, nonce: string | null } | undefined} The record, or `undefined` when this shim did not write the text
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
    ns: Number.isInteger(parsed.ns,) ? parsed.ns : null,
    nonce: typeof parsed.nonce === 'string' ? parsed.nonce : null,
  };
}

/**
 * Whether two records describe the same holding.
 *
 * @param {{ pid: number, startTicks: number | null, boot: string | null, ns: number | null, nonce: string | null } | undefined} left
 * @param {{ pid: number, startTicks: number | null, boot: string | null, ns: number | null, nonce: string | null } | undefined} right
 * @returns {boolean}
 */
function sameRecord(left, right,) {
  if (left === undefined || right === undefined)
    return false;
  return left.pid === right.pid
    && left.startTicks === right.startTicks
    && left.boot === right.boot
    && left.ns === right.ns
    && left.nonce === right.nonce;
}

/**
 * What this process can tell about the holder a record names: that the process is
 * gone, that it is running, or that neither can be established from here.
 *
 * `gone` is the only answer a lock may be taken over on. It is reached two ways,
 * both of them about the named process and neither of them about the lock's age:
 * the record was written in an earlier boot, so that process cannot exist now, or
 * the record names a process of this PID namespace that no longer exists, has
 * exited, or whose number now belongs to a process started at another tick.
 *
 * @param {{ pid: number, startTicks: number | null, boot: string | null, ns: number | null }} record
 * @returns {{ state: 'gone' | 'alive' | 'unknown', why: string, reason: string }} The answer and why it was given
 */
function holderState(record,) {
  const currentBoot = bootIdentity();
  if (record.boot !== null && currentBoot !== undefined && record.boot !== currentBoot) {
    return {
      state: 'gone',
      why: 'Its record was written in an earlier boot, so the process it names cannot exist now.',
      reason: 'earlier-boot',
    };
  }
  if (record.ns === null) {
    return {
      state: 'unknown',
      why: 'Its record does not say which PID namespace its holder was in, so its process number cannot be looked up with confidence.',
      reason: 'no-namespace',
    };
  }
  const ownNamespace = ownPidNamespace();
  if (ownNamespace === undefined) {
    return {
      state: 'unknown',
      why: 'This process cannot read its own PID namespace, so it cannot tell whether the recorded process number is one it can look up.',
      reason: 'no-namespace-of-reader',
    };
  }
  if (record.ns !== ownNamespace) {
    return {
      state: 'unknown',
      why: `Its holder ran in PID namespace ${record.ns} and this process runs in PID namespace ${ownNamespace}, so the process number it records names nothing this process can look up.`,
      reason: 'other-namespace',
    };
  }
  const identity = processIdentityOf(record.pid,);
  if (identity === undefined) {
    return {
      state: 'gone',
      why: `The process it names, ${record.pid}, no longer exists.`,
      reason: 'process-gone',
    };
  }
  if (identity === null) {
    return {
      state: 'unknown',
      why: `The process it names, ${record.pid}, exists but cannot be inspected from here.`,
      reason: 'process-unreadable',
    };
  }
  if (DEAD_PROCESS_STATES.has(identity.state,)) {
    return {
      state: 'gone',
      why: `The process it names, ${record.pid}, has exited.`,
      reason: 'process-exited',
    };
  }
  if (record.startTicks === null || identity.startTicks === null) {
    return {
      state: 'unknown',
      why: `The start time of the process it names, ${record.pid}, cannot be compared with the one in the record.`,
      reason: 'start-time-unknown',
    };
  }
  if (identity.startTicks !== record.startTicks) {
    return {
      state: 'gone',
      why: `The process number ${record.pid} now belongs to a process started later than the holder this record names.`,
      reason: 'process-number-reused',
    };
  }
  return {
    state: 'alive',
    why: `The process it names, ${record.pid}, is running.`,
    reason: 'process-running',
  };
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
 * The bytes at a lock path, or the reason they could not be read.
 *
 * @param {string} lockPath
 * @returns {{ present: boolean, text?: string, readError?: Error }} What the read gave
 */
function lockTextAt(lockPath,) {
  try {
    return { present: true, text: readFileSync(lockPath, { encoding: 'utf8', },), };
  }
  catch (error) {
    if (error.code === 'ENOENT')
      return { present: false, };
    return { present: true, readError: error, };
  }
}

/**
 * One path written so a shell reads it as a single word, whatever bytes it holds.
 *
 * Single quotes make every byte between them literal, and are the only construct
 * that does: a space, a quote, a dollar sign, a backtick and a line break all pass
 * through untouched. A single quote is the one byte they cannot hold, so each one
 * closes the quotes, is escaped outside them, and reopens them.
 *
 * The encoding happens here, at the last interpolation before the text becomes a
 * shell command, and never earlier, so the path itself is never rewritten.
 *
 * @param {string} path
 * @returns {string} The path as a shell word
 */
function shellWord(path,) {
  return `'${path.replaceAll("'", "'\\''",)}'`;
}

/**
 * The command that clears one lock this shim will not take over, for the caller's
 * message to name. It runs as printed in a shell, which is the only way a person
 * reads it.
 *
 * @param {{ what: string, entries?: number }} inspection
 * @param {string} lockPath
 * @returns {string}
 */
function clearingCommand(inspection, lockPath,) {
  const word = shellWord(lockPath,);
  if (inspection.entries !== undefined)
    return inspection.entries === 0 ? `rmdir ${word}` : `rm -r ${word}`;
  return `rm -f ${word}`;
}

/**
 * What is at a lock path, and what this process can tell about its holder.
 *
 * Only `gone` may be taken over. `held` is a running holder. Every other answer is
 * `unknown`, carrying what was found and why it cannot be judged, because this
 * shim takes a lock over on proof and alone.
 *
 * @param {string} lockPath
 * @returns {{ shape: 'absent' } | { shape: 'held' | 'gone', record: object, ino: number, dev: number } | { shape: 'unknown', what: string, why: string, reason: string, command: string, ino?: number, dev?: number }} The inspection
 */
function inspectLock(lockPath,) {
  const stats = lstatSync(lockPath, { throwIfNoEntry: false, },);
  if (stats === undefined)
    return { shape: 'absent', };
  const identity = { ino: stats.ino, dev: stats.dev, };
  if (stats.isDirectory()) {
    let entries;
    try {
      entries = readdirSync(lockPath,).length;
    }
    catch (error) {
      const inspection = {
        shape: 'unknown',
        what: `a lock directory this process could not list (${String(error.code,)})`,
        why: 'Nothing in a lock directory names the process that made it.',
        reason: 'directory',
        ...identity,
      };
      return { ...inspection, command: clearingCommand(inspection, lockPath,), };
    }
    const inspection = {
      shape: 'unknown',
      what: entries === 0
        ? 'an empty lock directory that an earlier version of this shim wrote'
        : `a lock directory that holds ${String(entries,)} ${entries === 1 ? 'entry' : 'entries'}`,
      why: 'Nothing in a lock directory names the process that made it.',
      reason: 'directory',
      entries,
      ...identity,
    };
    return { ...inspection, command: clearingCommand(inspection, lockPath,), };
  }
  if (!stats.isFile()) {
    const inspection = {
      shape: 'unknown',
      what: 'an entry that is neither a lock file nor a lock directory',
      why: 'Nothing in it says which process, if any, is inside the lock.',
      reason: 'not-a-lock-file',
      ...identity,
    };
    return { ...inspection, command: clearingCommand(inspection, lockPath,), };
  }
  const read = lockTextAt(lockPath,);
  if (!read.present)
    return { shape: 'absent', };
  if (read.readError !== undefined) {
    const inspection = {
      shape: 'unknown',
      what: 'a lock file this process could not read',
      why: `The read failed with "${read.readError.message}", and a lock whose record cannot be read may still be held.`,
      reason: 'unreadable-record',
      ...identity,
    };
    return { ...inspection, command: clearingCommand(inspection, lockPath,), };
  }
  const record = parseRecord(read.text,);
  if (record === undefined) {
    const inspection = {
      shape: 'unknown',
      what: 'a lock file whose bytes are not a record this shim wrote',
      why: 'A machine that loses power can empty a record before its bytes reach the disk, and nothing else writes one.',
      reason: 'unwritten-record',
      ...identity,
    };
    return { ...inspection, command: clearingCommand(inspection, lockPath,), };
  }
  const holder = holderState(record,);
  if (holder.state === 'unknown') {
    const inspection = {
      shape: 'unknown',
      what: 'a lock file whose record this process cannot judge',
      why: holder.why,
      reason: holder.reason,
      ...identity,
    };
    return { ...inspection, command: clearingCommand(inspection, lockPath,), };
  }
  return { shape: holder.state === 'gone' ? 'gone' : 'held', record, ...identity, };
}

/**
 * Remove one path only if it is still the entry that was judged dead: move it
 * aside under a name of this caller's own, then check the entry that was actually
 * moved.
 *
 * Moving first is what makes the check worth making: the entry examined is the
 * entry acted on, so a lock that another caller replaced in the meantime is put
 * back instead of deleted.
 * A moved entry that is not the judged one is put back with a hard link, which is
 * the one restore POSIX offers.
 *
 * @param {{ path: string, expected: { ino: number, dev: number, record: object } }} params
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
  const movedRecord = readRecordOrUndefined(aside,);
  const sameEntry = moved !== undefined && (moved.ino === expected.ino) && (moved.dev === expected.dev);
  const matched = sameEntry && sameRecord(movedRecord, expected.record,);
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
 * The record in a file, when it has one that can be read.
 *
 * @param {string} path
 * @returns {object | undefined}
 */
function readRecordOrUndefined(path,) {
  const read = lockTextAt(path,);
  if (!read.present || read.readError !== undefined)
    return undefined;
  return parseRecord(read.text,);
}

/**
 * The record at a lock path, for the ownership questions that only compare
 * records: what is holding, what is releasing, and what holds the takeover
 * record. Cheaper than {@link inspectLock}, because it never asks the kernel
 * whether a holder is alive.
 *
 * `readError` is separate from `record: undefined` on purpose: a read that failed
 * says nothing about who holds the lock, and a holder must not be told it lost a
 * lock it still holds.
 *
 * @param {string} lockPath
 * @returns {{ present: boolean, record?: object, readError?: Error }} What is at the path
 */
function lockRecordAt(lockPath,) {
  const stats = lstatSync(lockPath, { throwIfNoEntry: false, },);
  if (stats === undefined)
    return { present: false, };
  if (!stats.isFile())
    return { present: true, };
  const read = lockTextAt(lockPath,);
  if (!read.present)
    return { present: false, };
  if (read.readError !== undefined)
    return { present: true, readError: read.readError, };
  return { present: true, record: parseRecord(read.text,), };
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
  error.reason = 'held';
  return error;
}

/**
 * The error for a lock this process cannot judge: what was found, why it cannot be
 * told whether a run is still inside the lock, and the one command that clears it.
 *
 * The code is `ELOCKED` because every caller in this workspace treats that code as
 * "try again within my own bound", which is what a record that could not be read
 * once deserves; the caller's bound, not this shim, is what ends the wait, and the
 * message survives to the caller's own error.
 *
 * @param {{ lockPath: string, inspection: object, context?: string }} params
 * @returns {Error} An error with `code === 'ELOCKED'` whose message names the command
 */
function unjudgeableError({ lockPath, inspection, context, },) {
  const sentences = [
    context ?? '',
    `The lock at ${lockPath} is ${inspection.what}.`,
    inspection.why,
    'This shim takes a lock over only when the process that wrote it is proven gone, and it cannot prove that here.',
    `If no run that uses this file is inside the lock, remove it and start again with: ${inspection.command}`,
  ];
  const error = new Error(sentences.filter(sentence => sentence !== '').join(' ',),);
  error.code = 'ELOCKED';
  error.reason = inspection.reason;
  error.lockPath = lockPath;
  error.remedy = inspection.command;
  return error;
}

//endregion Reading and writing lock files

//region Taking over a dead holder's lock

/**
 * Claim the right to replace one dead lock. Of two callers that find the same
 * dead lock in the same instant, exactly one creates this record, so exactly one
 * proceeds to remove the dead lock and only that one can end up holding it.
 *
 * @param {{ takeoverPath: string, lockPath: string, record: object }} params
 * @returns {boolean} `true` when this caller may proceed
 * @throws Error with `code === 'ELOCKED'` when another taker is at work right now, or when the record it left behind cannot be judged
 */
function claimTakeover({ takeoverPath, lockPath, record, },) {
  if (createExclusiveFile({ path: takeoverPath, record, },))
    return true;
  const existing = inspectLock(takeoverPath,);
  if (existing.shape === 'absent') {
    // The other taker finished between this caller's link and its look.
    return false;
  }
  if (existing.shape === 'held') {
    const error = new Error(`Another caller is taking this lock over right now (lock: ${lockPath})`,);
    error.code = 'ELOCKED';
    error.reason = 'takeover-in-progress';
    throw error;
  }
  if (existing.shape !== 'gone') {
    throw unjudgeableError({
      lockPath: takeoverPath,
      inspection: existing,
      context: `Another caller began taking the lock at ${lockPath} over and left its own record there, and that record is what cannot be judged.`,
    },);
  }
  // The taker that wrote this record is proven gone, so clearing it is what lets
  // this attempt arbitrate. The clear is conditional on the entry judged gone, so
  // it can never take away the record a live taker wrote in the meantime.
  if (!removeJudgedEntry({ path: takeoverPath, expected: existing, },))
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
  if (!claimTakeover({ takeoverPath, lockPath, record, },))
    throw lockedError(lockPath,);
  try {
    const current = inspectLock(lockPath,);
    if (current.shape !== 'gone' || !sameRecord(current.record, inspection.record,))
      throw lockedError(lockPath,);
    // The entry is removed only while it is still the one judged gone, so a lock
    // another caller took in the meantime is put back rather than deleted.
    if (!removeJudgedEntry({ path: lockPath, expected: current, },))
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
 * A read that fails says nothing about who holds the lock, so neither the watch
 * nor the release treats one as a loss: the watch says nothing, and the release
 * rethrows the read's error and stays un-released, so a later call retries.
 *
 * @param {{ lockPath: string, record: object, options: { stale?: number, onCompromised?: (error: Error) => void } | undefined }} params
 * @returns {() => void} Release callback: removes the lock only while the record in it is this process's, tolerates the lock already being gone, and rethrows any other failure so a later call retries
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
    if (current.readError !== undefined)
      return;
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
    if (current.readError !== undefined)
      throw current.readError;
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
 * Acquire the lock, taking it over only when the process that wrote it is proven
 * gone, and refusing with words when that cannot be told.
 *
 * @param {string} target
 * @param {{ stale?: number, onCompromised?: (error: Error) => void } | undefined} options
 * @returns {() => void} Release callback
 * @throws Error with `code === 'ELOCKED'`: naming the lock when a live holder has it, and naming the command that clears it when this process cannot judge it
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
  if (inspection.shape === 'held')
    throw lockedError(lockPath,);
  if (inspection.shape !== 'gone')
    throw unjudgeableError({ lockPath, inspection, },);
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
 * A lock whose holder is proven gone is taken over inside this call, so the
 * caller's first attempt already succeeds where a killed run left the lock
 * behind. A lock this process cannot judge is refused with an error naming the
 * command that clears it.
 *
 * @param {string} target
 * @param {{ realpath?: boolean, stale?: number, onCompromised?: (error: Error) => void }} [_options] `realpath` is ignored: every caller passes `false`, so the target is treated as already resolved. `stale` is the interval of the ownership check of a held lock. `onCompromised` is called when the held lock stops being this process's.
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
 * `onCompromised` is called while the lock is held as soon as it stops being this
 * process's, and the interval of that ownership check is half of `stale`.
 * `realpath` is ignored, as on the sync path.
 *
 * A lock whose holder is proven gone is taken over without waiting for any bound.
 * A lock this process cannot judge is refused with an error whose message names
 * the command that clears it, after the caller's own retries, which are the only
 * retries there are.
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
