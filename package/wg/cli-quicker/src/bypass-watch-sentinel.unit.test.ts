import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
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

import {
  judgeWatcherEnd,
  parseSentinelArguments,
  runWatcherSentinel,
  watcherIsAlive,
} from '../dist/final/node/bypass-watch-sentinel-core.mjs';
import { readSentinelMarker, } from '../dist/final/node/bypass-watch-sentinel-marker.mjs';
import {
  PROCESS_ABSENT,
  readLinuxProcessIdentity,
} from '../dist/final/node/linux-process-identity.mjs';

/**
 What a read of a process identity answers,
 as the sentinel core declares it.
 */
type ProcessRead = Awaited<ReturnType<NonNullable<Parameters<typeof watcherIsAlive>[0]['read']>>>;

/**
 What a read of the watcher's identity record answers,
 as the sentinel core declares it.
 */
type RecordRead = Awaited<ReturnType<NonNullable<Parameters<typeof judgeWatcherEnd>[0]['read']>>>;

/**
 Identity of the watcher the fake records name.
 */
const IDENTITY = {
  ownerId: 'owner-1',
  pid: 4_242,
  startTime: '9001',
} as const;

/**
 Target the sentinel waits for in the logic tests.
 */
const TARGET = {
  statePath: '/run/wg-quicker/wgtest.json',
  identity: IDENTITY,
} as const;

/**
 Builds a delivery that records what it was asked to deliver.

 @returns The delivery and the calls it recorded.

 @example
 ```ts
 const { deliver, calls } = recordingDelivery();
 ```
 */
function recordingDelivery() {
  /**
   Deliveries in order.
   */
  const calls: { readonly kind: string; readonly line: string; }[] = [];
  return {
    calls,
    deliver: (call: { readonly kind: string; readonly line: string; },) => {
      calls.push(call,);
      return Promise.resolve();
    },
  };
}

await describe({
  name: parseSentinelArguments.name,
  children: [
    it({
      name: 'reads the state path, owner, pid and start time, and refuses a wrong count or a pid that is not positive',
      fn: async () => {
        expect(parseSentinelArguments({ argv: ['/run/x.json', 'owner-1', '4242', '9001',], },),).toEqual({
          statePath: '/run/x.json',
          identity: { ownerId: 'owner-1', pid: 4_242, startTime: '9001', },
        },);
        expect(() => parseSentinelArguments({ argv: ['/run/x.json', 'owner-1', '4242',], },),).toThrow('Usage',);
        expect(() => parseSentinelArguments({ argv: ['/run/x.json', 'owner-1', '0', '9001',], },),).toThrow('Invalid sentinel arguments',);
        expect(() => parseSentinelArguments({ argv: ['/run/x.json', 'owner-1', 'abc', '9001',], },),).toThrow('Invalid sentinel arguments',);
        expect(() => parseSentinelArguments({ argv: ['/run/x.json', 'owner-1', '1.5', '9001',], },),).toThrow('Invalid sentinel arguments',);
      },
    },),
  ],
},);

