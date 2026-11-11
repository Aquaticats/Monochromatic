import { spawn, } from 'node:child_process';
import { createHash, } from 'node:crypto';
import { once, } from 'node:events';
import { existsSync, } from 'node:fs';
import {
  mkdtemp,
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

import { readLinuxProcessIdentity, } from '../dist/final/node/linux-process-identity.mjs';
import {
  stopBypassWatcher,
  waitForWatcherStop,
  watcherIsRunning,
} from '../dist/final/node/tunnel-bypass-watch-service.mjs';

/**
 Reader of the live process as the function under test declares it.
 */
type ProcessReader = NonNullable<Parameters<typeof watcherIsRunning>[0]['read']>;

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
        expect(await outcomeOf({ read: observing({ state: 'Z', commandLine: [], },), },),).toBe('false',);
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
          { state: 'Z', commandLine: [], },
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
          const observation = script[Math.min(seen.reads, script.length - 1,)] ?? { state: 'Z', commandLine: [], };
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

await describe({
  name: stopBypassWatcher.name,
  children: [
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
        expect(existsSync(`${statePath}.watcher.json`,),).toBe(false,);
      },
    },),
  ],
},);
