import { spawn, type ChildProcess, } from 'node:child_process';
import { createHash, } from 'node:crypto';
import { once, } from 'node:events';
import {
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  PROCESS_ZOMBIE_STATE,
  readLinuxProcessIdentity,
} from '../dist/final/node/linux-process-identity.mjs';
import {
  waitForWatcherStop,
  watcherIsRunning,
  watcherProcessGroupExists,
  watcherProcessGroupHasRunningMember,
} from '../dist/final/node/tunnel-bypass-watch-process-group.mjs';
import { stopBypassWatcher, } from '../dist/final/node/tunnel-bypass-watch-service.mjs';

/**
 Module logger for this file.
 */
const l = tagged({ tag: 'tunnel-bypass-watch-service-test', },);

/**
 Says whether a file is there.

 @param path - File to look for.

 @returns `present` or `removed`.

 @example
 ```ts
 await presenceOf('/run/wg-quicker/interface-x.json.watcher.json');
 ```
 */
async function presenceOf(path: string,): Promise<'present' | 'removed'> {
  try {
    await readFile(path, 'utf8',);
    return 'present';
  }
  catch (error: unknown) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT')
      return 'removed';
    throw error;
  }
}

/**
 Reader of the live process as the function under test declares it.
 */
type ProcessReader = NonNullable<Parameters<typeof watcherIsRunning>[0]['read']>;

/**
 Whether an answer is anything at all.

 @param answer - What a bounded read answered.

 @returns Whether the answer is not empty.

 @example
 ```ts
 nonempty('42');
 ```
 */
function nonempty(answer: string,): boolean {
  return answer !== '';
}

/**
 Waits until a read answers something the caller wants,
 and fails when it never does.

 The wait is bounded so a wrong state fails this file with a sentence instead of holding it.

 @param what - What the wait is for, for the failure's words.

 @param read - Reader tried again after each pause;
 an empty answer means "not yet".

 @param until - Whether an answer is the one waited for;
 any nonempty answer by default.

 @returns The answer it stopped at.

 @throws {@link Error} when no answer arrived within the bound.

 @example
 ```ts
 await pollFor({ what: 'the member pid', read: () => readFile(pidFile, 'utf8') });
 ```
 */
