import { spawn, spawnSync, type ChildProcess, } from 'node:child_process';
import { once, } from 'node:events';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  utimesSync,
  writeFileSync,
} from 'node:fs';
import { mkdtemp, readFile, rm, } from 'node:fs/promises';
import { basename, dirname, join, resolve, } from 'node:path';
import { setTimeout as sleep, } from 'node:timers/promises';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import lockfile from './index.cjs';

//region Fixtures

/** Retry budget for the async cases, small enough that an exhausted budget still reports quickly. */
const ASYNC_RETRIES = 2;
/** First backoff wait for the async cases, in milliseconds. */
const ASYNC_MIN_TIMEOUT_MS = 20;
/** How long a holder keeps the lock before the waiting `lock` call is allowed to succeed, in milliseconds. */
const HOLDER_RELEASE_DELAY_MS = 5;
/**
 Budget each case that reaches the file system gives its work, in milliseconds.

 This machine's home drive has stalled for many seconds at a time (143 s on a small write in the night of
 2026-11-14), and the test harness's own logger sink reports the same stall. A case that fails on a stall would fail
 on the machine rather than on the shim, so the budget is far above the work's own cost of a few milliseconds, and a
 stall beyond it still fails.
 */
const ASYNC_CASE_TIMEOUT_MS = 30_000;
/**
 Longest a case waits for a child process to leave its report, in milliseconds.

 This is half the case budget on purpose: a child that never reports must fail its own case with the child named, which
 the harness's case timeout cannot say. The whole suite, every child included, finishes in 0.3 to 1.1 s on this
 machine, so this is fifteen to fifty times the work it waits for.
 */
const CHILD_REPORT_TIMEOUT_MS = 15_000;
/** Longest a case waits for a killed child to be reaped, in milliseconds, so a process in uninterruptible sleep cannot hold the suite. */
const CHILD_EXIT_TIMEOUT_MS = 10_000;
/** How often a case looks again for a child's report or a compromise, in milliseconds. */
const POLL_MS = 10;
/** Lock conflict code the upstream contract defines. */
const HELD_CODE = 'ELOCKED';
/** Code of the error the shim hands `onCompromised` when a held lock stops being the holder's. */
const COMPROMISED_CODE = 'ECOMPROMISED';
/** The one file-system code the shim treats as a release that already happened. */
const MISSING_CODE = 'ENOENT';
/** The file-system code a removal reports when the directory no longer allows it. */
const DENIED_CODE = 'EACCES';
/** Creating an entry inside a directory needs write permission on the directory. */
const READ_ONLY_DIRECTORY_MODE = 0o500;
/** The mode a fixture directory needs back before its own removal. */
const WRITABLE_DIRECTORY_MODE = 0o700;
/** The mode a lock file has while its holder holds it, so a case can give a record its read access back. */
const OWN_LOCK_MODE = 0o600;
/** What a holder writes in its report once it holds the lock. */
const HELD_REPORT = 'held';
/** The `stale` bound `@earendil-works/pi-coding-agent` passes on its async lock path, in milliseconds. */
const CALLER_STALE_MS = 30_000;
/** A `stale` small enough that the ownership watch of a held lock reports within one case, in milliseconds. */
const SHORT_STALE_MS = 500;
/** How long a case pretends a lock has been held, in milliseconds: long enough that no bound in the caller's hands matters. */
const AGED_LOCK_MS = 10 * 60 * 1_000;
/** A boot id no boot has, standing in for the boot a record was written in before a machine restarted. */
const EARLIER_BOOT_ID = '00000000-0000-4000-8000-000000000000';
/** A dollar sign, named so a fragment can hold one without this file reading it as a variable. */
const DOLLAR_SIGN = '$';
/** The system path a namespaced holder is started with, because `spawn` looks its program up in the child's own PATH. */
const SYSTEM_PATH = '/usr/local/bin:/usr/bin:/bin';
/** Longest the shim may take to refuse a lock it cannot judge, far under the thirty seconds the caller's own loop allows. */
const NO_WAIT_BOUND_MS = 5_000;
/** The mode that makes a lock file unreadable to its own owner, standing in for a read that fails on a stalled drive. */
const UNREADABLE_FILE_MODE = 0o000;
/** The `unshare` options that give a holder a PID namespace of its own, with its own private mount table. */
const OWN_PID_NAMESPACE_OPTIONS = [
  '--user',
  '--map-root-user',
  '--mount',
  '--propagation',
  'private',
  '--pid',
  '--fork',
  '--mount-proc',
];
/** How many rounds the two-taker race runs, so one unlucky interleaving cannot pass as a rule. */
const TAKER_ROUNDS = 6;
/** How long before the shared barrier both racing takers are started, in milliseconds. */
const TAKER_BARRIER_LEAD_MS = 150;
/** Budget the round-by-round race gives its child processes, in milliseconds. */
const TAKER_CASE_TIMEOUT_MS = 120_000;

/** Absolute path of the shim the holder processes load, so they run this package's code and no installed copy. */
const SHIM_PATH = fileURLToPath(new URL('index.cjs', import.meta.url,),);

/**
 Source of every holder process.
 It takes the lock, reports in a file, and stays alive until its case ends it, so a case can tell a live holder
 from one that is gone.
 A report is a file rather than a stream so a case never has to parse a child's output.
 */
