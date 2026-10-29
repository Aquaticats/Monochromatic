/**
 Tests for the contest ledger's file names.

 The ordinal once restarted at zero per process, so a relaunch into the
 same runs directory overwrote the earlier launch's contests one by one. The
 cases hold the name to the launch it was written by and to text order.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isLedgerFileName,
  LAUNCH_STAMP,
  ledgerFileName,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: ledgerFileName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the same ordinal differently under two launches, so a relaunch appends rather than '
            + 'overwrites',
          fn: async () => {
            expect(ledgerFileName({
              ordinal: 0,
              launch: '2026-08-26T03-00-00-000Z-100',
            },),).not.toBe(ledgerFileName({
              ordinal: 0,
              launch: '2026-08-26T04-00-00-000Z-200',
            },),);
          },
        },),
        it({
          name: 'SORTS as text by launch and then by ordinal, which is contest order across relaunches',
          fn: async () => {
            /**
             Names written by an earlier and a later launch, shuffled.
             */
            const names = [
              ledgerFileName({
                ordinal: 1,
                launch: '2026-08-26T04-00-00-000Z-200',
              },),
              ledgerFileName({
                ordinal: 10,
                launch: '2026-08-26T03-00-00-000Z-100',
              },),
              ledgerFileName({
                ordinal: 0,
                launch: '2026-08-26T04-00-00-000Z-200',
              },),
              ledgerFileName({
                ordinal: 2,
                launch: '2026-08-26T03-00-00-000Z-100',
              },),
            ];
            expect(names.toSorted(),).toEqual([
              '2026-08-26T03-00-00-000Z-100-000002.json',
              '2026-08-26T03-00-00-000Z-100-000010.json',
              '2026-08-26T04-00-00-000Z-200-000000.json',
              '2026-08-26T04-00-00-000Z-200-000001.json',
            ],);
          },
        },),
        it({
          name: 'STAMPS this launch with a file-safe time and its process id',
          fn: async () => {
            expect(LAUNCH_STAMP.endsWith(`-${String(process.pid,)}`,),).toBe(true,);
            expect(LAUNCH_STAMP.includes(':',),).toBe(false,);
            expect(LAUNCH_STAMP.includes('.',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isLedgerFileName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a name the recorder writes, the control the refusal departs from',
          fn: async () => {
            expect(isLedgerFileName({
              name: ledgerFileName({
                ordinal: 0,
                launch: LAUNCH_STAMP,
              },),
            },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES the atomic write\'s temporary name for that contest, which is no contest until it is '
            + 'renamed (ledger B65)',
          fn: async () => {
            expect(isLedgerFileName({
              name: `${
            ledgerFileName({
              ordinal: 0,
              launch: LAUNCH_STAMP,
            },)
          }.4242.partial`,
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
