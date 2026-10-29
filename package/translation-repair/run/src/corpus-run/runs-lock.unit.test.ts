/**
 Tests for the one-pass-at-a-time claim on a runs directory.

 Two passes sharing a directory never collide loudly. They overwrite each
 other's attempt counts, delete each other's cached slices whenever their
 pipelines differ, and the later write of any entry replaces the earlier one.
 Every one of those looks like ordinary output, which is why the refusal has
 to happen before any of it starts.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import fsPromises, {
  chmod,
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { syncBuiltinESMExports, } from 'node:module';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  evictStaleLock,
  hostIdentity,
  lockFileText,
  lockRunsDir,
  releaseIfOwned,
  RunsDirectoryBusyError,
  startTicksOf,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Process id no process can hold.

 Above the kernel's largest (`pid_max` is at most 4194304), so signalling it
 finds nothing and a lock file naming it is a lock naming nothing, which is
 what a stale-takeover case needs.
 */
const GONE_PID = 2_147_483_646;

/**
 Writes a lock file naming this very process, with the identity fields given.

 @param runsDir - directory to write it into

 @param identity - identity fields the lock records, beside this process's id

 @example
 ```ts
 await writeOwnLock({ runsDir, identity: { startTicks: '1', }, },);
 ```
 */
async function writeOwnLock(
  {
    runsDir,
    identity,
  }: {
    readonly runsDir: string;
    readonly identity: Readonly<Record<string, string>>;
  },
): Promise<void> {
  await writeFile(
    join(
      runsDir,
      'pass.lock',
    ),
    `${JSON.stringify({
      pid: process.pid,
      startedAt: '2026-08-14T00:00:00.000Z',
      token: 'earlier-holder',
      ...identity,
    },)}\n`,
  );
}

/**
 This host's identity and this process's start time, as a lock taken by this
 process records them.

 @returns Identity fields for a lock naming this process

 @example
 ```ts
 const mine = await ownIdentity();
 ```
 */
async function ownIdentity(): Promise<{
  readonly host: string;
  readonly bootId: string;
  readonly pidNamespace: string;
  readonly startTicks: string;
}> {
  /**
   This host, and this process's start.
   */
  const [
    hostRead,
    started,
  ] = await Promise.all([
    hostIdentity(),
    startTicksOf({ pid: process.pid, },),
  ],);
  if ((hostRead.kind !== 'read') || (started.kind !== 'read'))
    throw new Error('these cases need /proc, and this host did not answer',);
  return {
    ...hostRead.here,
    startTicks: started.startTicks,
  };
}

/**
 Refuses to run a case that needs this process to lack root's permission to
 write anywhere. Root writes past any directory mode (no EACCES), so a case
 built on that refusal would pass without ever reaching the rethrow it means
 to exercise.

 @throws Error naming the precondition, when this process is root

 @example
 ```ts
 requireNonRoot();
 ```
 */
function requireNonRoot(): void {
  if ((process.getuid !== undefined) && (process.getuid() === 0))
    throw new Error('this case needs a process that is not root, and this one is',);
}

/**
 Refuses to run a case that needs signalling pid 1 to answer EPERM, which is
 what the EPERM branch in `isAlive` (`runs-lock-holder.ts`) needs reached
 rather than skipped. A uid check alone cannot stand in for this: a
 non-root process inside a user namespace or rootless container can still
 own pid 1 itself, where the signal succeeds instead of refusing, and the
 branch this case means to exercise would stay cold while the case passed.

 @throws Error naming the precondition, when signalling pid 1 does not
 answer EPERM

 @example
 ```ts
 requireEpermOnPidOne();
 ```
 */
function requireEpermOnPidOne(): void {
  try {
    process.kill(
      1,
      0,
    );
  }
  catch (error) {
    if (Error.isError(error,) && ('code' in error) && (error.code === 'EPERM'))
      return;
    throw new Error(
      'this case needs pid 1 to answer EPERM, and it answered something else',
      { cause: error, },
    );
  }
  throw new Error('this case needs pid 1 to answer EPERM, and signalling it succeeded instead',);
}

/**
 Length a runs directory is built to so that its lock file's path, ten units
 longer, is one the system opens, and a claim's staged name, 53 units longer,
 is not: Linux refuses a path of 4,096 units or more (`PATH_MAX` counts the
 closing NUL) with ENAMETOOLONG.
 */
const CROWDED_RUNS_DIR_LENGTH = 4_060;

/**
 Units of each full directory name the crowded runs directory is built from,
 under the 255 one name may hold.
 */
const CROWDED_SEGMENT_UNITS = 200;