const HOLDER_SOURCE = [
  "const { writeFileSync } = require('node:fs');",
  "const reportPath = process.env['REPORT_PATH'];",
  "writeFileSync(reportPath + '.started', '1');",
  "const lockfile = require(process.env['SHIM_PATH']);",
  'try {',
  "  const release = lockfile.lockSync(process.env['LOCK_TARGET'], { realpath: false });",
  '  // Never released: the case ends this process instead, which is what leaves the lock behind.',
  "  writeFileSync(reportPath, 'held');",
  '  // A timer, never a busy loop: the process only has to stay alive.',
  '  setInterval(() => {}, 1000);',
  '  void release;',
  '}',
  'catch (error) {',
  String.raw`  writeFileSync(reportPath, 'refused:' + error.code);`,
  '}',
].join('\n',);

/**
 Source of a process that races another one for a lock at a shared instant.
 It reports in a file, either the held report or the code it was refused with, and a taker that gets the lock keeps
 it until its case ends it, so a round can only ever show one winner.
 */
const TAKER_SOURCE = [
  "const { writeFileSync } = require('node:fs');",
  "writeFileSync(process.env['REPORT_PATH'] + '.started', '1');",
  "const lockfile = require(process.env['SHIM_PATH']);",
  "const reportPath = process.env['REPORT_PATH'];",
  "const barrier = Number(process.env['BARRIER_AT']);",
  'setTimeout(() => {',
  '  let release;',
  '  try {',
  "    release = lockfile.lockSync(process.env['LOCK_TARGET'], { retries: 0 });",
  '  }',
  '  catch (error) {',
  '    writeFileSync(reportPath, String(error.code));',
  '    process.exit(0);',
  '  }',
  "  writeFileSync(reportPath, 'held');",
  '  // Never released: a winner holds until its case ends this process.',
  '  setInterval(() => {}, 1000);',
  '  void release;',
  '}, Math.max(0, barrier - Date.now()));',
].join('\n',);

/**
 Source of a holder of the lock the previous version of this shim made: a directory with nothing in it, so nothing
 in the lock names the process holding it.
 */
const OLD_HOLDER_SOURCE = [
  "const { mkdirSync, writeFileSync } = require('node:fs');",
  "writeFileSync(process.env['REPORT_PATH'] + '.started', '1');",
  'mkdirSync(process.env.LOCK_PATH);',
  "writeFileSync(process.env.REPORT_PATH, 'held');",
  '// Never released: the holder keeps the directory until its case ends this process.',
  'setInterval(() => {}, 1000);',
].join('\n',);

/**
 Source of a holder that takes the lock inside a PID namespace of its own, the way a container sharing this machine's
 filesystem does.
 */
const NAMESPACED_HOLDER_SOURCE = [
  "const { writeFileSync } = require('node:fs');",
  "writeFileSync(process.env['REPORT_PATH'] + '.started', '1');",
  "const lockfile = require(process.env['SHIM_PATH']);",
  "const release = lockfile.lockSync(process.env['LOCK_TARGET'], { realpath: false });",
  "writeFileSync(process.env.REPORT_PATH, 'held');",
  'setInterval(() => {}, 1000);',
  'void release;',
].join('\n',);

/**
 Scratch directory holding one lockable target, removed with whatever lock state the case left behind once the
 declaring scope ends.
 */
