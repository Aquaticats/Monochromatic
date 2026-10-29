/**
 Tests for judging whether a runs lock's holder still runs (ledger A16), on the
 one arm the runs-lock cases never reach: a host that cannot read its own
 identity.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { hostname, } from 'node:os';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { holderLiveness, } from '../../dist/final/node/index.mjs';

/**
 What a lock taken by this process on the named machine records.

 @param host - machine the lock was taken on

 @returns Lock's holder, whose process still runs

 @example
 ```ts
 const holder = recordedOn({ host: 'tabby', },);
 ```
 */
function recordedOn(
  { host, }: { readonly host: string; },
): {
  readonly pid: number;
  readonly startedAt: string;
  readonly token: string;
  readonly identity: {
    readonly kind: 'recorded';
    readonly host: string;
    readonly bootId: string;
    readonly pidNamespace: string;
    readonly startTicks: string;
  };
} {
  return {
    pid: process.pid,
    startedAt: '2026-08-15T00:00:00.000Z',
    token: 'tabby',
    identity: {
      kind: 'recorded',
      host,
      bootId: 'boot-of-the-cat',
      pidNamespace: 'pid:[4026531836]',
      startTicks: '275883698',
    },
  };
}

await describe({
  name: holderLiveness.name,
  children: [
    it({
      name: 'JUDGES BY THE ID ALONE a lock taken under this machine\'s name where this host reads no identity, '
        + 'and holds one that names another machine',
      fn: async () => {
        expect([
          await holderLiveness({
            holder: recordedOn({ host: hostname(), },),
            here: { kind: 'unread', },
          },),
          await holderLiveness({
            holder: recordedOn({ host: 'a-machine-in-the-next-room', },),
            here: { kind: 'unread', },
          },),
        ],).toEqual([
          {
            state: 'held',
            judgedBy: 'pid',
          },
          {
            state: 'held',
            judgedBy: 'host',
          },
        ],);
      },
    },),
  ],
},);
