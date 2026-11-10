/**
 Tests that the one-shot TCP server rejects an unexpected listening address
 instead of leaving its caller waiting forever
 ({@link createOneShotTcpServer}).

 The bound address is read from `server.address()`, which node types as
 `AddressInfo | string | null`: a server bound to a Unix socket path reports a
 string, and a server that is no longer listening reports null. Neither can be
 handed to `--morph-compact-tcp`, so the call must fail rather than stay pending.

 @module
 */

import { Server, } from 'node:net';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { createOneShotTcpServer, } from './ipc-socket-tcp.ts';

/**
 Suite timeout, in milliseconds: long enough for the listening callback and the
 address check to run, short enough that a pending promise fails the run.
 */
const ADDRESS_GUARD_TIMEOUT_MS = 15_000;

/**
 Payload served by the guarded server; never reaches a client.
 */
const PROBE_TEXT = 'unexpected-address probe';

await describe({
  name: '',
  children: [
    describe({
      name: createOneShotTcpServer.name,
      children: [
        it({
          name: 'rejects when the listening server reports a Unix socket path',
          timeout: ADDRESS_GUARD_TIMEOUT_MS,
          fn: async ({ sinon, },) => {
            /**
             Value node reports for a server bound to a Unix socket path; the
             TCP tier cannot use it, and the guard has to reject with it.
             */
            const socketPath = '/run/morph-compact-unexpected-address.sock';
            sinon
              .stub(
              Server.prototype,
              'address',
              )
              .returns(socketPath,);

            await expect(createOneShotTcpServer(PROBE_TEXT,),)
              .rejects
              .toThrow(socketPath,);
          },
        },),
        it({
          name: 'rejects when the listening server reports no address',
          timeout: ADDRESS_GUARD_TIMEOUT_MS,
          fn: async ({ sinon, },) => {
            sinon
              .stub(
              Server.prototype,
              'address',
              )
              .returns(null,);

            await expect(createOneShotTcpServer(PROBE_TEXT,),)
              .rejects
              .toThrow('no address',);
          },
        },),
      ],
    },),
  ],
},);