type ScratchTarget = {
  readonly root: string;
  readonly target: string;
  readonly lockPath: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A holder process that took a lock in a case, ended when the declaring scope ends so no child outlives it.
 */
type Holder = {
  readonly child: ChildProcess;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Directory the shim locks, recomputed from the naming convention this package's README documents instead of read back
 from the implementation, so a case can observe the path the caller is promised.

 @param target - file the lock guards

 @returns the lock path
 */
function lockPathFor(target: string,): string {
  const absolute = resolve(target,);
  return resolve(dirname(absolute,), `.${basename(absolute,)}.lock`,);
}

/**
 Path of the takeover record that arbitrates which caller may replace one dead lock, from the same README convention.

 @param lockPath - lock path the record arbitrates

 @returns the takeover record path
 */
function takeoverPathFor(lockPath: string,): string {
  return `${lockPath}.takeover`;
}

/**
 Creates a scratch directory with one lockable target file.

 @param fragment - bytes the scratch directory's own name carries, so a case can put them in the lock's path

 @returns the fixture; declare it with `await using` so the scratch directory is removed at the end of the scope
 */
async function makeTarget(fragment = 'shim-proper-lockfile',): Promise<ScratchTarget> {
  const root = await mkdtemp(join(scratchRoot(), `${fragment}-`,),);
  const target = join(root, 'auth.json',);
  writeFileSync(target, '{}',);
  return {
    root,
    target,
    lockPath: lockPathFor(target,),
    async [Symbol.asyncDispose]() {
      chmodSync(root, WRITABLE_DIRECTORY_MODE,);
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Plants one unjudgeable lock of each shape a person may have to clear in one scratch directory whose name carries the
 awkward bytes, runs each command the shim prints through a shell exactly as printed, and requires each lock gone.

 The three shapes are the three commands: an empty lock directory, a lock directory holding an entry, and a lock file
 whose bytes are not a record.

 @param fragment - bytes the scratch directory's own name carries
 */
async function expectPrintedCommandsToClear(fragment: string,): Promise<void> {
  await using fixture = await makeTarget(fragment,);
  const shapes: readonly { readonly name: string; readonly verb: string; readonly plant: (lockPath: string,) => void }[] = [
    {
      name: 'an empty lock directory',
      verb: 'rmdir',
      plant: (lockPath,) => mkdirSync(lockPath,),
    },
    {
      name: 'a lock directory holding an entry',
      verb: 'rm -r',
      plant: (lockPath,) => {
        mkdirSync(lockPath,);
        writeFileSync(join(lockPath, 'holder-note',), 'still here',);
      },
    },
    {
      name: 'a lock file whose bytes are not a record',
      verb: 'rm -f',
      plant: (lockPath,) => writeFileSync(lockPath, '',),
    },
  ];

  for (const shape of shapes) {
    const target = join(fixture.root, `${shape.verb}.json`,);
    writeFileSync(target, '{}',);
    const lockPath = lockPathFor(target,);
    shape.plant(lockPath,);

    const thrown = caughtFrom(() => {
      lockfile.lockSync(target,);
    },);
    const refused = expectUnjudgeable(thrown, { lockPath, verb: shape.verb, },);
    // The lock is there, and the command as printed is what clears it.
    expect(existsSync(lockPath,),).toBe(true,);
    const printed = runAsPrinted(refused.remedy,);
    expect(printed.stderr,).toBe('',);
    expect(printed.ok,).toBe(true,);
    expect(existsSync(lockPath,),).toBe(false,);
  }
}

/**
 Calls `call` and hands back what it threw, so each case asserts on the error itself instead of on a wrapper.

 @param call - work expected to throw

 @returns what it threw
 */
function caughtFrom(call: () => void,): unknown {
  try {
    call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Awaits `call` and hands back what it rejected with, so each async case asserts on the error itself.

 @param call - work expected to reject

 @returns what it rejected with
 */
async function rejectionFrom(call: () => Promise<unknown>,): Promise<unknown> {
  try {
    await call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Reads a child's report file, saying whether it is there.

 The read is asynchronous, so a stat on this machine's drive, which stalls, yields to the rest of the suite instead of
 blocking the timers of every other case in this one process.

 @param path - file to read

 @returns whether it exists, and its text when it does
 */
async function readReportFile(path: string,): Promise<{ readonly present: boolean; readonly text: string }> {
  try {
    return { present: true, text: await readFile(path, 'utf8',), };
  }
  catch (error) {
    if (isMissingFileError(error,))
      return { present: false, text: '', };
    throw error;
  }
}

/**
 Whether a failure is a file that is not there, told apart from every other read failure.

 @param error - what a read threw

 @returns `true` when the operating system reported that the file does not exist
 */
function isMissingFileError(error: unknown,): boolean {
  if (!Error.isError(error,))
    return false;
  return ('code' in error) && (error.code === MISSING_CODE);
}

/**
 Waits until a child process leaves its report, and reads it.
 The wait is bounded, so a child that never reports fails its own case instead of hanging the suite, and its failure
 names the child and whether it began at all: every child writes `<report>.started` before it loads the shim.
 An empty report file is treated as absent: a reader can see a report file between the write that creates it and the
 write that fills it, and only a non-empty file is a report.

 @param reportPath - file the child writes its report in

 @param child - the child writing it, named in the failure

 @param deadline - epoch milliseconds after which the case stops waiting

 @returns the report text
 */
async function reportOfChild({ reportPath, child, deadline, }: {
  readonly reportPath: string;
  readonly child: ChildProcess;
  readonly deadline: number;
},): Promise<string> {
  const report = await readReportFile(reportPath,);
  if (report.present && (report.text !== ''))
    return report.text;
  if (Date.now() >= deadline) {
    const began = (await readReportFile(`${reportPath}.started`,)).present;
    const reached = began ? 'began and did not report' : 'never began';
    const waited = `the child process ${String(child.pid,)} ${reached} within ${String(CHILD_REPORT_TIMEOUT_MS,)} ms`;
    throw new Error(`${waited}; its report is ${reportPath}`,);
  }
  await sleep(POLL_MS,);
  return await reportOfChild({ reportPath, child, deadline, },);
}

/**
 Whether a child process already ended, so ending it again is a no-op.

 @param child - the child to inspect

 @returns `true` when the process exited or was signalled
 */
function childFinished(child: ChildProcess,): boolean {
  return (child.exitCode !== null) || (child.signalCode !== null);
}

/**
 Ends a child process and reaps it.

 @param child - the child to kill
 */
async function killChild(child: ChildProcess,): Promise<void> {
  if (childFinished(child,))
    return;
  const exited = once(child, 'exit', { signal: AbortSignal.timeout(CHILD_EXIT_TIMEOUT_MS,), },);
  child.kill('SIGKILL',);
  try {
    await exited;
  }
  catch (error) {
    throw new Error(
      `the child process ${String(child.pid,)} did not end within ${String(CHILD_EXIT_TIMEOUT_MS,)} ms of SIGKILL`,
      { cause: error, },
    );
  }
}

/**
 Starts a real holder process that locks `target` and stays alive, and ends it when its scope ends.

 @param target - file the holder locks

 @param reportPath - file the holder reports in

 @returns the holder, already holding
 */
async function startHolder(target: string, reportPath: string,): Promise<Holder> {
  const child = spawn(process.execPath, ['--input-type=commonjs', '-e', HOLDER_SOURCE,], {
    env: { SHIM_PATH, LOCK_TARGET: target, REPORT_PATH: reportPath, },
    stdio: ['ignore', 'ignore', 'ignore',],
  },);
  const holder: Holder = {
    child,
    async [Symbol.asyncDispose]() {
      await killChild(child,);
    },
  };
  const report = await reportOfChild({
    reportPath,
    child,
    deadline: Date.now() + CHILD_REPORT_TIMEOUT_MS,
  },);
  if (report !== HELD_REPORT) {
    await killChild(child,);
    throw new Error(`a holder process reported ${report} instead of holding`,);
  }
  return holder;
}

/**
 Starts a real holder of the lock the previous version of this shim made: an empty directory, with nothing in it that
 names the process holding it.

 @param lockPath - the lock directory the holder makes

 @param reportPath - file the holder reports in

 @returns the holder, already holding
 */
async function startOldStyleHolder(lockPath: string, reportPath: string,): Promise<Holder> {
  const child = spawn(process.execPath, ['--input-type=commonjs', '-e', OLD_HOLDER_SOURCE,], {
    env: { LOCK_PATH: lockPath, REPORT_PATH: reportPath, },
    stdio: ['ignore', 'ignore', 'ignore',],
  },);
  const holder: Holder = {
    child,
    async [Symbol.asyncDispose]() {
      await killChild(child,);
    },
  };
  const report = await reportOfChild({
    reportPath,
    child,
    deadline: Date.now() + CHILD_REPORT_TIMEOUT_MS,
  },);
  if (report !== HELD_REPORT) {
    await killChild(child,);
    throw new Error(`an old-style holder process reported ${report} instead of holding`,);
  }
  return holder;
}

/**
 Starts a real holder that takes the lock inside a PID namespace of its own, so the process number it records means
 nothing in this process's namespace.

 @param target - file the holder locks

 @param reportPath - file the holder reports in

 @returns the holder, already holding
 */
async function startHolderInOwnPidNamespace(target: string, reportPath: string,): Promise<Holder> {
  const child = spawn('unshare', [
    ...OWN_PID_NAMESPACE_OPTIONS,
    process.execPath,
    '--input-type=commonjs',
    '-e',
    NAMESPACED_HOLDER_SOURCE,
  ], {
    env: {
      PATH: SYSTEM_PATH,
      SHIM_PATH,
      LOCK_TARGET: target,
      REPORT_PATH: reportPath,
    },
    stdio: ['ignore', 'ignore', 'inherit',],
  },);
  const holder: Holder = {
    child,
    async [Symbol.asyncDispose]() {
      await killChild(child,);
    },
  };
  const report = await reportOfChild({
    reportPath,
    child,
    deadline: Date.now() + CHILD_REPORT_TIMEOUT_MS,
  },);
  if (report !== HELD_REPORT) {
    await killChild(child,);
    throw new Error(`a namespaced holder process reported ${report} instead of holding`,);
  }
  return holder;
}

/**
 Asserts a lock call was refused because the lock cannot be judged: the caller's own code, the lock path, and the one
 command that clears it, all in the words the caller sees.

 @param thrown - what the call threw

 @param expected - the lock path, and the verb the clearing command starts with

 @returns what was thrown, so a case can run the command it carries
 */
function expectUnjudgeable(
  thrown: unknown,
  expected: { readonly lockPath: string; readonly verb: string },
): Error & { readonly remedy: string } {
  expect(thrown,).toHaveProperty('code', HELD_CODE,);
  const refused = thrown as Error & { readonly remedy: string };
  const { message, remedy, } = refused;
  expect(message,).toContain(expected.lockPath,);
  expect(remedy.startsWith(`${expected.verb} `),).toBe(true,);
  // The words carry the very command, so a person can copy it from them.
  expect(message,).toContain(remedy,);
  expect(message,).toContain('proven gone',);
  expect(message,).not.toContain('Lock file is already being held',);
  return refused;
}

/**
 Runs a clearing command through a shell exactly as the shim printed it, the way a person who copies it does.

 @param command - the command as printed

 @returns whether the shell reported success, and what it wrote on stderr
 */
function runAsPrinted(command: string,): { readonly ok: boolean; readonly stderr: string } {
  const done = spawnSync('/bin/sh', ['-c', command,], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe',], },);
  return { ok: (done.status === 0) && (done.error === undefined), stderr: done.stderr ?? '', };
}

/**
 Leaves a lock at `target` the way a run killed inside the locked section leaves one: a real holder process takes
 it and is killed without releasing.

 @param root - scratch directory the target lives in

 @param target - file the dead run locked
 */
async function leaveLockOfAKilledHolder(root: string, target: string,): Promise<void> {
  const reportPath = join(root, `dead-holder-${basename(target,)}.report`,);
  const holder = await startHolder(target, reportPath,);
  await killChild(holder.child,);
}

/**
 One racing taker: its process, and the report it left.
 */
type Taker = {
  readonly child: ChildProcess;
  readonly report: string;
};

/**
 Runs one racing taker, which locks at the shared barrier instant and reports the held report or the code it got.

 @param target - file the taker locks

 @param barrierAt - epoch milliseconds both takers wait for

 @param reportPath - file the taker reports in

 @returns the taker, holding when it won and already gone when it was refused
 */
async function raceFor(target: string, barrierAt: number, reportPath: string,): Promise<Taker> {
  const child = spawn(process.execPath, ['--input-type=commonjs', '-e', TAKER_SOURCE,], {
    env: {
      SHIM_PATH,
      LOCK_TARGET: target,
      BARRIER_AT: String(barrierAt,),
      REPORT_PATH: reportPath,
    },
    stdio: ['ignore', 'ignore', 'ignore',],
  },);
  const report = await reportOfChild({
    reportPath,
    child,
    deadline: Date.now() + CHILD_REPORT_TIMEOUT_MS,
  },);
  return { child, report, };
}

/**
 Waits until a case has seen the compromise report it is watching for, or its deadline passes.
 Waiting by recursion rather than by a loop keeps this file clear of awaited loop bodies.

 @param seen - reports the holder's callback was given

 @param deadline - epoch milliseconds after which the case stops waiting
 */
async function reportedBy(seen: Error[], deadline: number,): Promise<void> {
  if (seen.length > 0)
    return;
  if (Date.now() >= deadline)
    return;
  await sleep(POLL_MS,);
  await reportedBy(seen, deadline,);
}

/**
 Both takers of one dead lock in one round, each either holding or refused, and both ended before this returns.

 @param root - scratch directory the round's target and reports live in

 @param target - file both takers lock

 @returns their reports
 */
async function bothTakersOfOneDeadLock(root: string, target: string,): Promise<string[]> {
  writeFileSync(target, '{}',);
  await leaveLockOfAKilledHolder(root, target,);
  const barrierAt = Date.now() + TAKER_BARRIER_LEAD_MS;
  const takers = await Promise.all([
    raceFor(target, barrierAt, `${target}.first.report`,),
    raceFor(target, barrierAt, `${target}.second.report`,),
  ],);
  await Promise.all(takers.map(async taker => await killChild(taker.child,),),);
  return takers.map(taker => taker.report,);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region Export shape

    describe({
      name: 'export shape',
      children: [
        it({
          name: 'is callable and carries lock and lockSync members, matching the upstream function-form export',
          fn: async () => {
            expect(typeof lockfile,).toBe('function',);
            expect(typeof lockfile.lock,).toBe('function',);
            expect(typeof lockfile.lockSync,).toBe('function',);
          },
        },),
      ],
    },),
    //endregion Export shape

    //region lockSync

    describe({
      name: 'lockSync',
      children: [
        it({
          name: 'creates the .<basename>.lock file and its release removes it',
          fn: async () => {
            await using fixture = await makeTarget();

            const release = lockfile.lockSync(fixture.target,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);

            release();

            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
        },),
        it({
          name: 'throws ELOCKED naming the lock path when another holder owns the lock',
          fn: async () => {
            await using fixture = await makeTarget();

            const held = lockfile.lockSync(fixture.target,);
            const thrown = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            held();

            expect(thrown,).toHaveProperty('code', HELD_CODE,);
            expect((thrown as Error).message,).toContain(fixture.lockPath,);
          },
        },),
      ],
    },),
    //endregion lockSync

    //region A lock whose holder is gone

    describe({
      name: 'a lock whose holder is gone',
      children: [
        it({
          name: 'is taken over on the first sync attempt once the holder process was killed inside the locked section',
          fn: async () => {
            await using fixture = await makeTarget();
            await leaveLockOfAKilledHolder(fixture.root, fixture.target,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);

            const release = lockfile.lockSync(fixture.target,);
            release();

            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'is taken over by the async entry point the caller uses, with its own retries and stale bound',
          fn: async () => {
            await using fixture = await makeTarget();
            await leaveLockOfAKilledHolder(fixture.root, fixture.target,);

            const release = await lockfile.lock(fixture.target, {
              retries: 0,
              stale: CALLER_STALE_MS,
            },);
            await release();

            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'is taken over when its record was written in an earlier boot, even though its process number is in use',
          fn: async () => {
            await using fixture = await makeTarget();
            // A record written in the boot before a restart names a process that cannot
            // exist now, which is a proof about the holder and not about the lock's age.
            // The record is this process's own, so its process number is in use: only the
            // boot it names can prove it gone.
            const own = lockfile.lockSync(fixture.target,);
            const record = JSON.parse(readFileSync(fixture.lockPath, 'utf8',),) as Record<string, unknown>;
            own();
            writeFileSync(fixture.lockPath, JSON.stringify({ ...record, boot: EARLIER_BOOT_ID, },),);

            const release = lockfile.lockSync(fixture.target,);
            release();

            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
        },),
        it({
          name: 'is left alone while its holder process is alive, however old the lock file is',
          fn: async () => {
            await using fixture = await makeTarget();
            const reportPath = join(fixture.root, 'live-holder.report',);
            await using holder = await startHolder(fixture.target, reportPath,);
            // A live holder blocked on a stalled drive makes no younger impression,
            // so the age of the lock file must not be what decides.
            const longAgo = new Date(Date.now() - AGED_LOCK_MS,);
            utimesSync(fixture.lockPath, longAgo, longAgo,);

            const syncThrown = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            const asyncThrown = await rejectionFrom(async () => {
              await lockfile.lock(fixture.target, { retries: 0, stale: CALLER_STALE_MS, },);
            },);

            expect(childFinished(holder.child,),).toBe(false,);
            expect(syncThrown,).toHaveProperty('code', HELD_CODE,);
            expect(asyncThrown,).toHaveProperty('code', HELD_CODE,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
      ],
    },),
    //endregion A lock whose holder is gone

    //region Two takers of one dead lock

    describe({
      name: 'two takers of one dead lock',
      children: [
        it({
          name: 'lets exactly one of them through, round after round',
          fn: async () => {
            await using fixture = await makeTarget();
            const roundTargets: string[] = [];
            for (let round = 0; round < TAKER_ROUNDS; round++)
              roundTargets.push(join(fixture.root, `round-${String(round,)}.json`,),);

            const rounds = await Promise.all(
              roundTargets.map(async target => await bothTakersOfOneDeadLock(fixture.root, target,),),
            );
            const reports = rounds.flat();
            const held = reports.filter(report => report === HELD_REPORT,);
            const refused = reports.filter(report => report === HELD_CODE,);
            // A report that is neither says the round ended in a way this case does not expect, so it is named here.
            const known = new Set([HELD_REPORT, HELD_CODE,],);
            const unexpected = reports.filter(report => !known.has(report,),);

            expect(unexpected,).toEqual([],);
            expect(held.length,).toBe(TAKER_ROUNDS,);
            expect(refused.length,).toBe(TAKER_ROUNDS,);
          },
          timeout: TAKER_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'leaves a dead lock in place while another taker holds the takeover record, and takes it over once that taker is gone',
          fn: async () => {
            await using fixture = await makeTarget();
            const otherTarget = join(fixture.root, 'settings.json',);
            writeFileSync(otherTarget, '{}',);
            await leaveLockOfAKilledHolder(fixture.root, fixture.target,);

            // A taker's record is a live holder's record: reuse the bytes a real
            // holder wrote, so the case need not restate the record's format.
            const otherReportPath = join(fixture.root, 'live-taker.report',);
            await using liveTaker = await startHolder(otherTarget, otherReportPath,);
            const takeoverPath = takeoverPathFor(fixture.lockPath,);
            writeFileSync(takeoverPath, readFileSync(lockPathFor(otherTarget,), 'utf8',),);

            const whileTakerIsAlive = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            expect(whileTakerIsAlive,).toHaveProperty('code', HELD_CODE,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);

            await killChild(liveTaker.child,);

            // The record the dead taker left is cleared by this attempt itself, and the
            // dead lock is taken over in the same call: clearing an abandoned record is
            // part of one arbitration, not a retry.
            const release = lockfile.lockSync(fixture.target,);
            expect(existsSync(takeoverPath,),).toBe(false,);
            release();
            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
      ],
    },),
    //endregion Two takers of one dead lock

    //region A lock this shim cannot judge

    describe({
      name: 'a lock this shim cannot judge',
      children: [
        it({
          name: 'is left alone, and named, when it is the empty lock directory a live holder of the previous shim made',
          fn: async () => {
            await using fixture = await makeTarget();
            const reportPath = join(fixture.root, 'old-holder.report',);
            await using holder = await startOldStyleHolder(fixture.lockPath, reportPath,);
            const longHeld = new Date(Date.now() - AGED_LOCK_MS,);
            utimesSync(fixture.lockPath, longHeld, longHeld,);

            const syncThrown = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            const asyncThrown = await rejectionFrom(async () => {
              await lockfile.lock(fixture.target, { retries: 0, stale: CALLER_STALE_MS, },);
            },);

            expectUnjudgeable(syncThrown, {
              lockPath: fixture.lockPath,
              verb: 'rmdir',
            },);
            expectUnjudgeable(asyncThrown, {
              lockPath: fixture.lockPath,
              verb: 'rmdir',
            },);
            expect(childFinished(holder.child,),).toBe(false,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'is left alone, and named, when its record cannot be read while its holder is alive',
          fn: async () => {
            await using fixture = await makeTarget();
            const reportPath = join(fixture.root, 'unreadable-holder.report',);
            await using holder = await startHolder(fixture.target, reportPath,);
            // A read that fails, standing in for the stalled home drive this machine has:
            // the holder is alive and its lock is valid, and only the read is refused.
            chmodSync(fixture.lockPath, UNREADABLE_FILE_MODE,);
            const longHeld = new Date(Date.now() - AGED_LOCK_MS,);
            utimesSync(fixture.lockPath, longHeld, longHeld,);

            const syncThrown = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            const asyncThrown = await rejectionFrom(async () => {
              await lockfile.lock(fixture.target, { retries: 0, stale: CALLER_STALE_MS, },);
            },);

            expectUnjudgeable(syncThrown, {
              lockPath: fixture.lockPath,
              verb: 'rm -f',
            },);
            expectUnjudgeable(asyncThrown, {
              lockPath: fixture.lockPath,
              verb: 'rm -f',
            },);
            expect((syncThrown as Error).message,).toContain('could not read',);
            expect(childFinished(holder.child,),).toBe(false,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);
            // A third caller is refused as well, so no second holder appears beside the live one.
            const thirdThrown = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            expect(thirdThrown,).toHaveProperty('code', HELD_CODE,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'is left alone, and named, when its bytes are not a record this shim wrote',
          fn: async () => {
            await using fixture = await makeTarget();
            writeFileSync(fixture.lockPath, '',);
            const longHeld = new Date(Date.now() - AGED_LOCK_MS,);
            utimesSync(fixture.lockPath, longHeld, longHeld,);

            const thrown = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);

            expectUnjudgeable(thrown, {
              lockPath: fixture.lockPath,
              verb: 'rm -f',
            },);
            expect(thrown,).toHaveProperty('reason', 'unwritten-record',);
            expect(existsSync(fixture.lockPath,),).toBe(true,);
          },
        },),
        it({
          name: 'is left alone, and named, when its record was written in another PID namespace, and stays so after that holder dies',
          fn: async () => {
            await using fixture = await makeTarget();
            const reportPath = join(fixture.root, 'namespaced-holder.report',);
            const holder = await startHolderInOwnPidNamespace(fixture.target, reportPath,);

            const thrown = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);

            expectUnjudgeable(thrown, {
              lockPath: fixture.lockPath,
              verb: 'rm -f',
            },);
            expect((thrown as Error).message,).toContain('PID namespace',);
            expect(holder.child.exitCode,).toBe(null,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);

            // The holder is gone, but nothing here proves it: its process number names
            // nothing in this namespace, so the lock is cleared by hand and not by this shim.
            await killChild(holder.child,);
            const afterDeath = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            expectUnjudgeable(afterDeath, {
              lockPath: fixture.lockPath,
              verb: 'rm -f',
            },);
            expect(afterDeath,).toHaveProperty('reason', 'other-namespace',);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'ends within the bound the caller itself allows, with the command in the words it leaves',
          fn: async () => {
            await using fixture = await makeTarget();
            mkdirSync(fixture.lockPath,);

            const started = Date.now();
            const thrown = await rejectionFrom(async () => {
              // The options `@earendil-works/pi-coding-agent` passes on its async path.
              await lockfile.lock(fixture.target, { retries: 0, stale: CALLER_STALE_MS, },);
            },);
            const elapsed = Date.now() - started;

            expectUnjudgeable(thrown, {
              lockPath: fixture.lockPath,
              verb: 'rmdir',
            },);
            // The shim adds no wait of its own: the caller's thirty-second loop is the only bound.
            expect(elapsed,).toBeLessThan(NO_WAIT_BOUND_MS,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
      ],
    },),
    //endregion A lock this shim cannot judge

    //region The wait for a child process

    describe({
      name: 'the wait for a child process',
      children: [
        it({
          name: 'names the child, its process number and whether it began, instead of leaving the case timeout to speak',
          fn: async () => {
            await using fixture = await makeTarget();
            const reportPath = join(fixture.root, 'never.report',);
            const child = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000);',], {
              stdio: ['ignore', 'ignore', 'ignore',],
            },);

            const neverBegan = await rejectionFrom(async () => await reportOfChild({
              reportPath,
              child,
              deadline: Date.now() - 1,
            },),);
            writeFileSync(`${reportPath}.started`, '1',);
            const began = await rejectionFrom(async () => await reportOfChild({
              reportPath,
              child,
              deadline: Date.now() - 1,
            },),);
            await killChild(child,);

            expect((neverBegan as Error).message,).toContain(`the child process ${String(child.pid,)}`,);
            expect((neverBegan as Error).message,).toContain('never began',);
            expect((neverBegan as Error).message,).toContain(reportPath,);
            expect((began as Error).message,).toContain('began and did not report',);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
      ],
    },),
    //endregion The wait for a child process

    //region The command this shim prints

    describe({
      name: 'the command this shim prints',
      children: [
        it({
          name: 'clears the lock at a path holding a space when it is run as printed',
          fn: async () => {
            await expectPrintedCommandsToClear('with a space',);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'clears the lock at a path holding a single quote when it is run as printed',
          fn: async () => {
            await expectPrintedCommandsToClear('with a single \' quote',);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'clears the lock at a path holding a double quote when it is run as printed',
          fn: async () => {
            await expectPrintedCommandsToClear('with a "double" quote',);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'clears the lock at a path holding a dollar sign when it is run as printed',
          fn: async () => {
            // A dollar sign and a brace expansion, which a shell would otherwise read as a variable.
            await expectPrintedCommandsToClear(`with a ${DOLLAR_SIGN}dollar and ${DOLLAR_SIGN}{brace}`,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'clears the lock at a path holding a line break when it is run as printed',
          fn: async () => {
            await expectPrintedCommandsToClear('with a\nline break',);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
      ],
    },),
    //endregion The command this shim prints

    //region Release when the lock is no longer the holder's

    describe({
      name: 'release when the lock is no longer the holder\'s',
      children: [
        it({
          name: 'removes nothing another process holds and calls onCompromised',
          fn: async () => {
            await using fixture = await makeTarget();
            const compromises: Error[] = [];
            const release = await lockfile.lock(fixture.target, {
              retries: 0,
              stale: CALLER_STALE_MS,
              onCompromised: (error: Error,) => {
                compromises.push(error,);
              },
            },);

            // A person removes the lock, and the next caller takes the free lock.
            rmSync(fixture.lockPath, { recursive: true, force: true, },);
            const successorReportPath = join(fixture.root, 'successor.report',);
            await using successor = await startHolder(fixture.target, successorReportPath,);

            release();

            expect(compromises.length,).toBe(1,);
            expect(compromises[0],).toHaveProperty('code', COMPROMISED_CODE,);
            expect(childFinished(successor.child,),).toBe(false,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);
            const stillHeld = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            expect(stillHeld,).toHaveProperty('code', HELD_CODE,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'removes nothing a later lock of the same process holds, so a kept release callback cannot undo a new holding',
          fn: async () => {
            await using fixture = await makeTarget();
            const compromises: Error[] = [];
            const firstRelease = lockfile.lockSync(fixture.target, {
              onCompromised: (error: Error,) => {
                compromises.push(error,);
              },
            },);

            // The lock the first holding wrote is taken away, and the same process
            // takes the lock again, which is a different holding of the same file.
            rmSync(fixture.lockPath, { recursive: true, force: true, },);
            const secondRelease = lockfile.lockSync(fixture.target,);

            firstRelease();

            expect(compromises.length,).toBe(1,);
            expect(compromises[0],).toHaveProperty('code', COMPROMISED_CODE,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);
            const stillHeld = caughtFrom(() => {
              lockfile.lockSync(fixture.target,);
            },);
            expect(stillHeld,).toHaveProperty('code', HELD_CODE,);

            secondRelease();
            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
        },),
        it({
          name: 'reports no compromise and removes nothing while the lock it holds cannot be read',
          fn: async () => {
            await using fixture = await makeTarget();
            const compromises: Error[] = [];
            const release = lockfile.lockSync(fixture.target, {
              stale: SHORT_STALE_MS,
              onCompromised: (error: Error,) => {
                compromises.push(error,);
              },
            },);
            // A read that fails, standing in for the stalled drive: the holder still holds
            // its lock, so neither its watch nor its release may say otherwise.
            chmodSync(fixture.lockPath, UNREADABLE_FILE_MODE,);

            // Two ownership checks pass by while the record cannot be read.
            await sleep(SHORT_STALE_MS * 2,);
            expect(compromises.length,).toBe(0,);

            const thrown = caughtFrom(release,);
            expect(thrown,).toHaveProperty('code', DENIED_CODE,);
            expect(compromises.length,).toBe(0,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);

            // Once the record can be read again, the same callback removes its own lock.
            chmodSync(fixture.lockPath, OWN_LOCK_MODE,);
            release();
            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'calls onCompromised while the lock is still held, so the caller can stop writing',
          fn: async () => {
            await using fixture = await makeTarget();
            const compromises: Error[] = [];
            const release = await lockfile.lock(fixture.target, {
              retries: 0,
              stale: SHORT_STALE_MS,
              onCompromised: (error: Error,) => {
                compromises.push(error,);
              },
            },);

            rmSync(fixture.lockPath, { recursive: true, force: true, },);
            await reportedBy(compromises, Date.now() + ASYNC_CASE_TIMEOUT_MS,);

            expect(compromises.length,).toBe(1,);
            expect(compromises[0],).toHaveProperty('code', COMPROMISED_CODE,);
            release();
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
      ],
    },),
    //endregion Release when the lock is no longer the holder's

    //region release after a removal that fails for a reason other than the lock being gone

    describe({
      name: 'release after a removal that fails for a reason other than the lock being gone',
      children: [
        it({
          name: 'rethrows the filesystem error instead of reporting the lock released',
          fn: async () => {
            await using fixture = await makeTarget();

            const release = lockfile.lockSync(fixture.target,);
            chmodSync(fixture.root, READ_ONLY_DIRECTORY_MODE,);

            const thrown = caughtFrom(release,);

            expect(thrown,).toHaveProperty('code', DENIED_CODE,);
            expect(existsSync(fixture.lockPath,),).toBe(true,);
          },
        },),
        it({
          name: 'retries the removal on the next release call once the obstacle is gone',
          fn: async () => {
            await using fixture = await makeTarget();

            const release = lockfile.lockSync(fixture.target,);
            chmodSync(fixture.root, READ_ONLY_DIRECTORY_MODE,);
            expect(caughtFrom(release,),).toBeDefined();

            chmodSync(fixture.root, WRITABLE_DIRECTORY_MODE,);
            release();

            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
        },),
      ],
    },),
    //endregion release after a removal that fails for a reason other than the lock being gone

    //region release when the lock file is already gone

    describe({
      name: 'release when the lock file is already gone',
      children: [
        it({
          name: `tolerates the ${MISSING_CODE} disappearance and stays quiet on every later release call`,
          fn: async () => {
            await using fixture = await makeTarget();

            const release = lockfile.lockSync(fixture.target,);
            rmSync(fixture.lockPath, { recursive: true, force: true, },);

            expect(caughtFrom(release,),).toBeUndefined();
            expect(caughtFrom(release,),).toBeUndefined();
          },
        },),
      ],
    },),
    //endregion release when the lock file is already gone

    //region async lock

    describe({
      name: 'lock',
      children: [
        it({
          name: 'retries until the holder releases, then removes the lock file on await release()',
          fn: async () => {
            await using fixture = await makeTarget();

            const held = lockfile.lockSync(fixture.target,);
            setTimeout(held, HOLDER_RELEASE_DELAY_MS,);

            const release = await lockfile.lock(fixture.target, {
              retries: { retries: ASYNC_RETRIES, minTimeout: ASYNC_MIN_TIMEOUT_MS, },
            },);
            await release();

            expect(existsSync(fixture.lockPath,),).toBe(false,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
        it({
          name: 'throws ELOCKED once the retry budget is exhausted',
          fn: async () => {
            await using fixture = await makeTarget();

            const held = lockfile.lockSync(fixture.target,);
            const thrown = await rejectionFrom(async () => {
              await lockfile.lock(fixture.target, {
                retries: { retries: 0, minTimeout: ASYNC_MIN_TIMEOUT_MS, },
              },);
            },);
            held();

            expect(thrown,).toHaveProperty('code', HELD_CODE,);
          },
          timeout: ASYNC_CASE_TIMEOUT_MS,
        },),
      ],
    },),
    //endregion async lock
  ],
},);
