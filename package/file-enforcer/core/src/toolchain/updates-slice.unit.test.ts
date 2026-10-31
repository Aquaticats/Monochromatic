/**
 Tests that toolchain sync and update runs move into the clearnet updates slice only where that can work,
 and that the slice spelling matches the one `wg-quicker-exempt` attaches to.

 Every case uses a fake `systemd-run`; nothing creates a real scope, touches the real user manager,
 or reaches the network.
 */

import { readFile, } from 'node:fs/promises';
import { resolve, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  decideUpdatesSlice,
  insideUpdatesSlice,
  NOT_RELAUNCHED,
  relaunchInUpdatesSlice,
  runChild,
  systemdRunArguments,
  TOOLCHAIN_UPDATES_SLICE,
  UPDATES_SLICE_GUARD_ENVIRONMENT,
  UPDATES_SLICE_OPT_OUT_ENVIRONMENT,
  type ChildOutcome,
  type UpdatesSliceFacts,
} from '../../dist/final/node/index.mjs';

/**
 Facts for a Linux host with a user manager,
 outside the slice.
 */
const READY_FACTS: UpdatesSliceFacts = {
  platform: 'linux',
  env: { XDG_RUNTIME_DIR: '/run/user/1000', },
  cgroupText: '0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-ghostty.scope\n',
  userManagerPresent: true,
};

/**
 One recorded `systemd-run` call.
 */
type RecordedCall = {
  readonly args: readonly string[];
  readonly env: Readonly<NodeJS.ProcessEnv>;
  readonly quiet: boolean;
};

/**
 Builds a fake `systemd-run` that records its calls and answers with scripted exit codes.

 @param codes - Exit codes returned in order.

 @returns Recorded calls and the runner.

 @example
 ```ts
 const fake = fakeSystemd({ codes: [0, 7,] });
 ```
 */
function fakeSystemd(
  { codes, }: { readonly codes: readonly number[]; },
): {
  readonly calls: RecordedCall[];
  readonly run: (call: RecordedCall,) => Promise<ChildOutcome>;
} {
  /**
   Calls received so far.
   */
  const calls: RecordedCall[] = [];
  return {
    calls,
    run: function run(call,): Promise<ChildOutcome> {
      calls.push(call,);
      return Promise.resolve({ code: codes[calls.length - 1] ?? 0, },);
    },
  };
}

await describe({
  name: decideUpdatesSlice.name,
  children: [
    it({
      name: 'relaunches on a Linux host with a user manager outside the slice',
      fn: async () => {
        expect(decideUpdatesSlice(READY_FACTS,),).toEqual({ relaunch: true, },);
      },
    },),
    it({
      name: 'stays in place for another platform, a guarded child, an opt-out, a run already in the slice and a host without a user manager',
      fn: async () => {
        expect(decideUpdatesSlice({ ...READY_FACTS, platform: 'darwin', },).relaunch,).toBe(false,);
        expect(
          decideUpdatesSlice({ ...READY_FACTS, env: { [UPDATES_SLICE_GUARD_ENVIRONMENT]: '1', }, },).relaunch,
        ).toBe(false,);
        expect(
          decideUpdatesSlice({ ...READY_FACTS, env: { [UPDATES_SLICE_OPT_OUT_ENVIRONMENT]: 'off', }, },).relaunch,
        ).toBe(false,);
        expect(
          decideUpdatesSlice({
            ...READY_FACTS,
            cgroupText: `0::/user.slice/app.slice/${TOOLCHAIN_UPDATES_SLICE}/run-p1-i2.scope\n`,
          },).relaunch,
        ).toBe(false,);
        expect(decideUpdatesSlice({ ...READY_FACTS, userManagerPresent: false, },).relaunch,).toBe(false,);
      },
    },),
    it({
      name: 'treats any other opt-out value as no opt-out',
      fn: async () => {
        expect(
          decideUpdatesSlice({ ...READY_FACTS, env: { [UPDATES_SLICE_OPT_OUT_ENVIRONMENT]: 'on', }, },).relaunch,
        ).toBe(true,);
      },
    },),
  ],
},);

