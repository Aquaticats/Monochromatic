import {
  chmod,
  mkdtemp,
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

import {
  createRepeatLimit,
  failureKey,
  withHeldRepeats,
} from '../dist/final/node/failure-repeat-limit.mjs';
import {
  createFailureDelivery,
  FAILURE_JOURNAL_INTERVAL_MS,
  FAILURE_NOTIFICATION_INTERVAL_MS,
  journalCommand,
  notificationArguments,
  notificationCommand,
  resolveNotificationUser,
  runDeliveryCommandWithin,
  type DeliveryCommand,
} from '../dist/final/node/runtime-failure-report.mjs';

/**
 A user to notify in the delivery tests.
 */
const USER = { uid: 1_000, gid: 1_001, } as const;

/**
 Builds a delivery over a fake runner and a fake clock.

 @param lookupFound - Whether a user to notify exists.

 @param failing - Command names whose run rejects.

 @returns The delivery,
 the commands it ran,
 the channel failures it reported,
 and a setter for the clock.

 @example
 ```ts
 const fixture = delivery({ lookupFound: true, failing: [] });
 ```
 */
function delivery(
  {
    lookupFound,
    failing,
  }: {
    readonly lookupFound: boolean;
    readonly failing: readonly string[];
  },
) {
  /**
   Commands the fake runner ran.
   */
  const ran: DeliveryCommand[] = [];
  /**
   Channel failures reported.
   */
  const problems: string[] = [];
  /**
   Clock value in milliseconds.
   */
  const clock = { now: 0, };
  return {
    ran,
    problems,
    clock,
    deliver: createFailureDelivery({
      lookup: lookupFound
        ? { found: true, user: USER, }
        : { found: false, reason: 'no user to notify: SUDO_UID is not set', },
      run: (call,) => {
        ran.push(call,);
        return failing.includes(call.command,) ? Promise.reject(new Error(`${call.command} refused`,),) : Promise.resolve();
      },
      now: () => clock.now,
      asRoot: true,
      reportChannelFailure: (message,) => {
        problems.push(message,);
      },
    },),
  };
}

await describe({
  name: resolveNotificationUser.name,
  children: [
    it({
      name: 'names the sudo caller and refuses a missing or malformed id, saying which one',
      fn: async () => {
        expect(resolveNotificationUser({ environment: { SUDO_UID: '1000', SUDO_GID: '1001', }, },),).toEqual({
          found: true,
          user: { uid: 1_000, gid: 1_001, },
        },);
        expect(resolveNotificationUser({ environment: {}, },),).toEqual({
          found: false,
          reason: 'no user to notify: SUDO_UID is not set',
        },);
        expect(resolveNotificationUser({ environment: { SUDO_UID: '1000', }, },),).toEqual({
          found: false,
          reason: 'no user to notify: SUDO_GID is not set',
        },);
        expect(resolveNotificationUser({ environment: { SUDO_UID: '1e3', SUDO_GID: '1', }, },),).toEqual({
          found: false,
          reason: 'no user to notify: SUDO_UID is not a decimal id: 1e3',
        },);
        expect(resolveNotificationUser({ environment: { SUDO_UID: '1', SUDO_GID: '', }, },),).toEqual({
          found: false,
          reason: 'no user to notify: SUDO_GID is not a decimal id: ',
        },);
      },
    },),
  ],
},);

await describe({
  name: 'delivery commands',
  children: [
    it({
      name: 'write the journal line at error level under the wg-quicker identifier',
      fn: async () => {
        expect(journalCommand({ line: 'POLICY NOT IN FORCE: x', },),).toEqual({
          command: 'systemd-cat',
          args: ['--identifier=wg-quicker', '--priority=err',],
          input: 'POLICY NOT IN FORCE: x\n',
        },);
      },
    },),
    it({
      name: 'drop root to the user with a cleared environment, and call the client directly when not root',
      fn: async () => {
        /**
         Command of a root watcher.
         */
        const root = notificationCommand({ user: USER, line: 'text', asRoot: true, },);
        expect(root.command,).toBe('setpriv',);
        expect(root.args.slice(0, 7,),).toEqual([
          '--reuid',
          '1000',
          '--regid',
          '1001',
          '--clear-groups',
          '--no-new-privs',
          'env',
        ],);
        expect(root.args.slice(7, 10,),).toEqual([
          '--ignore-environment',
          'PATH=/usr/bin:/bin',
          'XDG_RUNTIME_DIR=/run/user/1000',
        ],);
        expect(root.args[10],).toBe('busctl',);
        expect(root.args.slice(11,),).toEqual(notificationArguments({
          summary: 'wg-quicker: update traffic policy not in force',
          body: 'text',
        },),);
        /**
         Command of a watcher that already runs as the user.
         */
        const direct = notificationCommand({ user: USER, line: 'text', asRoot: false, },);
        expect(direct.command,).toBe('env',);
        expect(direct.args.slice(0, 3,),).toEqual(['--ignore-environment', 'PATH=/usr/bin:/bin', 'XDG_RUNTIME_DIR=/run/user/1000',],);
      },
    },),
    it({
      name: 'follow the Notify signature of the freedesktop notification service',
      fn: async () => {
        /**
         Arguments for one notification.
         */
        const args = notificationArguments({ summary: 'summary text', body: 'body text', },);
        expect(args.slice(0, 2,),).toEqual(['--user', 'call',],);
        expect(args[2],).toBe('org.freedesktop.Notifications',);
        expect(args[5],).toBe('Notify',);
        expect(args[6],).toBe('susssasa{sv}i',);
        expect(args.slice(7,),).toEqual(['wg-quicker', '0', 'dialog-warning', 'summary text', 'body text', '0', '0', '0',],);
      },
    },),
  ],
},);

await describe({
  name: createRepeatLimit.name,
  children: [
    it({
      name: 'holds back repeats inside the interval, counts them, and reports the count at the next delivery',
      fn: async () => {
        /**
         Limit of one minute.
         */
        const limit = createRepeatLimit({ intervalMs: 60_000, },);
        expect(limit({ key: 'a', now: 0, },),).toEqual({ deliver: true, held: 0, },);
        expect(limit({ key: 'a', now: 10_000, },),).toEqual({ deliver: false, },);
        expect(limit({ key: 'a', now: 59_999, },),).toEqual({ deliver: false, },);
        expect(limit({ key: 'b', now: 59_999, },),).toEqual({ deliver: true, held: 0, },);
        expect(limit({ key: 'a', now: 60_000, },),).toEqual({ deliver: true, held: 2, },);
        expect(limit({ key: 'a', now: 120_000, },),).toEqual({ deliver: true, held: 0, },);
      },
    },),
    it({
      name: 'forgets expired failures when too many are remembered, so memory stays bounded',
      fn: async () => {
        /**
         Limit that remembers up to 256 failures.
         */
        const limit = createRepeatLimit({ intervalMs: 1_000, },);
        for (let index = 0; index < 300; index += 1)
          limit({ key: `failure ${String(index,)}`, now: index, },);
        expect(limit({ key: 'failure 0', now: 5_000, },),).toEqual({ deliver: true, held: 0, },);
      },
    },),
  ],
},);

await describe({
  name: 'failure keys and held repeats',
  children: [
    it({
      name: 'ignore digits, and word the held-back count',
      fn: async () => {
        expect(failureKey({ line: 'retry 12 in 30 s', },),).toBe(failureKey({ line: 'retry 7 in 9 s', },),);
        expect(failureKey({ line: 'retry in 30 s', },),).not.toBe(failureKey({ line: 'give up in 30 s', },),);
        expect(withHeldRepeats({ line: 'x', held: 0, },),).toBe('x',);
        expect(withHeldRepeats({ line: 'x', held: 3, },),).toBe('x (repeated 3 more time(s) since the last report)',);
      },
    },),
  ],
},);

await describe({
  name: createFailureDelivery.name,
  children: [
    it({
      name: 'writes a new failure to the journal and the desktop',
      fn: async () => {
        const { deliver, ran, problems, } = delivery({ lookupFound: true, failing: [], },);
        await deliver({ line: 'POLICY NOT IN FORCE: update hosts: x', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv',],);
        expect(ran[0]?.input,).toBe('POLICY NOT IN FORCE: update hosts: x\n',);
        expect(problems,).toEqual([],);
      },
    },),
    it({
      name: 'limits the journal to a minute and the desktop to ten minutes, and counts what it held back',
      fn: async () => {
        const { deliver, ran, clock, } = delivery({ lookupFound: true, failing: [], },);
        await deliver({ line: 'refresh 1 failed: boom', },);
        clock.now = 30_000;
        await deliver({ line: 'refresh 2 failed: boom', },);
        expect(ran,).toHaveLength(2,);
        clock.now = FAILURE_JOURNAL_INTERVAL_MS + 1;
        await deliver({ line: 'refresh 3 failed: boom', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv', 'systemd-cat',],);
        expect(ran[2]?.input,).toBe('refresh 3 failed: boom (repeated 1 more time(s) since the last report)\n',);
        clock.now = FAILURE_NOTIFICATION_INTERVAL_MS + 1;
        await deliver({ line: 'refresh 4 failed: boom', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv', 'systemd-cat', 'systemd-cat', 'setpriv',],);
        expect(ran[4]?.args.at(-5),).toBe('wg-quicker: update traffic policy not in force',);
        expect(ran[4]?.args.at(-4),).toBe('refresh 4 failed: boom (repeated 2 more time(s) since the last report)',);
      },
    },),
    it({
      name: 'delivers a different failure at once',
      fn: async () => {
        const { deliver, ran, clock, } = delivery({ lookupFound: true, failing: [], },);
        await deliver({ line: 'the route monitor failed', },);
        clock.now = 1;
        await deliver({ line: 'the update-host refresh failed', },);
        expect(ran,).toHaveLength(4,);
      },
    },),
    it({
      name: 'names a failing journal and still tries the desktop, and the reverse',
      fn: async () => {
        /**
         Delivery whose journal command fails.
         */
        const journalDown = delivery({ lookupFound: true, failing: ['systemd-cat',], },);
        await journalDown.deliver({ line: 'POLICY NOT IN FORCE: x', },);
        expect(journalDown.ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv',],);
        expect(journalDown.problems,).toHaveLength(1,);
        expect(journalDown.problems[0],).toContain('journal: Error: systemd-cat refused',);
        expect(journalDown.problems[0],).toContain('report was: POLICY NOT IN FORCE: x',);
        /**
         Delivery whose desktop command fails.
         */
        const desktopDown = delivery({ lookupFound: true, failing: ['setpriv',], },);
        await desktopDown.deliver({ line: 'POLICY NOT IN FORCE: y', },);
        expect(desktopDown.problems[0],).toContain('desktop notification: Error: setpriv refused',);
      },
    },),
    it({
      name: 'journals a failure and names the missing user when there is nobody to notify',
      fn: async () => {
        const { deliver, ran, problems, } = delivery({ lookupFound: false, failing: [], },);
        await deliver({ line: 'POLICY NOT IN FORCE: z', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat',],);
        expect(problems,).toHaveLength(1,);
        expect(problems[0],).toContain('desktop notification: no user to notify: SUDO_UID is not set',);
      },
    },),
  ],
},);

await describe({
  name: runDeliveryCommandWithin.name,
  children: [
    it({
      name: 'pipes the input, clears the environment, rejects a nonzero exit and kills a command past its timeout',
      fn: async () => {
        /**
         Scratch directory for the fake client.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-run-',),);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         Files the fake client writes.
         */
        const stdinFile = join(directory, 'stdin',);
        const envFile = join(directory, 'env',);
        /**
         Fake client recording its standard in and environment.
         */
        const recorder = join(directory, 'recorder',);
        await writeFile(recorder, `#!/bin/sh\n/usr/bin/cat > '${stdinFile}'\n/usr/bin/env > '${envFile}'\n`,);
        await chmod(recorder, 0o700,);
        await runDeliveryCommandWithin({ call: { command: recorder, args: [], input: 'one line\n', }, timeoutMs: 5_000, },);
        expect(await readFile(stdinFile, 'utf8',),).toBe('one line\n',);
        /**
         Names in the environment of the child:
         the secure path we set,
         and the three variables `/bin/sh` adds by itself.
         */
        const names = (await readFile(envFile, 'utf8',)).trim()
          .split('\n',)
          .map((line,) => line.split('=',)
            .at(0,) ?? '',)
          .toSorted((left, right,) => left.localeCompare(right,),);
        expect(names,).toEqual(['_', 'PATH', 'PWD', 'SHLVL',],);
        expect(await readFile(envFile, 'utf8',),).toContain('PATH=/usr/bin:/bin',);
        /**
         Fake client that fails.
         */
        const failing = join(directory, 'failing',);
        await writeFile(failing, '#!/bin/sh\necho boom >&2\nexit 3\n',);
        await chmod(failing, 0o700,);
        await expect(runDeliveryCommandWithin({ call: { command: failing, args: [], }, timeoutMs: 5_000, },),).rejects.toThrow();
        /**
         Fake client that outlives its timeout.
         */
        const hanging = join(directory, 'hanging',);
        await writeFile(hanging, '#!/bin/sh\nexec /usr/bin/sleep 30\n',);
        await chmod(hanging, 0o700,);
        /**
         Moment the hanging run began.
         */
        const started = performance.now();
        await expect(runDeliveryCommandWithin({ call: { command: hanging, args: [], }, timeoutMs: 200, },),).rejects.toThrow();
        expect(performance.now() - started,).toBeLessThan(10_000,);
        await expect(runDeliveryCommandWithin({ call: { command: join(directory, 'absent',), args: [], }, timeoutMs: 200, },),).rejects.toThrow();
      },
    },),
  ],
},);
