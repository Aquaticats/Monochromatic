/**
 Tests for reading which process a process id names (ledger A16).

 Linux only: the reads come from `/proc`, and the cases assert this host
 answers before reading anything else, so a host without it fails loudly
 rather than passing on nothing read.

 @module
 */

import fsPromises from 'node:fs/promises';
import { syncBuiltinESMExports, } from 'node:module';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  hostIdentity,
  startTicksOf,
  startTicksOfStat,
} from '../../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';

/**
 Process id no process can hold: above the kernel's largest (`pid_max` is at
 most 4194304), so nothing is ever read under it.
 */
const GONE_PID = 2_147_483_646;

/**
 Fields of a `/proc/<pid>/stat` line after the command name, up to the one
 before the start time.
 */
const FIELDS_BEFORE_START = 'S 0 1 1 0 -1 4194560 1 2 3 4 5 6 7 8 20 0 1 0';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: startTicksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the same start time twice for one living process, so a lock naming it matches',
          fn: async () => {
            /**
             Two reads of this process.
             */
            const [
              first,
              second,
            ] = await Promise.all([
              startTicksOf({ pid: process.pid, },),
              startTicksOf({ pid: process.pid, },),
            ],);
            expect(first.kind,).toBe('read',);
            expect(second,).toStrictEqual(first,);
          },
        },),
        it({
          name: 'READS another start time for another process, the control that a mismatch can show',
          fn: async () => {
            /**
             This process and the first process on the system.
             */
            const [
              mine,
              init,
            ] = await Promise.all([
              startTicksOf({ pid: process.pid, },),
              startTicksOf({ pid: 1, },),
            ],);
            expect(init.kind,).toBe('read',);
            expect(init,).not.toStrictEqual(mine,);
          },
        },),
        it({
          name: 'READS NOTHING for a process id no process holds',
          fn: async () => {
            expect(await startTicksOf({ pid: GONE_PID, },),).toStrictEqual({ kind: 'unread', },);
          },
        },),
      ],
    },),

    describe({
      name: startTicksOfStat.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the start time after the last closing parenthesis, where the command name holds one',
          fn: async () => {
            expect(startTicksOfStat({ stat: `7 (tabby) cat) ${FIELDS_BEFORE_START} 275883698 4096`, },),).toStrictEqual({
              kind: 'read',
              startTicks: '275883698',
            },);
          },
        },),
        it({
          name: 'READS NOTHING from a line with fewer fields than the start time stands at',
          fn: async () => {
            expect(startTicksOfStat({ stat: '7 (tabby) S 0 1', },),).toStrictEqual({ kind: 'unread', },);
          },
        },),
        it({
          name: 'READS NOTHING from a line whose start time field is empty',
          fn: async () => {
            expect(startTicksOfStat({ stat: `7 (tabby) ${FIELDS_BEFORE_START}  275883698`, },),).toStrictEqual({
              kind: 'unread',
            },);
          },
        },),
      ],
    },),

    describe({
      name: hostIdentity.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS this machine, its boot and its process-id namespace',
          fn: async () => {
            /**
             This host's identity.
             */
            const read = await hostIdentity();
            expect(read.kind,).toBe('read',);
            if (read.kind === 'read') {
              expect(read.here.bootId.length,).toBeGreaterThan(0,);
              expect(read.here.pidNamespace.startsWith('pid:[',),).toBe(true,);
              expect(read.here.host.length,).toBeGreaterThan(0,);
            }
          },
        },),
        it({
          name: 'READS NOTHING where the system answers no `/proc` file, saying so, as on a machine without one',
          fn: async (ctx,) => {
            /**
             The real `readFile`, which every other path still reaches.
             */
            const realReadFile = fsPromises.readFile;
            /**
             `readFile` as it answers on a system with no `/proc`, for the one
             file the identity is read from.
             */
            const withoutProc = ctx.sinon.stub(
              fsPromises,
              'readFile',
            ).callsFake(async function readFileWithoutProc(path, options,) {
              if (path === '/proc/sys/kernel/random/boot_id') {
                throw Object.assign(
                  new Error('no such file',),
                  { code: 'ENOENT', },
                );
              }
              return await realReadFile(
                path,
                options,
              );
            },);
            syncBuiltinESMExports();
            // THE ESM BINDING OF `readFile` FOLLOWS THE STUB ONLY WHILE IT IS SYNCED,
            // so putting it back is a step of its own that the case's end runs.
            using putBack = {
              [Symbol.dispose]: function restoreReadFile(): void {
                withoutProc.restore();
                syncBuiltinESMExports();
              },
            };
            const {
              result,
              warned,
            } = await warnLinesDuring({ run: hostIdentity, },);
            expect({
              read: result,
              warned,
            },).toEqual({
              read: { kind: 'unread', },
              warned: [
                '[process-identity] no process identity on this system (refused by Error); a runs lock is '
                  + 'judged by process id alone',
              ],
            },);
          },
        },),
      ],
    },),
  ],
},);