/**
 A runs directory path of exactly `CROWDED_RUNS_DIR_LENGTH` units under a
 root, made of directory names each short enough to create, so the lock's
 own `mkdir` makes it.

 @param root - scratch directory the path starts at

 @returns Path, not yet created

 @example
 ```ts
 const runsDir = crowdedRunsDir({ root: scratch.path, },);
 ```
 */
function crowdedRunsDir({ root, }: { readonly root: string; },): string {
  /**
   Full names that fit while leaving one or more units for the last name.
   */
  const full = Math.floor((CROWDED_RUNS_DIR_LENGTH - root.length - 2) / (CROWDED_SEGMENT_UNITS + 1),);
  /**
   Units the last name takes to bring the path to its length exactly.
   */
  const last = CROWDED_RUNS_DIR_LENGTH - root.length - 1 - (full * (CROWDED_SEGMENT_UNITS + 1));
  return join(
    root,
    ...Array.from({ length: full, }, () => 'c'.repeat(CROWDED_SEGMENT_UNITS,)),
    'd'.repeat(last,),
  );
}

/**
 Whole message a refusal over the fixture directory renders when the race
 judged the holder, around the line that says what the lock records.

 @param holderLine - line naming the holder, or saying the lock records nothing

 @param runsDir - directory the refusal names, the fixture one unless a case ran the lock over its own

 @returns Message the error carries

 @example
 ```ts
 const message = busyMessage({ holderLine: '  its lock file records nothing readable', },);
 ```
 */
function busyMessage(
  {
    holderLine,
    runsDir = '/mittens/runs',
  }: {
    readonly holderLine: string;
    readonly runsDir?: string;
  },
): string {
  return [
    `Another pass is running in ${runsDir}.`,
    holderLine,
    '',
    'Two passes sharing one runs directory do not conflict loudly. They',
    'overwrite each other\'s attempt counts, delete each other\'s cached',
    'slices whenever their pipelines differ, and the later write of any',
    'entry simply replaces the earlier one. Every one of those looks like',
    'ordinary output.',
    '',
    'Point this run at another directory with TRANSLATION_REPAIR_RUNS_DIR,',
    'or stop the other pass. A lock whose process is gone is taken over',
    'automatically. Another pass took it over at the same moment as this one.',
  ].join('\n',);
}

/**
 What a starter that lost a takeover reported, as the child process printed it.
 */
type LostTakeoverReport = {
  /**
   Directory the starter was pointed at.
   */
  readonly runsDir: string;

  /**
   Class name and whole message of what `lockRunsDir` threw, or `acquired`.
   */
  readonly outcome: string;

  /**
   Names the runs directory holds afterwards.
   */
  readonly left: readonly string[];

  /**
   What the lock file says afterwards.
   */
  readonly lockText: string;
};

/**
 Whether a parsed child report has the fields of a lost-takeover report.

 @param value - parsed JSON the child printed

 @returns Whether it is a record of the four fields, typed as each is

 @example
 ```ts
 if (isLostTakeoverReport(parsed,)) console.log(parsed.outcome,);
 ```
 */
function isLostTakeoverReport(value: unknown,): value is LostTakeoverReport {
  return isJsonRecord(value,)
    && ((typeof value.runsDir) === 'string')
    && ((typeof value.outcome) === 'string')
    && ((typeof value.lockText) === 'string')
    && isJsonArray(value.left,)
    && value.left.every(function isName(name,): boolean {
      return (typeof name) === 'string';
    },);
}

/**
 Runs a starter that finds a stale lock, evicts it, and then loses the claim
 to a rival that writes its own lock in the instant between.

 A CHILD PROCESS, because the instant is made by wrapping the one filesystem
 call that evicts the lock (`rename` on the lock's own name) so the rival's
 lock appears right after it, and that wrapper must not reach the cases of
 this file. No seam of the module is used: the rival is the filesystem state a
 concurrent starter leaves.

 @param rivalText - exact text the rival's lock holds

 @returns What the starter threw and what the directory held afterwards

 @example
 ```ts
 const report = await losingTheTakeoverTo({ rivalText: '{"pid":4242,"startedAt":"2026-08-14T01:00:00.000Z"}\n', },);
 ```
 */