async function pollFor(
  {
    what,
    read,
    until = nonempty,
  }: {
    readonly what: string;
    readonly read: () => Promise<string>;
    readonly until?: (answer: string,) => boolean;
  },
): Promise<string> {
  /**
   Attempts made so far.
   */
  const attempts = { count: 0, };
  /* oxlint-disable eslint/no-await-in-loop -- Each attempt must follow the one before it. */
  while (attempts.count < 300) {
    /**
     What this attempt read.
     */
    const answer = await read();
    if (until(answer,))
      return answer;
    attempts.count += 1;
    await new Promise(function later(resolve,) {
      setTimeout(resolve, 20,);
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  throw new Error(`the wait for ${what} never saw it: the last read answered nothing usable.`,);
}

/**
 Script and state arguments a watcher is expected to run with.
 */
const WATCHER_PATH = '/package/bypass-watch.mjs';

/**
 State path argument of the fixture watcher.
 */
const STATE_PATH = '/run/wg-quicker/interface.json';

/**
 Persisted identity of the fixture watcher.
 */
const IDENTITY = {
  ownerId: 'owner',
  pid: 4_242,
  startTime: '9001',
} as const;

/**
 Builds a reader of the live process that answers with a fixed observation.

 @param state - State letter the process shows.

 @param commandLine - Command line the process shows.

 @param startTime - Start time the process shows.

 @returns The reader.

 @example
 ```ts
 const read = observing({ state: 'S', commandLine: ['/node', WATCHER_PATH, STATE_PATH] });
 ```
 */
function observing(
  {
    state,
    commandLine,
    startTime = IDENTITY.startTime,
  }: {
    readonly state: string;
    readonly commandLine: readonly string[];
    readonly startTime?: string;
  },
): ProcessReader {
  return function read(): ReturnType<ProcessReader> {
    return Promise.resolve({
      pid: IDENTITY.pid,
      state,
      startTime,
      commandLine,
    },);
  };
}

/**
 Runs the check and says whether it accepted the process or why it refused.

 @param read - Reader of the live process.

 @returns `true` or `false` as text when the check answered, or the refusal text.

 @example
 ```ts
 await outcomeOf({ read: observing({ state: 'S', commandLine: [] }) });
 ```
 */
async function outcomeOf({ read, }: { readonly read: ProcessReader; },): Promise<string> {
  try {
    return String(await watcherIsRunning({
      identity: IDENTITY,
      statePath: STATE_PATH,
      watcherPath: WATCHER_PATH,
      read,
    },),);
  }
  catch (error) {
    return String(error,);
  }
}

await describe({
  name: watcherIsRunning.name,
  children: [
    it({
      name: 'accepts the exact watcher script and state argument whatever node binary started it',
      fn: async () => {
        expect(
          await Promise.all(['/runtime/node-previous', '/runtime/node-current',].map(function accepted(executable,): Promise<string> {
            return outcomeOf({ read: observing({ state: 'S', commandLine: [executable, WATCHER_PATH, STATE_PATH,], },), },);
          },),),
        ).toEqual(['true', 'true',],);
      },
    },),
    it({
      name: 'counts a process that is exiting, whose command line already reads empty but whose state is still live, as no longer running and not as another command',
      fn: async () => {
        expect(
          await Promise.all(['R', 'S', 'D',].map(function exiting(state,): Promise<string> {
            return outcomeOf({ read: observing({ state, commandLine: [], },), },);
          },),),
        ).toEqual(['false', 'false', 'false',],);
      },
    },),
    it({
      name: 'still refuses a live process that runs another command, so another program is never signaled',
      fn: async () => {
        expect(await outcomeOf({ read: observing({ state: 'S', commandLine: ['/usr/bin/node', '/other/script.mjs', STATE_PATH,], },), },),)
          .toContain('Refusing to signal PID 4242',);
        expect(await outcomeOf({ read: observing({ state: 'S', commandLine: ['/usr/bin/node', WATCHER_PATH,], },), },),)
          .toContain('Refusing to signal PID 4242',);
      },
    },),
    it({
      name: 'says not running for another start time, a zombie, and a process that is gone',
      fn: async () => {
        expect(await outcomeOf({ read: observing({ state: 'S', startTime: '1', commandLine: ['/node', WATCHER_PATH, STATE_PATH,], },), },),).toBe('false',);
        expect(await outcomeOf({ read: observing({ state: PROCESS_ZOMBIE_STATE, commandLine: [], },), },),).toBe('false',);
        /**
         A child that ended, so its pid reads as gone.
         */
        const ended = spawn(process.execPath, ['--eval', '0',], { stdio: 'ignore', },);
        await once(ended, 'close',);
        const { pid, } = ended;
        expect(pid !== undefined,).toBe(true,);
        expect(await watcherIsRunning({
          identity: { ...IDENTITY, pid: pid ?? 0, },
          statePath: STATE_PATH,
          watcherPath: WATCHER_PATH,
        },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: waitForWatcherStop.name,
  children: [
    it({
      name: 'waits through a watcher that is exiting instead of failing the stop, as in the incident where the wait threw while the signaled watcher was still releasing its memory',
      fn: async () => {
        /**
         What successive reads of the dying watcher show:
         running, then exiting with an empty command line for a while, then a zombie.
         */
        const script = [
          { state: 'S', commandLine: ['/node', WATCHER_PATH, STATE_PATH,], },
          { state: 'S', commandLine: ['/node', WATCHER_PATH, STATE_PATH,], },
          { state: 'R', commandLine: [], },
          { state: 'R', commandLine: [], },
          { state: 'R', commandLine: [], },
          { state: PROCESS_ZOMBIE_STATE, commandLine: [], },
        ];
        /**
         Reads made so far and pauses taken.
         */
        const seen = { reads: 0, pauses: 0, };
        /**
         Reader that follows the script and repeats its last observation.

         @returns The observation for this read.
         */
        function scripted(): ReturnType<ProcessReader> {
          /**
           Observation for this read.
           */
          const observation = script[Math.min(seen.reads, script.length - 1,)] ?? { state: PROCESS_ZOMBIE_STATE, commandLine: [], };
          seen.reads += 1;
          return Promise.resolve({ pid: IDENTITY.pid, startTime: IDENTITY.startTime, ...observation, },);
        }
        /**
         Whether the watcher disappeared within the bound.
         */
        const stopped = await waitForWatcherStop({
          identity: IDENTITY,
          statePath: STATE_PATH,
          watcherPath: WATCHER_PATH,
          isRunning: (call,) => watcherIsRunning({ ...call, read: scripted, },),
          groupExists: () => seen.reads < 6,
          pause: () => {
            seen.pauses += 1;
            return Promise.resolve();
          },
        },);
        expect(stopped,).toBe(true,);
        expect(seen.pauses > 0,).toBe(true,);
      },
    },),
  ],
},);

/**
 A process group whose only member has exited and was never reaped.

 A holder starts the member as its own group, writes its pid into a file of the
 case's scratch directory, and stops itself.
 A stopped process runs no `wait`, so the exited member stays a task the kernel
 still counts in the group, and the kernel still answers `kill(-pgid, 0)` for it.
 That is the shape a stop finds when the process that adopted the member has not
 reached its `wait` yet, which is what a reaping process under load does.

 @param directory - Scratch directory of the case, which holds the pid file.

 @returns The member's pid and start time, with the guard that ends the fixture.

 @throws {@link Error} when the fixture never reaches the state the case needs.

 @example
 ```ts
 await using member = await zombieGroupMember({ directory });
 ```
 */
async function zombieGroupMember(
  { directory, }: { readonly directory: string; },
): Promise<{
  readonly pid: number;
  readonly startTime: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   File the holder writes the member's pid into before it stops itself.
   */
  const pidFile = join(directory, 'member-pid.txt',);
  /**
   Holder of the member: it starts the member as its own group,
   names it, and stops itself so that it cannot reap it.
   */
  const holder = spawn(process.execPath, [
    '--eval',
    `const { spawn } = require('node:child_process');
const { writeFileSync } = require('node:fs');
const member = spawn('sleep', ['60'], { detached: true, stdio: 'ignore' });
writeFileSync(${JSON.stringify(pidFile,)}, String(member.pid));
process.kill(process.pid, 'SIGSTOP');`,
  ], { stdio: 'ignore', },);
  /**
   The member's pid, once the holder has named it.
   */
  const named = await pollFor({
    what: 'the member pid',
    read: async () => {
      try {
        return (await readFile(pidFile, 'utf8',)).trim();
      }
      catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code === 'ENOENT')
          return '';
        throw error;
      }
    },
  },);
  /**
   The member as the kernel holds it.
   */
  const pid = Number(named,);
  return {
    pid,
    startTime: await stopHolderAndEndMember({ holder, pid, }),
    async [Symbol.asyncDispose](): Promise<void> {
      if (holder.pid !== undefined) {
        try {
          process.kill(holder.pid, 'SIGKILL',);
        }
        catch (error: unknown) {
          // The holder is gone already, which is what this cleanup is for.
          l.debug(`the holder was gone before the cleanup: ${String(error,)}`,);
        }
      }
      try {
        process.kill(-pid, 'SIGKILL',);
      }
      catch (error: unknown) {
        // The member's group is gone already, which is what this cleanup is for.
        l.debug(`the member's group was gone before the cleanup: ${String(error,)}`,);
      }
    },
  };
}

/**
 Waits for the holder to stop itself, ends the member, and answers the member's start time.

 @param holder - The process that owns the member and must not reap it.

 @param pid - The member's process identifier.

 @returns The member's start time, read while the kernel still lists it.

 @throws {@link Error} when the holder cannot be read or the member never becomes a zombie.

 @example
 ```ts
 await stopHolderAndEndMember({ holder, pid: 4242 });
 ```
 */
async function stopHolderAndEndMember(
  {
    holder,
    pid,
  }: {
    readonly holder: ChildProcess;
    readonly pid: number;
  },
): Promise<string> {
  // A holder that is still running when the member ends would reap it and leave no task behind.
  await pollFor({
    what: 'the holder to stop itself',
    read: async () => {
      /**
       Holder's state as procfs holds it, or nothing when procfs has no entry.
       */
      const live = await readLinuxProcessIdentity({ pid: holder.pid ?? 0, },);
      return ((typeof live) === 'symbol') ? '' : live.state;
    },
    until: (state,) => state === 'T',
  },);
  process.kill(pid, 'SIGKILL',);
  await pollFor({
    what: 'the member to become a zombie',
    read: async () => {
      /**
       Member's state as procfs holds it, or nothing when procfs has no entry.
       */
      const live = await readLinuxProcessIdentity({ pid, },);
      return ((typeof live) === 'symbol') ? '' : live.state;
    },
    until: (state,) => state === PROCESS_ZOMBIE_STATE,
  },);
  /**
   Identity the kernel still holds for the exited member.
   */
  const identity = await readLinuxProcessIdentity({ pid, },);
  if ((typeof identity) === 'symbol')
    throw new Error('The exited member left procfs, so its start time cannot be read.',);
  return identity.startTime;
}

await describe({
  name: watcherProcessGroupHasRunningMember.name,
  children: [
    it({
      name: 'says a group whose only member has exited has nothing left that can run, though the kernel still answers for the group until the member is reaped',
      fn: async () => {
        /**
         Scratch directory of this case.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-group-member-',),);
        await using scratch = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        await using member = await zombieGroupMember({ directory, },);
        // The kernel still answers for the group, which is what a stop read as a monitor still running.
        expect(watcherProcessGroupExists({ pid: member.pid, },),).toBe(true,);
        expect(await watcherProcessGroupHasRunningMember({ pid: member.pid, },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: stopBypassWatcher.name,
  // One at a time: each case owns the runtime directory of the process.
  concurrency: 1,
  children: [
    it({
      name: 'removes the record of a watcher whose group holds only a member that has exited, where the group still answering used to fail the stop',
      fn: async () => {
        /**
         Scratch runtime directory standing in for /run/wg-quicker.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-group-gone-',),);
        /**
         Runtime directory the process had before this test.
         */
        const previous = process.env.WG_QUICKER_RUNTIME_DIRECTORY;
        process.env.WG_QUICKER_RUNTIME_DIRECTORY = directory;
        await using scratch = {
          async [Symbol.asyncDispose](): Promise<void> {
            if (previous === undefined)
              delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
            else
              process.env.WG_QUICKER_RUNTIME_DIRECTORY = previous;
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        await using member = await zombieGroupMember({ directory, },);
        /**
         State path of the fixture interface.
         */
        const statePath = join(directory, `interface-${createHash('sha256',).update('wgfixture',).digest('hex',).slice(0, 32,)}.json`,);
        /**
         The state a completed up leaves behind.
         */
        const state = {
          version: 2,
          interfaceName: 'wgfixture',
          mark: 100,
          table: 52_000,
          preference: 50,
          ownerId: 'owner-1',
          routes: [],
        } as const;
        await writeFile(statePath, JSON.stringify(state,),);
        // The record names the exited member, so the stop finds a group the kernel still answers for.
        await writeFile(`${statePath}.watcher.json`, JSON.stringify({
          ownerId: state.ownerId,
          pid: member.pid,
          startTime: member.startTime,
        },),);
        expect(watcherProcessGroupExists({ pid: member.pid, },),).toBe(true,);
        await stopBypassWatcher({ state, },);
        expect(await presenceOf(`${statePath}.watcher.json`,),).toBe('removed',);
      },
    },),
    it({
      name: 'stops cleanly when the record holds no identity, keeps the fragment beside it, and lets the next stop and start through, where a kill during the write used to wedge every later lifecycle',
      fn: async () => {
        /**
         Scratch runtime directory standing in for /run/wg-quicker.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-record-',),);
        /**
         Runtime directory the process had before this test,
         and the search path the signal `isRunning` would need.
         */
        const previous = process.env.WG_QUICKER_RUNTIME_DIRECTORY;
        process.env.WG_QUICKER_RUNTIME_DIRECTORY = directory;
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            if (previous === undefined)
              delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
            else
              process.env.WG_QUICKER_RUNTIME_DIRECTORY = previous;
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         State path of the fixture interface.
         */
        const statePath = join(directory, `interface-${createHash('sha256',).update('wgfixture',).digest('hex',).slice(0, 32,)}.json`,);
        /**
         The state a completed up leaves behind.
         */
        const state = {
          version: 2,
          interfaceName: 'wgfixture',
          mark: 100,
          table: 52_000,
          preference: 50,
          ownerId: 'owner-1',
          routes: [],
        } as const;
        await writeFile(statePath, JSON.stringify(state,),);
        // The first byte of a record whose writer was killed, as the earlier in-place write left it.
        await writeFile(`${statePath}.watcher.json`, `{"ownerId":"owner-1"`,);
        await stopBypassWatcher({
          state,
          watcherPath: '/nonexistent/bypass-watch.mjs',
        },);
        // A start calls the stop first, so the second call is what a later up does.
        await stopBypassWatcher({
          state,
          watcherPath: '/nonexistent/bypass-watch.mjs',
        },);
        /**
         What the runtime directory holds now.
         */
        const left = await readdir(directory,);
        expect(left.filter((name,) => name.includes('.watcher.json.unreadable-',),).length,).toBe(1,);
        expect(left.includes(`${statePath.split('/').at(-1)}.watcher.json`,),).toBe(false,);
      },
    },),
    it({
      name: 'ends the process group of a watcher whose leader was killed, so the route monitor it started does not outlive it',
      fn: async () => {
        /**
         Scratch runtime directory standing in for /run/wg-quicker.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-group-',),);
        /**
         Runtime directory the process had before this test.
         */
        const previous = process.env.WG_QUICKER_RUNTIME_DIRECTORY;
        process.env.WG_QUICKER_RUNTIME_DIRECTORY = directory;
        /**
         The stand-in leader's pid,
         zero until it is started,
         so this test can end its group however the test ends.
         */
        const leader = { pid: 0, };
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            if (previous === undefined)
              delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
            else
              process.env.WG_QUICKER_RUNTIME_DIRECTORY = previous;
            if (leader.pid !== 0) {
              try {
                process.kill(-leader.pid, 'SIGKILL',);
              }
              catch (error: unknown) {
                l.debug(`the stand-in group was already gone: ${String(error,)}`,);
              }
            }
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         State path of the fixture interface.
         */
        const statePath = join(directory, `interface-${createHash('sha256',).update('wgfixture',).digest('hex',).slice(0, 32,)}.json`,);
        /**
         The state a completed up leaves behind.
         */
        const state = {
          version: 2,
          interfaceName: 'wgfixture',
          mark: 100,
          table: 52_000,
          preference: 50,
          ownerId: 'owner-1',
          routes: [],
        } as const;
        await writeFile(statePath, JSON.stringify(state,),);
        /**
         A stand-in watcher: a detached group leader with a child of its own,
         the shape the watcher and its `ip monitor route` have.
         */
        const script = join(directory, 'stand-in-watcher.cjs',);
        await writeFile(script, `
const { spawn } = require('node:child_process');
const child = spawn('sleep', ['60'], { stdio: 'ignore' });
process.stdout.write(String(child.pid));
setInterval(() => {}, 1000);
`,);
        const started = spawn(process.execPath, [script,], { detached: true, stdio: ['ignore', 'pipe', 'pipe'], },);
        leader.pid = started.pid ?? 0;
        /**
         What the stand-in leader said besides its first line,
         which is how a stand-in that could not start its child explains itself.
         */
        const diagnostics = { text: '', };
        started.stderr.on('data', (chunk) => {
          diagnostics.text += String(chunk,);
        });
        /**
         The child's pid, read from the leader's first line.
         The wait is bounded, so a silent stand-in fails this test instead of holding it.
         */
        const childPid = Number(await new Promise(function firstLineOrEnd(resolve,) {
          const timer = setTimeout(() => resolve('0',), 5_000,);
          started.stdout.once('data', (chunk,) => {
            clearTimeout(timer,);
            resolve(String(chunk,).trim());
          },);
          started.once('close', () => {
            clearTimeout(timer,);
            resolve('0',);
          },);
        },));
        if (childPid === 0)
          throw new Error(`The stand-in leader named no child (exit ${String(started.exitCode,)}): ${diagnostics.text}`,);
        /**
         Identity of the running leader.
         */
        const live = await readLinuxProcessIdentity({ pid: started.pid ?? 0, },);
        if ((typeof live) === 'symbol')
          throw new Error('The stand-in leader is not in procfs.',);
        await writeFile(`${statePath}.watcher.json`, JSON.stringify({
          ownerId: state.ownerId,
          pid: started.pid,
          startTime: live.startTime,
        },),);
        // Only the leader dies, as with SIGKILL to the watcher alone.
        process.kill(started.pid ?? 0, 'SIGKILL',);
        await once(started, 'close',);
        await stopBypassWatcher({ state, watcherPath: script, },);
        /**
         What became of the child the leader started.
         A monitor that exited cannot do anything,
         and one the kernel still holds as a zombie until its reaper collects it is that same claim:
         its socket and everything else it held are closed.
         */
        const child = await readLinuxProcessIdentity({ pid: childPid, },);
        expect((typeof child) === 'symbol' ? 'gone' : ((child.state === PROCESS_ZOMBIE_STATE) ? 'gone' : child.state),).toBe('gone',);
        expect(await presenceOf(`${statePath}.watcher.json`,),).toBe('removed',);
      },
    },),
    it({
      name: 'stops a real watcher whose state and record are in the format the build that started the live tunnel wrote, and removes its record',
      fn: async () => {
        /**
         Scratch runtime directory standing in for /run/wg-quicker.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-stop-',),);
        /**
         Runtime directory the process had before this test.
         */
        const previous = process.env.WG_QUICKER_RUNTIME_DIRECTORY;
        process.env.WG_QUICKER_RUNTIME_DIRECTORY = directory;
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            if (previous === undefined)
              delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
            else
              process.env.WG_QUICKER_RUNTIME_DIRECTORY = previous;
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         Stand-in watcher script that exits on SIGTERM.
         */
        const script = join(directory, 'stand-in-watcher.mjs',);
        await writeFile(script, "process.on('SIGTERM', () => process.exit(0)); setInterval(() => {}, 1000);\n",);
        /**
         State path the stop computes for the interface:
         the first 32 hex characters of the SHA-256 of its name, under the runtime directory.
         */
        const statePath = join(directory, `interface-${createHash('sha256',).update('wgfixture',).digest('hex',).slice(0, 32,)}.json`,);
        /**
         Detached stand-in watcher, as the build starts the real one.
         */
        const watcher = spawn(process.execPath, [script, statePath,], { detached: true, stdio: 'ignore', },);
        watcher.unref();
        const { pid, } = watcher;
        if (pid === undefined)
          throw new Error('The stand-in watcher has no pid.',);
        /**
         Identity of the running stand-in.
         */
        const live = await readLinuxProcessIdentity({ pid, },);
        if ((typeof live) === 'symbol')
          throw new Error('The stand-in watcher is not in procfs.',);
        /**
         State text exactly as `557ea24e24` serialized it (key order included).
         */
        const stateText = JSON.stringify({
          version: 2,
          interfaceName: 'wgfixture',
          mark: 100,
          table: 52_000,
          preference: 50,
          ownerId: 'owner-1',
          routes: [{ proto: '-4', tokens: ['default', 'via', '192.168.1.1', 'dev', 'wlp9s0', 'table', '52000', 'proto', '201',], },],
        },);
        await writeFile(statePath, stateText,);
        await writeFile(`${statePath}.watcher.json`, `{"ownerId":"owner-1","pid":${String(pid,)},"startTime":"${live.startTime}"}`,);
        /**
         The persisted state as the stop reads it.
         */
        const state = JSON.parse(stateText,) as Parameters<typeof stopBypassWatcher>[0]['state'];
        await stopBypassWatcher({ state, watcherPath: script, },);
        /**
         What the stand-in looks like after the stop.
         */
        const after = await readLinuxProcessIdentity({ pid, },);
        expect(['gone', 'Z',],).toContain((typeof after) === 'symbol' ? 'gone' : after.state,);
        expect(await presenceOf(`${statePath}.watcher.json`,),).toBe('removed',);
      },
    },),
  ],
},);
