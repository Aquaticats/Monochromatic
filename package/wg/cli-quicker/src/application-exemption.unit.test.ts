import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applicationWatchStartArgs,
  applicationWatchStopArgs,
  resolveApplicationUid,
  startupFailureLines,
} from '../dist/final/node/application-exemption.mjs';

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