async function losingTheTakeoverTo(
  { rivalText, }: { readonly rivalText: string; },
): Promise<LostTakeoverReport> {
  await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
  /**
   Directory the starter competes for, apart from the child's working directory.
   */
  const runsDir = join(
    scratch.path,
    'runs',
  );
  await mkdir(runsDir,);
  await writeFile(
    join(
      runsDir,
      'pass.lock',
    ),
    `${JSON.stringify({
      pid: GONE_PID,
      startedAt: '2026-08-14T00:00:00.000Z',
    },)}\n`,
  );
  /**
   Built package the child imports.
   */
  const apiUrl = pathToFileURL(join(
    import.meta.dirname,
    '..',
    '..',
    'dist',
    'final',
    'node',
    'index.mjs',
  ),).href;
  /**
   Child program: wraps the eviction's rename, then asks for the lock once.
   */
  const program = `
import files from 'node:fs/promises';
import { syncBuiltinESMExports } from 'node:module';
const runsDir = ${JSON.stringify(runsDir,)};
const lockPath = ${JSON.stringify(join(runsDir, 'pass.lock',),)};
const api = await import(${JSON.stringify(apiUrl,)});
const originalRename = files.rename;
files.rename = async function renameThenRival(from, to) {
  await originalRename(from, to);
  if (from === lockPath) await files.writeFile(lockPath, ${JSON.stringify(rivalText,)});
};
syncBuiltinESMExports();
let outcome = 'acquired';
try {
  await api.lockRunsDir({ runsDir });
} catch (error) {
  outcome = error.name + ': ' + error.message;
}
console.log('LOST_TAKEOVER ' + JSON.stringify({
  runsDir,
  outcome,
  left: (await files.readdir(runsDir)).toSorted(),
  lockText: await files.readFile(lockPath, 'utf8'),
}));
`;
  /**
   The child's run.
   */
  const done = await runKeyless({
    file: process.execPath,
    args: ['--input-type=module', '--eval', program,],
    cwd: scratch.path,
  },);
  /**
   Marker the child's one report line starts with.
   */
  const marker = 'LOST_TAKEOVER ';
  /**
   That line, among whatever the package's logger printed.
   */
  const line = done.stdout
    .split('\n',)
    .find(function isReport(text,): boolean {
      return text.startsWith(marker,);
    },);
  if (line === undefined)
    throw new Error(`the child printed no report (status ${String(done.code,)}): ${done.stderr}`,);
  /**
   What the child reported, as parsed.
   */
  const reported: unknown = JSON.parse(line.slice(marker.length,),);
  if (!isLostTakeoverReport(reported,))
    throw new Error('the child reported a line that is no lost-takeover report',);
  return reported;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: lockRunsDir.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'claims a fresh directory and releases on scope exit, which is the '
            + 'ordinary path: a pass that ran and finished must leave nothing '
            + 'behind for the next one to reason about',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            {
              await using _lock = await lockRunsDir({ runsDir, },);

              expect(await readdir(runsDir,),).toEqual(['pass.lock',],);
            }

            expect(await readdir(runsDir,),).toEqual([],);
          },
        },),

        it({
          name: 'REFUSES a second claim while the first is held, which is the whole '
            + 'guard: the interference between two passes is invisible in the '
            + 'output, so the refusal has to come before either writes anything',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            await using _lock = await lockRunsDir({ runsDir, },);

            /**
             What lockRunsDir refused with, read for class as well as wording.
             */
            const refusalOfLockRunsDir = lockRunsDir({ runsDir, },);

            await expect(refusalOfLockRunsDir,).rejects.toBeInstanceOf(RunsDirectoryBusyError,);
            await expect(refusalOfLockRunsDir,).rejects.toThrow('Another pass is running',);
          },
        },),

        it({
          name: 'names the holder it refused for, since an operator meeting this '
            + 'has to decide whether to stop that process or point this run '
            + 'somewhere else, and cannot do either without knowing which it is',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            await using _lock = await lockRunsDir({ runsDir, },);

            await expect(lockRunsDir({ runsDir, },),)
              .rejects
              .toThrow(String(process.pid,),);
          },
        },),

        it({
          name: 'TAKES OVER a lock whose process is gone. A pass killed at its hard '
            + 'cap leaves one behind, and refusing forever would make every crash '
            + 'need manual cleanup, which is how a guard gets routed around',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            await writeFile(
              join(
                runsDir,
                'pass.lock',
              ),
              `${JSON.stringify({
            pid: GONE_PID,
            startedAt: '2026-08-14T00:00:00.000Z',
          },)}\n`,
            );

            await using _lock = await lockRunsDir({ runsDir, },);

            expect(await readdir(runsDir,),).toEqual(['pass.lock',],);
          },
        },),

        it({
          name: 'takes over a lock file that says nothing readable, because a lock '
            + 'nobody can respect is not a lock, and honouring it forever would '
            + 'strand the directory on a truncated write',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            await writeFile(
              join(
                runsDir,
                'pass.lock',
              ),
              '{"pid":',
            );

            await using _lock = await lockRunsDir({ runsDir, },);

            expect(await readdir(runsDir,),).toEqual(['pass.lock',],);
          },
        },),

        it({
          name: 'TAKES OVER a lock file whose text parses as JSON but not as a record (an array), since nothing '
            + 'that cannot be probed for `pid` is a lock anyone can respect',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            await writeFile(
              join(
                runsDir,
                'pass.lock',
              ),
              '[]\n',
            );

            await using _lock = await lockRunsDir({ runsDir, },);

            /**
             What the new lock records, proving this pass actually took over
             rather than finding the directory merely still holding one file.
             */
            const recorded: unknown = JSON.parse(await readFile(
              join(
                runsDir,
                'pass.lock',
              ),
              'utf8',
            ),);
            expect(recorded,).toMatchObject({ pid: process.pid, },);
          },
        },),

        it({
          name: 'TAKES OVER a lock file recording no `pid` field at all, which names no holder to respect',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            await writeFile(
              join(
                runsDir,
                'pass.lock',
              ),
              '{}\n',
            );

            await using _lock = await lockRunsDir({ runsDir, },);

            /**
             What the new lock records, proving this pass actually took over.
             */
            const recorded: unknown = JSON.parse(await readFile(
              join(
                runsDir,
                'pass.lock',
              ),
              'utf8',
            ),);
            expect(recorded,).toMatchObject({ pid: process.pid, },);
          },
        },),

        it({
          name: 'TAKES OVER a lock file naming a `pid` but recording no `startedAt`, which names no holder to '
            + 'respect',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            await writeFile(
              join(
                runsDir,
                'pass.lock',
              ),
              `${JSON.stringify({ pid: 4_194_305, },)}\n`,
            );

            await using _lock = await lockRunsDir({ runsDir, },);

            /**
             What the new lock records, proving this pass actually took over.
             */
            const recorded: unknown = JSON.parse(await readFile(
              join(
                runsDir,
                'pass.lock',
              ),
              'utf8',
            ),);
            expect(recorded,).toMatchObject({ pid: process.pid, },);
          },
        },),

        it({
          name: 'PROPAGATES the write refusal of a runs directory it cannot write into (EACCES at the claim\'s staged '
            + 'text) rather than mistaking it for another pass already holding the lock, since only EEXIST at the '
            + 'link means the file is already claimed',
          fn: async () => {
            requireNonRoot();
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            /**
             The directory actually passed to lockRunsDir. A SUBDIRECTORY of
             the scratch root, chmod'd unwritable, so the scratch root itself
             stays writable and the fixture's own disposal can still remove it.
             */
            const runsDir = join(
              scratch.path,
              'runs',
            );
            await mkdir(runsDir,);
            await chmod(
              runsDir,
              0o500,
            );

            /**
             This one attempt, settled rather than awaited directly, so a
             rejection's class and code can both be inspected without an
             intermediate throw.
             */
            const [settled,] = await Promise.allSettled([lockRunsDir({ runsDir, },),],);

            if (settled.status === 'fulfilled') {
              await settled.value[Symbol.asyncDispose]();
              throw new Error('lockRunsDir resolved against a directory it cannot write into',);
            }

            expect(settled.reason,).not.toBeInstanceOf(RunsDirectoryBusyError,);
            expect(
              (Error.isError(settled.reason,) && ('code' in settled.reason))
                ? settled.reason.code
                : undefined,
            ).toBe('EACCES',);
          },
        },),
      ],
    },),

    describe({
      name: 'stale-lock takeover under contention',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'EVICTS a stale lock exactly once when two starters race for it, since the eviction is a '
            + 'rename and a rename is atomic',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            /**
             Lock file both starters find stale.
             */
            const path = join(
              runsDir,
              'pass.lock',
            );
            await writeFile(
              path,
              `${JSON.stringify({
            pid: GONE_PID,
            startedAt: '2026-08-14T00:00:00.000Z',
          },)}\n`,
            );
            /**
             What each concurrent eviction reported.
             */
            const outcomes = await Promise.all([
              evictStaleLock({ path, },),
              evictStaleLock({ path, },),
            ],);
            expect(outcomes.toSorted(),).toEqual(['evicted', 'gone',],);
            expect(await readdir(runsDir,),).toEqual([],);
          },
        },),
        it({
          name: 'KEEPS a lock it does not own on release, so a starter that lost a takeover cannot delete '
            + 'the winner\'s lock on its way out',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            /**
             Lock file under test.
             */
            const path = join(
              runsDir,
              'pass.lock',
            );
            /**
             Acquisition whose release is under test.
             */
            const lock = await lockRunsDir({ runsDir, },);
            // Another holder took the file over underneath this acquisition.
            await writeFile(
              path,
              `${JSON.stringify({
            pid: process.pid,
            startedAt: '2026-08-14T00:00:00.000Z',
            token: 'somebody-else',
          },)}\n`,
            );
            await lock[Symbol.asyncDispose]();
            expect(await readdir(runsDir,),).toEqual(['pass.lock',],);
            expect((await readFile(path, 'utf8',)).includes('somebody-else',),).toBe(true,);
            expect(await releaseIfOwned({
              path,
              holder: { token: 'somebody-else', },
            },),).toBe('released',);
          },
        },),
        it({
          name: 'REFUSES the loser of two concurrent takeovers, and the winner still holds the lock '
            + 'afterwards',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            await writeFile(
              join(
                runsDir,
                'pass.lock',
              ),
              `${JSON.stringify({
            pid: GONE_PID,
            startedAt: '2026-08-14T00:00:00.000Z',
          },)}\n`,
            );
            /**
             Both starters, settled.
             */
            const settled = await Promise.allSettled([
              lockRunsDir({ runsDir, },),
              lockRunsDir({ runsDir, },),
            ],);
            /**
             The one that acquired.
             */
            const winners = settled.filter(function won(outcome,): boolean {
              return outcome.status === 'fulfilled';
            },);
            /**
             The one that was refused.
             */
            const losers = settled.filter(function lost(outcome,): boolean {
              return outcome.status === 'rejected';
            },);
            expect(winners.length,).toBe(1,);
            expect(losers.length,).toBe(1,);
            expect(
              (losers[0]?.status === 'rejected') ? losers[0].reason : undefined,
            ).toBeInstanceOf(RunsDirectoryBusyError,);
            expect(await readdir(runsDir,),).toEqual(['pass.lock',],);
            if (winners[0]?.status === 'fulfilled')
              await winners[0].value[Symbol.asyncDispose]();
            expect(await readdir(runsDir,),).toEqual([],);
          },
        },),
        it({
          name: 'NAMES THE RIVAL\'S PROCESS AND START TIME when a starter evicts a stale lock and then loses the '
            + 'claim to a rival whose lock reads back, leaving that lock in place',
          fn: async () => {
            /**
             Exact text the rival's lock holds.
             */
            const rivalText = `${JSON.stringify({
              pid: 4_242,
              startedAt: '2026-08-14T01:00:00.000Z',
              token: 'rival-pass',
            },)}\n`;
            /**
             What the losing starter threw and the directory held afterwards.
             */
            const report = await losingTheTakeoverTo({ rivalText, },);
            expect(report.outcome,).toBe(
              `RunsDirectoryBusyError: ${
                busyMessage({
                  holderLine: '  process 4242, since 2026-08-14T01:00:00.000Z',
                  runsDir: report.runsDir,
                },)
              }`,
            );
            expect(report.left,).toEqual(['pass.lock',],);
            expect(report.lockText,).toBe(rivalText,);
          },
        },),
        it({
          name: 'SAYS the lock records nothing readable when a starter evicts a stale lock and then loses the '
            + 'claim to a rival whose lock cannot be read back, leaving that lock in place',
          fn: async () => {
            /**
             Exact text the rival's lock holds, which is no JSON.
             */
            const rivalText = 'a cat sits on the lock\n';
            /**
             What the losing starter threw and the directory held afterwards.
             */
            const report = await losingTheTakeoverTo({ rivalText, },);
            expect(report.outcome,).toBe(
              `RunsDirectoryBusyError: ${
                busyMessage({
                  holderLine: '  its lock file records nothing readable',
                  runsDir: report.runsDir,
                },)
              }`,
            );
            expect(report.left,).toEqual(['pass.lock',],);
            expect(report.lockText,).toBe(rivalText,);
          },
        },),
        it({
          name: 'REFUSES A STARTER THAT ARRIVES WHILE THE FIRST IS STILL CLAIMING, so one pass holds the directory, '
            + 'where a starter arriving between the lock\'s creation and its text read the empty lock as unreadable, '
            + 'took it over, and both passes went on holding the directory',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            /**
             Directory both starters compete for, apart from the child's
             working directory, where its logger may keep a file.
             */
            const runsDir = join(
              scratch.path,
              'runs',
            );
            /**
             Built package the child imports.
             */
            const apiUrl = pathToFileURL(join(
              import.meta.dirname,
              '..',
              '..',
              'dist',
              'final',
              'node',
              'index.mjs',
            ),).href;
            // The child replaces the two filesystem calls a claim can make on the
            // lock's own name, an exclusive `open` and a `link` onto it, so the
            // second starter runs to its end inside the first starter's claim:
            // after an exclusive create and before the write that follows it, or
            // before a link. No seam of the module is used; `interleaved` says the
            // second starter ran there at all.
            const program = `
import files from 'node:fs/promises';
import { syncBuiltinESMExports } from 'node:module';
const runsDir = ${JSON.stringify(runsDir,)};
const lockPath = ${JSON.stringify(join(runsDir, 'pass.lock',),)};
const api = await import(${JSON.stringify(apiUrl,)});
const state = { armed: true, interleaved: false, second: 'never started', secondLock: undefined };
async function secondStarter() {
  state.armed = false;
  state.interleaved = true;
  try {
    state.secondLock = await api.lockRunsDir({ runsDir });
    state.second = 'acquired';
  } catch (error) {
    state.second = 'refused by ' + error.name;
  }
}
const originalOpen = files.open;
const originalLink = files.link;
files.open = async function openThenSecond(path, flags, mode) {
  const handle = await originalOpen(path, flags, mode);
  if (state.armed && (path === lockPath) && (flags === 'wx')) await secondStarter();
  return handle;
};
files.link = async function secondThenLink(existing, target) {
  if (state.armed && (target === lockPath)) await secondStarter();
  return await originalLink(existing, target);
};
syncBuiltinESMExports();
let first = 'never started';
let firstLock;
try {
  firstLock = await api.lockRunsDir({ runsDir });
  first = 'acquired';
} catch (error) {
  first = 'refused by ' + error.name;
}
await firstLock?.[Symbol.asyncDispose]();
await state.secondLock?.[Symbol.asyncDispose]();
console.log('LOCK_INTERLEAVE ' + JSON.stringify({ interleaved: state.interleaved, first, second: state.second, left: await files.readdir(runsDir) }));
`;
            /**
             The child's run.
             */
            const done = await runKeyless({
              file: process.execPath,
              args: ['--input-type=module', '--eval', program,],
              cwd: scratch.path,
            },);
            /**
             Marker the child's one report line starts with.
             */
            const marker = 'LOCK_INTERLEAVE ';
            /**
             That line, among whatever the package's logger printed.
             */
            const line = done.stdout
              .split('\n',)
              .find(function isReport(text,): boolean {
                return text.startsWith(marker,);
              },);
            if (line === undefined)
              throw new Error(`the child printed no report (status ${String(done.code,)}): ${done.stderr}`,);
            /**
             What the child reported, as parsed.
             */
            const reported: unknown = JSON.parse(line.slice(marker.length,),);
            expect(reported,).toEqual({
              interleaved: true,
              first: 'refused by RunsDirectoryBusyError',
              second: 'acquired',
              left: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'holder judged by which process its id names (ledger A16)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES OVER a lock whose process id now names a process that started later, since the '
            + 'holder ended and the kernel handed its id on',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            await writeOwnLock({
              runsDir,
              identity: {
                ...(await ownIdentity()),
                startTicks: '1',
              },
            },);

            await using _lock = await lockRunsDir({ runsDir, },);

            expect((await readFile(
              join(
                runsDir,
                'pass.lock',
              ),
              'utf8',
            )).includes('earlier-holder',),).toBe(false,);
          },
        },),
        it({
          name: 'TAKES OVER a lock taken before this machine last booted, since no process outlives a boot',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            await writeOwnLock({
              runsDir,
              identity: {
                ...(await ownIdentity()),
                bootId: 'an-earlier-boot',
              },
            },);

            await using _lock = await lockRunsDir({ runsDir, },);

            expect(await readdir(runsDir,),).toEqual(['pass.lock',],);
          },
        },),
        it({
          name: 'TAKES OVER a stale lock from this boot whose recorded hostname differs, since a hostname can '
            + 'change within one boot and a boot id already proves the same machine',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            await writeOwnLock({
              runsDir,
              identity: {
                ...(await ownIdentity()),
                host: 'renamed-by-dhcp',
                startTicks: '1',
              },
            },);

            await using _lock = await lockRunsDir({ runsDir, },);

            expect(await readdir(runsDir,),).toEqual(['pass.lock',],);
          },
        },),
        it({
          name: 'REFUSES a lock naming this very process as it started, the control that a match still holds',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            await writeOwnLock({
              runsDir,
              identity: await ownIdentity(),
            },);

            await expect(lockRunsDir({ runsDir, },),)
              .rejects
              .toThrow('started when the lock was taken',);
          },
        },),
        it({
          name: 'REFUSES a lock from another process-id namespace, whose ids this one cannot read, and says so',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            await writeOwnLock({
              runsDir,
              identity: {
                ...(await ownIdentity()),
                pidNamespace: 'pid:[1]',
                startTicks: '1',
              },
            },);

            await expect(lockRunsDir({ runsDir, },),)
              .rejects
              .toThrow('another process-id namespace',);
          },
        },),
        it({
          name: 'REFUSES a lock from another machine, whose processes this one cannot see, and says so',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            await writeOwnLock({
              runsDir,
              identity: {
                ...(await ownIdentity()),
                host: 'another-cat-tree',
                bootId: 'its-own-boot',
              },
            },);

            await expect(lockRunsDir({ runsDir, },),)
              .rejects
              .toThrow('another machine',);
          },
        },),
        it({
          name: 'REFUSES a lock recording no start time while its id is in use, and says it judged by id alone',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            await writeOwnLock({
              runsDir,
              identity: {},
            },);

            await expect(lockRunsDir({ runsDir, },),)
              .rejects
              .toThrow('by process id alone',);
          },
        },),
        it({
          name: 'RECORDS its own identity in the lock it takes, so the next pass can judge it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;

            await using _lock = await lockRunsDir({ runsDir, },);

            /**
             What the lock records.
             */
            const recorded: unknown = JSON.parse(await readFile(
              join(
                runsDir,
                'pass.lock',
              ),
              'utf8',
            ),);
            expect(recorded,).toMatchObject(await ownIdentity(),);
          },
        },),
        it({
          name: 'REFUSES a lock naming a process id this one cannot signal, since EPERM still means the id is '
            + 'in use and the lock is held rather than free',
          fn: async () => {
            requireEpermOnPidOne();
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            // Signalling pid 1 refuses with EPERM (confirmed by
            // requireEpermOnPidOne) rather than succeeding or refusing with
            // ESRCH. No identity fields are written, so the lock is judged
            // by id alone.
            await writeFile(
              join(
                runsDir,
                'pass.lock',
              ),
              `${JSON.stringify({
                pid: 1,
                startedAt: '2026-08-14T00:00:00.000Z',
              },)}\n`,
            );

            await expect(lockRunsDir({ runsDir, },),)
              .rejects
              .toThrow('  process 1, since 2026-08-14T00:00:00.000Z',);
          },
        },),
        it({
          name: 'TAKES OVER a lock whose process id now names nothing at all, even though its recorded host, '
            + 'boot and namespace all match this one, since no start time can be read for an id nothing holds',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            // GONE_PID with a FULL identity matching this host carries the
            // judgement past the boot and namespace checks and into the start
            // time read, which finds nothing for an id nothing holds.
            await writeFile(
              join(
                runsDir,
                'pass.lock',
              ),
              `${JSON.stringify({
                pid: GONE_PID,
                startedAt: '2026-08-14T00:00:00.000Z',
                ...(await ownIdentity()),
              },)}\n`,
            );

            await using _lock = await lockRunsDir({ runsDir, },);

            /**
             What the new lock records, proving this pass actually took over.
             */
            const recorded: unknown = JSON.parse(await readFile(
              join(
                runsDir,
                'pass.lock',
              ),
              'utf8',
            ),);
            expect(recorded,).toMatchObject({ pid: process.pid, },);
          },
        },),
      ],
    },),

    describe({
      name: 'lockRunsDir where the filesystem or the system answers otherwise',
      concurrency: 1,
      children: [
        it({
          name: 'PASSES ON UNCHANGED A LINK THE FILESYSTEM REFUSES with anything but EEXIST, as one without hard links '
            + 'refuses it, leaving neither a lock nor the claim\'s staged text behind',
          fn: async (ctx,) => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            /**
             What a filesystem without hard links refuses a link with.
             */
            const refusal = Object.assign(
              new Error('EPERM: operation not permitted, link',),
              { code: 'EPERM', },
            );
            /**
             `link` as it answers on that filesystem, for every call this case makes.
             */
            const withoutLinks = ctx.sinon.stub(
              fsPromises,
              'link',
            ).callsFake(async function linkRefused(): Promise<void> {
              throw refusal;
            },);
            syncBuiltinESMExports();
            // THE ESM BINDING OF `link` FOLLOWS THE STUB ONLY WHILE IT IS SYNCED,
            // so putting it back is a step of its own that the case's end runs.
            using _putBack = {
              [Symbol.dispose]: function restoreLink(): void {
                withoutLinks.restore();
                syncBuiltinESMExports();
              },
            };

            /**
             What the claim threw.
             */
            const thrown = await rejectionOf(async function locking(): Promise<unknown> {
              return await lockRunsDir({ runsDir, },);
            },);

            expect(thrown,).toBe(refusal,);
            expect(await readdir(runsDir,),).toEqual([],);
          },
        },),

        it({
          name: 'SAYS THE CLAIM\'S STAGED TEXT STANDS ONLY IF THE CLAIM WROTE IT, where removing it fails: a runs '
            + 'directory whose path leaves no room for the staged name refuses the claim with the system\'s own '
            + 'error, and the removal fails the same way',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            /**
             Runs directory whose lock file fits and whose staged name does not.
             */
            const runsDir = crowdedRunsDir({ root: scratch.path, },);
            /**
             What the claim threw, and what the lock warned meanwhile.
             */
            const {
              result: thrown,
              warned,
            } = await warnLinesDuring({
              run: async function locking(): Promise<unknown> {
                return await rejectionOf(async function claiming(): Promise<unknown> {
                  return await lockRunsDir({ runsDir, },);
                },);
              },
            },);
            /**
             The staged name the claim tried, which ends in a random id.
             */
            const staged = String(Reflect.get(
              Error.isError(thrown,) ? thrown : {},
              'path',
            ),);
            /**
             Units of the random id that ends the staged name.
             */
            const idUnits = 36;

            expect({
              thrown: String(thrown,),
              stagedBeforeId: staged.slice(0, -idUnits,),
              idGroupUnits: staged
                .slice(-idUnits,)
                .split('-',)
                .map((group,) => group.length),
              warned,
            },).toEqual({
              thrown: `Error: ENAMETOOLONG: name too long, open '${staged}'`,
              stagedBeforeId: `${join(runsDir, 'pass.lock',)}.claim-`,
              idGroupUnits: [
                8,
                4,
                4,
                4,
                12,
              ],
              warned: [
                `[runs-lock] ${staged} could not be removed after a claim (ENAMETOOLONG); it stands beside the lock `
                  + 'if the claim wrote it',
              ],
            },);
          },
        },),

        it({
          name: 'RECORDS NO IDENTITY in the lock it takes where the system answers no `/proc` file, as on a machine '
            + 'without one, so the lock holds its process id, start and token alone',
          fn: async (ctx,) => {
            await using scratch = await scratchDir({ prefix: 'runs-lock-', },);
            const runsDir = scratch.path;
            /**
             The real `readFile`, which every other path still reaches.
             */
            const realReadFile = fsPromises.readFile;
            /**
             `readFile` as it answers on a system with no `/proc`, for every
             file under it: the boot the host identity is read from and the
             status line this process's start is read from.
             */
            const withoutProc = ctx.sinon.stub(
              fsPromises,
              'readFile',
            ).callsFake(async function readFileWithNoProcFiles(path, options,) {
              if (((typeof path) === 'string') && path.startsWith('/proc/',)) {
                throw Object.assign(
                  new Error(`ENOENT: no such file or directory, open '${path}'`,),
                  { code: 'ENOENT', },
                );
              }
              return await realReadFile(
                path,
                options,
              );
            },);
            syncBuiltinESMExports();
            // THE ESM BINDING OF `readFile` FOLLOWS THE STUB ONLY WHILE IT IS SYNCED,
            // so putting it back is a step of its own that the case's end runs.
            using _putBack = {
              [Symbol.dispose]: function restoreReadFile(): void {
                withoutProc.restore();
                syncBuiltinESMExports();
              },
            };

            await using _lock = await lockRunsDir({ runsDir, },);

            /**
             What the lock records.
             */
            const recorded: unknown = JSON.parse(await realReadFile(
              join(
                runsDir,
                'pass.lock',
              ),
              'utf8',
            ),);
            expect(isJsonRecord(recorded,)
              ? {
                fields: Object.keys(recorded,),
                pid: recorded.pid,
              }
              : recorded,).toEqual({
                fields: [
                  'pid',
                  'startedAt',
                  'token',
                ],
                pid: process.pid,
              },);
          },
        },),
      ],
    },),

    describe({
      name: RunsDirectoryBusyError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS its lock file records nothing readable when constructed without a holder, the shape a '
            + 'race loss leaves once the winner\'s own lock could not be read back',
          fn: async () => {
            /**
             Constructed directly, as the race-loss branch does when the
             winner's lock could not be read, with no `holder` field at all.
             */
            const error = new RunsDirectoryBusyError({
              runsDir: '/mittens/runs',
              judgedBy: 'race',
            },);

            expect(error.message,).toBe(busyMessage({
              holderLine: '  its lock file records nothing readable',
            },),);
          },
        },),
        it({
          name: 'NAMES THE HOLDER\'S PROCESS AND START TIME when constructed with the holder the lock records, '
            + 'and does not say the lock file records nothing',
          fn: async () => {
            /**
             Constructed with the holder a race loser reads back from the
             winner's lock.
             */
            const error = new RunsDirectoryBusyError({
              runsDir: '/mittens/runs',
              holder: {
                pid: 1,
                startedAt: '2026-09-28T10:00:00.000Z',
                token: 't',
                identity: { kind: 'unrecorded', },
              },
              judgedBy: 'race',
            },);

            expect(error.message,).toBe(busyMessage({
              holderLine: '  process 1, since 2026-09-28T10:00:00.000Z',
            },),);
          },
        },),
      ],
    },),

    describe({
      name: lockFileText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES the lock line as the named fields alone where the identity is unrecorded, so a '
            + 'host with no identity to read still locks',
          fn: async () => {
            expect(lockFileText({
              holder: {
                pid: 1,
                startedAt: '2026-09-28T10:00:00.000Z',
                token: 't',
                identity: { kind: 'unrecorded', },
              },
            },),).toBe('{"pid":1,"startedAt":"2026-09-28T10:00:00.000Z","token":"t"}\n',);
          },
        },),
      ],
    },),
  ],
},);
