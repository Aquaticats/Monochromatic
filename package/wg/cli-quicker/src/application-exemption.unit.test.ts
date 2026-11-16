import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applicationWatchStartArgs,
  applicationWatchStopArgs,
  resolveApplicationUid,
  startApplicationExemptions,
  startExemptionsOrAbort,
  startupFailureLines,
  stopApplicationExemptions,
} from '../dist/final/node/application-exemption.mjs';
import type { ApplicationExemptionCommand, } from '../dist/final/node/application-exemption-command.mjs';
import {
  POLICY_PROCESS_WATCH,
  POLICY_SOCKET_MARKING,
} from '../dist/final/node/update-policy-notice.mjs';

/**
 Resolution that finds nothing,
 after searching two root-controlled locations.
 */
const NOT_FOUND: ApplicationExemptionCommand = {
  found: false,
  searched: ['/repo/package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt', '/usr/bin/wg-quicker-exempt',],
};

/**
 Reads what a call throws.

 @param run - Call expected to throw.

 @returns The thrown value,
 or nothing when it did not throw.

 @example
 ```ts
 await thrownBy({ run: () => Promise.reject(new Error('x')) });
 ```
 */
async function thrownBy({ run, }: { readonly run: () => unknown; },): Promise<unknown> {
  try {
    await run();
    return undefined;
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: resolveApplicationUid.name,
  children: [
    it({
      name: 'builds exact watcher start arguments',
      fn: async () => {
        expect(applicationWatchStartArgs({
          interfaceName: 'wg0',
          mark: 100,
          uid: 1_000,
        },),).toEqual([
          'watch-start',
          'wg0',
          '100',
          '1000',
        ],);
      },
    },),
    it({
      name: 'builds exact watcher stop arguments',
      fn: async () => {
        expect(applicationWatchStopArgs({ interfaceName: 'wg0', },),).toEqual([
          'watch-stop',
          'wg0',
        ],);
      },
    },),
    it({
      name: 'ignores an explicit user override in the environment and follows the sudo identity',
      fn: async () => {
        expect(resolveApplicationUid({
          environment: {
            WG_QUICKER_EXEMPT_UID: '2000',
            SUDO_UID: '1000',
          } as { readonly SUDO_UID: string; },
          currentUid: 0,
        },),).toBe(1_000,);
        expect(() => resolveApplicationUid({
          environment: { WG_QUICKER_EXEMPT_UID: '2000', } as Record<string, string>,
          currentUid: 0,
        },),).toThrow('Application exemptions require SUDO_UID',);
      },
    },),
    it({
      name: 'uses original sudo user for root process',
      fn: async () => {
        expect(resolveApplicationUid({
          environment: { SUDO_UID: '1000', },
          currentUid: 0,
        },),).toBe(1_000,);
      },
    },),
    it({
      name: 'uses non-root effective user without environment identity',
      fn: async () => {
        expect(resolveApplicationUid({
          environment: {},
          currentUid: 1_001,
        },),).toBe(1_001,);
      },
    },),
    it({
      name: 'rejects ambiguous direct-root execution',
      fn: async () => {
        expect(() => resolveApplicationUid({
          environment: {},
          currentUid: 0,
        },),).toThrow('Application exemptions require SUDO_UID',);
      },
    },),
    it({
      name: 'rejects non-decimal UID',
      fn: async () => {
        expect(() => resolveApplicationUid({
          environment: { SUDO_UID: '1e3', },
          currentUid: 0,
        },),).toThrow('not decimal',);
      },
    },),
    it({
      name: 'rejects UID outside unsigned range',
      fn: async () => {
        expect(() => resolveApplicationUid({
          environment: { SUDO_UID: '4294967296', },
          currentUid: 0,
        },),).toThrow('outside unsigned 32-bit range',);
      },
    },),
  ],
},);

await describe({
  name: startupFailureLines.name,
  children: [
    it({
      name: 'relays only the loud reports of the watcher start, so a failed policy reaches the output of up',
      fn: async () => {
        /**
         Standard error of a start that printed its notice and one failure.
         */
        const stderr = [
          'wg-quicker-exempt: POLICY NOTICE. By the repository\'s policy, and without any switch, this traffic leaves outside the tunnel',
          'wg-quicker-exempt: POLICY NOT IN FORCE: process watch: update entry points: the updates slice cannot be created: boom',
          '',
        ].join('\n',);
        expect(startupFailureLines({ stderr, },),).toEqual([
          'wg-quicker-exempt: POLICY NOT IN FORCE: process watch: update entry points: the updates slice cannot be created: boom',
        ],);
        expect(startupFailureLines({ stderr: '', },),).toEqual([],);
        expect(startupFailureLines({ stderr: 'moved pid 5\n', },),).toEqual([],);
        /**
         A loud line that embeds a name with an escape sequence and a direction mark.
         */
        const hostile = startupFailureLines({ stderr: 'wg-quicker-exempt: POLICY NOT IN FORCE: x \u001B[2J\u001B]0;pwned\u0007 \u202E done\n', },);
        expect(hostile,).toEqual([String.raw`wg-quicker-exempt: POLICY NOT IN FORCE: x \u{001b}[2J\u{001b}]0;pwned\u{0007} \u{202e} done`,],);
      },
    },),
  ],
},);

