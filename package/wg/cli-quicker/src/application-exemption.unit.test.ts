import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  applicationWatchStartArgs,
  applicationWatchStopArgs,
  resolveApplicationUid,
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
      name: 'prefers explicit override over sudo identity',
      fn: async () => {
        expect(resolveApplicationUid({
          environment: {
            WG_QUICKER_EXEMPT_UID: '2000',
            SUDO_UID: '1000',
          },
          currentUid: 0,
        },),).toBe(2_000,);
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
