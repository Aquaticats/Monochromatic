/**
 Tests for the host and helper spelling: how an LFS host becomes a credential key,
 and how the private file's path is quoted inside the helper command.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  LfsHostError,
  parseLfsHost,
  storeCredentialHelper,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Worker URL the cases parse.
 */
const WORKER_URL = 'https://monochromatic-lfs.aquaticat.workers.dev';

await describe({
  name: '',
  children: [
    describe({
      name: parseLfsHost.name,
      children: [
        it({
          name: 'splits a Worker URL into origin, protocol and host',
          fn: async () => {
            expect(parseLfsHost({ host: WORKER_URL, },),).toEqual({
              origin: WORKER_URL,
              protocol: 'https',
              host: 'monochromatic-lfs.aquaticat.workers.dev',
            },);
          },
        },),
        it({
          name: 'keeps the port in origin and host',
          fn: async () => {
            expect(parseLfsHost({ host: 'http://127.0.0.1:8791', },),).toEqual({
              origin: 'http://127.0.0.1:8791',
              protocol: 'http',
              host: '127.0.0.1:8791',
            },);
          },
        },),
        it({
          name: 'drops a path from the origin it matches on',
          fn: async () => {
            expect(parseLfsHost({ host: `${WORKER_URL}/objects`, },).origin,).toBe(WORKER_URL,);
          },
        },),
        ...([
          ['a relative path', 'monochromatic-lfs.aquaticat.workers.dev',],
          ['a scheme that is not http', 'ssh://git@example.test/repository',],
        ] as const).map(function mapRefused([description, host,]: readonly [string, string,],) {
          return it({
            name: `refuses ${description}`,
            fn: async () => {
              /**
               Error the parse raised.
               */
              let caught: unknown;
              try {
                parseLfsHost({ host, },);
              }
              catch (error) {
                caught = error;
              }
              expect(caught,).toBeInstanceOf(LfsHostError,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: storeCredentialHelper.name,
      children: [
        it({
          name: 'names the host in the key and quotes the file in the command',
          fn: async () => {
            expect(storeCredentialHelper({
              host: parseLfsHost({ host: WORKER_URL, },),
              credentialFile: '/home/someone/.config/monochromatic-lfs/credentials',
            },),).toEqual({
              key: `credential.${WORKER_URL}.helper`,
              command: `store --file '/home/someone/.config/monochromatic-lfs/credentials'`,
            },);
          },
        },),
        it({
          name: 'escapes a single quote in the file path, because Git runs the helper through a shell',
          fn: async () => {
            /**
             Helper whose file path holds one single quote.
             */
            const helper = storeCredentialHelper({
              host: parseLfsHost({ host: WORKER_URL, },),
              credentialFile: "/home/oa'neil/credentials",
            },);
            expect(helper.command,).toBe(`store --file '/home/oa'\\''neil/credentials'`,);
          },
        },),
        it({
          name: 'keeps a path with spaces as one shell word',
          fn: async () => {
            /**
             Helper whose file path holds a space.
             */
            const helper = storeCredentialHelper({
              host: parseLfsHost({ host: WORKER_URL, },),
              credentialFile: '/home/someone/my config/credentials',
            },);
            expect(helper.command,).toBe(`store --file '/home/someone/my config/credentials'`,);
          },
        },),
      ],
    },),
  ],
},);