await describe({
  name: watcherIsAlive.name,
  children: [
    it({
      name: 'is alive only for the same pid with the same start time',
      fn: async () => {
        /**
         Process identity with a chosen start time.
         */
        const live = (startTime: string,) => ({ commandLine: [], startTime, state: 'S', });
        expect(await watcherIsAlive({ identity: IDENTITY, read: () => Promise.resolve<ProcessRead>(live('9001',),), },),).toBe(true,);
        expect(await watcherIsAlive({ identity: IDENTITY, read: () => Promise.resolve<ProcessRead>(live('9002',),), },),).toBe(false,);
        /**
         A pid no process can have,
         read through the real reader,
         because an absence injected from this module would not be the symbol of the bundle under test.
         */
        const impossible = { ...IDENTITY, pid: 2_147_483_647, };
        expect(await watcherIsAlive({ identity: impossible, },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: judgeWatcherEnd.name,
  children: [
    it({
      name: 'calls the end expected when the record is gone, holds no identity or names another watcher, unexpected when it still names this one, and unknown when it cannot be read at all',
      fn: async () => {
        /**
         Judgement over a fake record reader.
         */
        const judge = (record: Error | RecordRead,) => judgeWatcherEnd({
          statePath: TARGET.statePath,
          identity: IDENTITY,
          read: () => (Error.isError(record,) ? Promise.reject(record,) : Promise.resolve<RecordRead>(record,)),
        },);
        /**
         A state path with no identity record,
         read through the real reader.
         */
        expect(await judgeWatcherEnd({ statePath: '/nonexistent-directory/wgtest.json', identity: IDENTITY, },),).toEqual({ kind: 'expected', },);
        expect(await judge({ kind: 'identity', identity: { ...IDENTITY, }, },),).toEqual({ kind: 'unexpected', },);
        expect(await judge({ kind: 'identity', identity: { ...IDENTITY, pid: 4_243, }, },),).toEqual({ kind: 'expected', },);
        expect(await judge({ kind: 'identity', identity: { ...IDENTITY, startTime: '9002', }, },),).toEqual({ kind: 'expected', },);
        expect(await judge({ kind: 'identity', identity: { ...IDENTITY, ownerId: 'owner-2', }, },),).toEqual({ kind: 'expected', },);
        // A fragment left by a killed writer names no watcher, so it is not this watcher's record either.
        expect(await judge({ kind: 'unreadable', path: '/run/wg-quicker/x.watcher.json', reason: 'SyntaxError', },),).toEqual({ kind: 'expected', },);
        expect(
          await judge(new Error('EACCES: permission denied',),),
        ).toEqual({ kind: 'unknown', reason: 'Error: EACCES: permission denied', },);
      },
    },),
  ],
},);

/**
 Runs a sentinel whose watcher ended while the whole system was stopping,
 and checks that nothing was reported and the probe was asked once.

 @param verdict - What the record said about the end.

 @example
 ```ts
 await expectNothingReportedDuringShutdown({ verdict: { kind: 'unexpected' } });
 ```
 */
async function expectNothingReportedDuringShutdown(
  { verdict, }: { readonly verdict: Awaited<ReturnType<typeof judgeWatcherEnd>>; },
): Promise<void> {
  /**
   Delivery and calls of a run during a shutdown.
   */
  const stopping = recordingDelivery();
  /**
   Whether the probe was asked.
   */
  const asked = { count: 0, };
  /**
   What the sentinel concluded.
   */
  const outcome = await runWatcherSentinel({
    target: TARGET,
    deliver: stopping.deliver,
    isAlive: () => Promise.resolve(false,),
    pause: () => Promise.resolve(),
    judge: () => Promise.resolve(verdict,),
    isSystemStopping: () => {
      asked.count += 1;
      return Promise.resolve(true,);
    },
  },);
  expect(outcome,).toEqual({ kind: 'system stopping', },);
  expect(stopping.calls,).toEqual([],);
  expect(asked.count,).toBe(1,);
}

await describe({
  name: runWatcherSentinel.name,
  children: [
    it({
      name: 'waits while the watcher lives, waits the grace, and delivers a report naming both policies when the record still names it',
      fn: async () => {
        const { deliver, calls, } = recordingDelivery();
        /**
         Liveness answers in order: alive twice, then gone.
         */
        const answers = [true, true, false,];
        /**
         Pauses requested.
         */
        const pauses: number[] = [];
        const verdict = await runWatcherSentinel({
          isSystemStopping: () => Promise.resolve(false,),
          target: TARGET,
          deliver,
          isAlive: () => Promise.resolve(answers.shift() ?? false,),
          pause: (milliseconds,) => {
            pauses.push(milliseconds,);
            return Promise.resolve();
          },
          judge: () => Promise.resolve({ kind: 'unexpected', } as const,),
          graceMs: 1_500,
          pollMs: 500,
        },);
        expect(verdict,).toEqual({ kind: 'unexpected', },);
        expect(pauses,).toEqual([500, 500, 1_500,],);
        expect(calls,).toHaveLength(1,);
        expect(calls[0]?.kind,).toBe('route watcher died',);
        expect(calls[0]?.line,).toContain('POLICY NOT IN FORCE',);
        expect(calls[0]?.line,).toContain('pid 4242',);
        expect(calls[0]?.line,).toContain('bypass table',);
        expect(calls[0]?.line,).toContain('clearnet hosts',);
        expect(calls[0]?.line,).toContain('wg-quicker up',);
      },
    },),
    it({
      name: 'says nothing when the end was asked for, and reports an unreadable record as unknown',
      fn: async () => {
        /**
         Delivery and calls of a run whose record is gone.
         */
        const expected = recordingDelivery();
        await runWatcherSentinel({
          isSystemStopping: () => Promise.resolve(false,),
          target: TARGET,
          deliver: expected.deliver,
          isAlive: () => Promise.resolve(false,),
          pause: () => Promise.resolve(),
          judge: () => Promise.resolve({ kind: 'expected', } as const,),
        },);
        expect(expected.calls,).toEqual([],);
        /**
         Delivery and calls of a run whose record cannot be read.
         */
        const unknown = recordingDelivery();
        await runWatcherSentinel({
          isSystemStopping: () => Promise.resolve(false,),
          target: TARGET,
          deliver: unknown.deliver,
          isAlive: () => Promise.resolve(false,),
          pause: () => Promise.resolve(),
          judge: () => Promise.resolve({ kind: 'unknown', reason: 'Error: EACCES', } as const,),
        },);
        expect(unknown.calls,).toHaveLength(1,);
        expect(unknown.calls[0]?.line,).toContain('cannot be read (Error: EACCES)',);
        expect(unknown.calls[0]?.line,).toContain('cannot be told whether wg-quicker asked for that',);
      },
    },),
    it({
      name: 'reports nothing when the end came while the whole system was stopping, for a record that names the watcher and for one that cannot be read',
      fn: async () => {
        await expectNothingReportedDuringShutdown({ verdict: { kind: 'unexpected', }, },);
        await expectNothingReportedDuringShutdown({ verdict: { kind: 'unknown', reason: 'Error: ENOENT', }, },);
      },
    },),
    it({
      name: 'does not ask whether the system is stopping when the end was asked for',
      fn: async () => {
        /**
         Whether the probe was asked.
         */
        const asked = { count: 0, };
        const outcome = await runWatcherSentinel({
          target: TARGET,
          deliver: recordingDelivery().deliver,
          isAlive: () => Promise.resolve(false,),
          pause: () => Promise.resolve(),
          judge: () => Promise.resolve({ kind: 'expected', } as const,),
          isSystemStopping: () => {
            asked.count += 1;
            return Promise.resolve(true,);
          },
        },);
        expect(outcome,).toEqual({ kind: 'expected', },);
        expect(asked.count,).toBe(0,);
      },
    },),
    it({
      name: 'waits for a real throwaway process through procfs and reports its unasked death',
      fn: async () => {
        /**
         Throwaway process standing in for the watcher.
         */
        const child = spawn('/usr/bin/sleep', ['60',], { stdio: 'ignore', },);
        await once(child, 'spawn',);
        const { pid, } = child;
        if (pid === undefined)
          throw new Error('The throwaway process has no pid.',);
        /**
         Its real procfs identity.
         */
        const live = await readLinuxProcessIdentity({ pid, },);
        if (live === PROCESS_ABSENT)
          throw new Error('The throwaway process is not in procfs.',);
        const { deliver, calls, } = recordingDelivery();
        /**
         Moment the process is killed.
         */
        const killed = { at: 0, };
        setTimeout(
          () => {
            killed.at = performance.now();
            child.kill('SIGKILL',);
          },
          300,
        );
        /**
         Sentinel over the real process with a short poll and grace.
         */
        const verdict = await runWatcherSentinel({
          isSystemStopping: () => Promise.resolve(false,),
          target: { statePath: TARGET.statePath, identity: { ownerId: 'owner-1', pid, startTime: live.startTime, }, },
          deliver,
          judge: () => Promise.resolve({ kind: 'unexpected', } as const,),
          graceMs: 50,
          pollMs: 25,
        },);
        expect(verdict,).toEqual({ kind: 'unexpected', },);
        expect(performance.now() - killed.at,).toBeLessThan(5_000,);
        expect(killed.at,).toBeGreaterThan(0,);
        expect(calls,).toHaveLength(1,);
        expect(calls[0]?.line,).toContain(`pid ${String(pid,)}`,);
      },
    },),
    it({
      name: 'runs as a detached process, finds its watcher gone and its record removed, and exits without a report',
      fn: async () => {
        /**
         Scratch directory for the state path.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-sentinel-',),);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         Throwaway process standing in for the watcher.
         */
        const child = spawn('/usr/bin/sleep', ['60',], { stdio: 'ignore', },);
        await once(child, 'spawn',);
        const { pid, } = child;
        if (pid === undefined)
          throw new Error('The throwaway process has no pid.',);
        /**
         Its real procfs identity.
         */
        const live = await readLinuxProcessIdentity({ pid, },);
        if (live === PROCESS_ABSENT)
          throw new Error('The throwaway process is not in procfs.',);
        /**
         State path whose identity record does not exist,
         as after a clean stop.
         */
        const statePath = join(directory, 'wgtest.json',);
        await writeFile(statePath, '{}',);
        /**
         The real sentinel entry from the built output.
         */
        const sentinel = spawn(
          process.execPath,
          [
            new URL('../dist/final/node/bypass-watch-sentinel.mjs', import.meta.url,).pathname,
            statePath,
            'owner-1',
            String(pid,),
            live.startTime,
          ],
          {
            stdio: 'ignore',
            env: { PATH: '/usr/bin:/bin', },
          },
        );
        setTimeout(
          () => {
            child.kill('SIGKILL',);
          },
          400,
        );
        /**
         Moment the sentinel was started.
         */
        const started = performance.now();
        /**
         Arguments of the exit event.
         */
        const exitArguments: readonly unknown[] = await once(sentinel, 'exit',);
        expect(exitArguments[0],).toBe(0,);
        expect(performance.now() - started,).toBeGreaterThan(400,);
        expect(performance.now() - started,).toBeLessThan(15_000,);
        /**
         The marker the real entry left when it started,
         which is how a launcher confirms the sentinel got as far as running.
         */
        const marker = await readSentinelMarker({ statePath, },);
        expect(marker.found,).toBe(true,);
        expect(marker.found ? marker.marker.watcherPid : -1,).toBe(pid,);
        expect(marker.found ? marker.marker.pid : -1,).toBe(sentinel.pid,);
      },
    },),
  ],
},);