await describe({
  name: insideUpdatesSlice.name,
  children: [
    it({
      name: 'matches the slice as one whole path segment of the unified cgroup only',
      fn: async () => {
        expect(
          insideUpdatesSlice({ cgroupText: `0::/user.slice/app.slice/${TOOLCHAIN_UPDATES_SLICE}/run-p1.scope\n`, },),
        ).toBe(true,);
        expect(insideUpdatesSlice({ cgroupText: `0::/app.slice/${TOOLCHAIN_UPDATES_SLICE}\n`, },),).toBe(true,);
        expect(insideUpdatesSlice({ cgroupText: `0::/app.slice/${TOOLCHAIN_UPDATES_SLICE}-extra/x\n`, },),).toBe(false,);
        expect(insideUpdatesSlice({ cgroupText: `1:name=systemd:/${TOOLCHAIN_UPDATES_SLICE}/x\n`, },),).toBe(false,);
        expect(insideUpdatesSlice({ cgroupText: '', },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: systemdRunArguments.name,
  children: [
    it({
      name: 'places the command in a fresh user scope below the slice',
      fn: async () => {
        expect(systemdRunArguments({ command: ['node', 'sync.ts', '--global',], },),).toEqual([
          '--user',
          '--scope',
          '--quiet',
          '--collect',
          String.raw`--slice=app-monochromatic\x2dupdates.slice`,
          '--',
          'node',
          'sync.ts',
          '--global',
        ],);
      },
    },),
  ],
},);

await describe({
  name: relaunchInUpdatesSlice.name,
  children: [
    it({
      name: 'probes quietly, then runs the entry point with its arguments under the guard and returns its exit code',
      fn: async () => {
        const fake = fakeSystemd({ codes: [0, 3,], });
        expect(
          await relaunchInUpdatesSlice({
            argv: ['/usr/bin/node', '/repo/sync.ts', '--global',],
            execPath: '/usr/bin/node',
            execArgv: ['--no-warnings',],
            facts: READY_FACTS,
            runSystemd: fake.run,
          },),
        ).toBe(3,);
        expect(fake.calls,).toHaveLength(2,);
        expect(fake.calls[0]?.quiet,).toBe(true,);
        expect(fake.calls[0]?.args.slice(-2,),).toEqual(['/usr/bin/node', '--version',],);
        expect(fake.calls[0]?.env[UPDATES_SLICE_GUARD_ENVIRONMENT],).toBeUndefined();
        expect(fake.calls[1]?.quiet,).toBe(false,);
        expect(fake.calls[1]?.args.slice(6,),).toEqual(['/usr/bin/node', '--no-warnings', '/repo/sync.ts', '--global',],);
        expect(fake.calls[1]?.env[UPDATES_SLICE_GUARD_ENVIRONMENT],).toBe('1',);
        expect(fake.calls[1]?.env.XDG_RUNTIME_DIR,).toBe('/run/user/1000',);
      },
    },),
    it({
      name: 'stays in place without running the work when the probe fails',
      fn: async () => {
        const fake = fakeSystemd({ codes: [1,], });
        expect(
          await relaunchInUpdatesSlice({
            argv: ['/usr/bin/node', '/repo/sync.ts',],
            execPath: '/usr/bin/node',
            execArgv: [],
            facts: READY_FACTS,
            runSystemd: fake.run,
          },),
        ).toBe(NOT_RELAUNCHED,);
        expect(fake.calls,).toHaveLength(1,);
      },
    },),
    it({
      name: 'calls nothing when the decision is to stay in place',
      fn: async () => {
        const fake = fakeSystemd({ codes: [0,], });
        expect(
          await relaunchInUpdatesSlice({
            argv: ['/usr/bin/node', '/repo/sync.ts',],
            execPath: '/usr/bin/node',
            execArgv: [],
            facts: { ...READY_FACTS, userManagerPresent: false, },
            runSystemd: fake.run,
          },),
        ).toBe(NOT_RELAUNCHED,);
        expect(fake.calls,).toHaveLength(0,);
      },
    },),
  ],
},);

await describe({
  name: runChild.name,
  children: [
    it({
      name: 'returns a real child exit code, 127 for a missing executable and 128 plus the signal after a signal',
      fn: async () => {
        expect(
          await runChild({
            command: process.execPath,
            args: ['--eval', 'process.exit(3)',],
            env: process.env,
            quiet: true,
          },),
        ).toEqual({ code: 3, },);
        expect(
          await runChild({
            command: '/nonexistent/wg-quicker-test-command',
            args: [],
            env: process.env,
            quiet: true,
          },),
        ).toEqual({ code: 127, },);
        expect(
          await runChild({
            command: process.execPath,
            args: ['--eval', 'process.kill(process.pid, "SIGTERM")',],
            env: process.env,
            quiet: true,
          },),
        ).toEqual({ code: 143, },);
      },
    },),
  ],
},);

await describe({
  name: 'TOOLCHAIN_UPDATES_SLICE',
  children: [
    it({
      name: 'is spelled exactly as the Rust watcher matches it',
      fn: async () => {
        /**
         Rust source holding the matcher's constant.
         */
        const source = await readFile(
          resolve(
            import.meta.dirname,
            '..',
            '..',
            '..',
            '..',
            '..',
            'package',
            'wg',
            'cli-quicker-exempt',
            'src',
            'application_targets.rs',
          ),
          'utf8',
        );
        expect(source,).toContain(`pub const TOOLCHAIN_UPDATES_SLICE: &str = "${TOOLCHAIN_UPDATES_SLICE.replaceAll('\\', String.raw`\\`,)}";`,);
        expect(TOOLCHAIN_UPDATES_SLICE,).toBe(String.raw`app-monochromatic\x2dupdates.slice`,);
      },
    },),
  ],
},);
