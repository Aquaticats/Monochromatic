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
      },
    },),
  ],
},);

await describe({
  name: startApplicationExemptions.name,
  children: [
    it({
      name: 'does not start, does not throw and reports both policies as not in force when the companion is missing',
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
         Report carried by the failed outcome.
         */
        const report = outcome.ok ? '' : outcome.report;
        expect(report,).toContain('POLICY NOT IN FORCE',);
        expect(report,).toContain(POLICY_PROCESS_WATCH,);
        expect(report,).toContain(POLICY_SOCKET_MARKING,);
        expect(report,).toContain('/usr/bin/wg-quicker-exempt',);
        expect(report,).toContain('the caller\'s own PATH is never searched',);
        expect(report,).toContain('mise run //package/wg/cli-quicker-exempt:build',);
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