await describe({
  name: startApplicationExemptions.name,
  children: [
    it({
      name: 'does not start, does not throw and returns the message with every searched location when the companion is missing',
      fn: async () => {
        /**
         Commands the start tried to run.
         */
        const ran: string[] = [];
        /**
         Outcome of a start without a companion.
         */
        const outcome = await startApplicationExemptions({
          interfaceName: 'wg0',
          mark: 100,
          resolveCommand: () => Promise.resolve(NOT_FOUND,),
          runCommand: (call,) => {
            ran.push(call.command,);
            return Promise.resolve({ stdout: '', stderr: '', },);
          },
        },);
        expect(ran,).toEqual([],);
        expect(outcome.ok,).toBe(false,);
        /**
         Message carried by the failed outcome.
         */
        const message = outcome.ok ? '' : outcome.message;
        expect(message,).toContain('/usr/bin/wg-quicker-exempt',);
        expect(message,).toContain('The caller\'s own PATH is never searched',);
        expect(message,).toContain('mise run //package/wg/cli-quicker-exempt:build',);
        expect(outcome.ok ? [] : outcome.searched,).toEqual([
          '/repo/package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt',
          '/usr/bin/wg-quicker-exempt',
        ],);
      },
    },),
    it({
      name: 'runs the companion that was found with the watcher arguments for the caller',
      fn: async () => {
        /**
         Commands the start ran.
         */
        const ran: { readonly command: string; readonly args: readonly string[]; }[] = [];
        /**
         Outcome of a start with a companion.
         */
        const outcome = await startApplicationExemptions({
          interfaceName: 'wg0',
          mark: 100,
          resolveCommand: () => Promise.resolve({ found: true, path: '/usr/bin/wg-quicker-exempt', },),
          runCommand: (call,) => {
            ran.push({ command: call.command, args: call.args, },);
            return Promise.resolve({ stdout: '', stderr: '', },);
          },
        },);
        expect(outcome,).toEqual({ ok: true, },);
        expect(ran,).toHaveLength(1,);
        expect(ran[0]?.command,).toBe('/usr/bin/wg-quicker-exempt',);
        expect(ran[0]?.args.slice(0, 3,),).toEqual(['watch-start', 'wg0', '100',],);
      },
    },),
  ],
},);

await describe({
  name: startExemptionsOrAbort.name,
  children: [
    it({
      name: 'throws the missing-watcher error naming the searched paths and how to install, and runs nothing, when the companion is missing',
      fn: async () => {
        /**
         Commands the start tried to run.
         */
        const ran: string[] = [];
        /**
         Error the abort threw.
         */
        const error = await thrownBy({
          run: () => startExemptionsOrAbort({
            interfaceName: 'wg0',
            mark: 100,
            start: (call,) => startApplicationExemptions({
              ...call,
              resolveCommand: () => Promise.resolve(NOT_FOUND,),
              runCommand: (command,) => {
                ran.push(command.command,);
                return Promise.resolve({ stdout: '', stderr: '', },);
              },
            },),
          },),
        },);
        expect(ran,).toEqual([],);
        expect(Error.isError(error,),).toBe(true,);
        expect((error as Error).name,).toBe('WatcherMissingError',);
        expect((error as Error).message,).toContain('/repo/package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt, /usr/bin/wg-quicker-exempt',);
        expect((error as Error).message,).toContain('mise run //package/wg/cli-quicker-exempt:build',);
        expect((error as Error).message,).toContain('rolled back',);
      },
    },),
    it({
      name: 'returns normally and runs the watcher start when the companion is found',
      fn: async () => {
        /**
         Commands the start ran.
         */
        const ran: string[] = [];
        await startExemptionsOrAbort({
          interfaceName: 'wg0',
          mark: 100,
          start: (call,) => startApplicationExemptions({
            ...call,
            resolveCommand: () => Promise.resolve({ found: true, path: '/usr/bin/wg-quicker-exempt', },),
            runCommand: (command,) => {
              ran.push(command.command,);
              return Promise.resolve({ stdout: '', stderr: '', },);
            },
          },),
        },);
        expect(ran,).toEqual(['/usr/bin/wg-quicker-exempt',],);
      },
    },),
  ],
},);

await describe({
  name: stopApplicationExemptions.name,
  children: [
    it({
      name: 'reports instead of throwing when the companion is missing, so teardown is never blocked',
      fn: async () => {
        /**
         Outcome of a stop without a companion.
         */
        const outcome = await stopApplicationExemptions({
          interfaceName: 'wg0',
          configured: true,
          resolveCommand: () => Promise.resolve(NOT_FOUND,),
          runCommand: () => Promise.reject(new Error('must not run',),),
        },);
        expect(outcome.ok,).toBe(false,);
        expect(outcome.ok ? '' : outcome.report,).toContain('cannot stop a watcher an earlier run may have started',);
      },
    },),
    it({
      name: 'stops through the companion that was found',
      fn: async () => {
        /**
         Arguments the stop ran with.
         */
        const ran: string[][] = [];
        /**
         Outcome of a stop with a companion.
         */
        const outcome = await stopApplicationExemptions({
          interfaceName: 'wg0',
          configured: true,
          resolveCommand: () => Promise.resolve({ found: true, path: '/usr/bin/wg-quicker-exempt', },),
          runCommand: (call,) => {
            ran.push([call.command, ...call.args,],);
            return Promise.resolve({ stdout: '', stderr: '', },);
          },
        },);
        expect(outcome,).toEqual({ ok: true, },);
        expect(ran,).toEqual([['/usr/bin/wg-quicker-exempt', 'watch-stop', 'wg0',],],);
      },
    },),
  ],
},);
