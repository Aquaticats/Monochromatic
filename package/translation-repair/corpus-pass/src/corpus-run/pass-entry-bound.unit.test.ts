/**
 Tests the bound an entry hands its seat reader: the smaller of what the pass has left and what the entry's own hard cap
 has left, on a clock the case moves by hand.

 Fixtures are cat-themed invention; the client is a stub that carries only a bound and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RunClient, } from '@monochromatic-dev/translation-repair-run';
import { withEntryBound, } from '../../dist/final/node/index.mjs';

/**
 Hours of an entry's cap, in milliseconds, so the cases read as the pass sees them.
 */
const HOUR_MS = 3_600_000;

/**
 A client that carries only the bound a case gives it.

 @param passLeftMs - what the pass has left, absent for a pass that bounds nothing

 @returns Client standing in for the pass's

 @example
 ```ts
 const client = clientWithBound({ passLeftMs: 5 * HOUR_MS, },);
 ```
 */
function clientWithBound({ passLeftMs, }: { readonly passLeftMs?: number; },): RunClient {
  return ((passLeftMs === undefined) ? {} : { passLeftMs: () => passLeftMs, }) as unknown as RunClient;
}

await describe({
  name: withEntryBound.name,
  concurrency: 1,
  children: [
    it({
      name: 'TAKES THE ENTRY\'S CAP LEFT WHEN IT IS THE SMALLER: a pass with 60 h left and an entry 400 min into a '
        + '420 min cap has 20 min, and the bound falls as the entry\'s clock moves',
      fn: async () => {
        const clock = { now: 1_000, };
        const entry = withEntryBound({
          client: clientWithBound({ passLeftMs: 60 * HOUR_MS, },),
          hardCapMs: 7 * HOUR_MS,
          startedAt: clock.now - (400 * 60_000),
          now: () => clock.now,
        },);
        expect(entry.passLeftMs?.(),).toBe(20 * 60_000,);
        expect(entry.passBoundOwner?.(),).toBe('entry',);
        clock.now += 5 * 60_000;
        expect(entry.passLeftMs?.(),).toBe(15 * 60_000,);
      },
    },),

    it({
      name: 'TAKES THE PASS\'S BOUND WHEN IT IS THE SMALLER: an entry just begun under a pass with 10 min left',
      fn: async () => {
        const entry = withEntryBound({
          client: clientWithBound({ passLeftMs: 10 * 60_000, },),
          hardCapMs: 7 * HOUR_MS,
          startedAt: 5_000,
          now: () => 5_000,
        },);
        expect(entry.passLeftMs?.(),).toBe(10 * 60_000,);
        expect(entry.passBoundOwner?.(),).toBe('pass',);
      },
    },),

    it({
      name: 'BOUNDS BY THE ENTRY\'S CAP ALONE for a client whose pass bounds nothing, and runs to zero and below once '
        + 'the cap is spent, so no return is waited for',
      fn: async () => {
        const clock = { now: 0, };
        const entry = withEntryBound({
          client: clientWithBound({},),
          hardCapMs: 1_000,
          startedAt: 0,
          now: () => clock.now,
        },);
        expect(entry.passLeftMs?.(),).toBe(1_000,);
        expect(entry.passBoundOwner?.(),).toBe('entry',);
        clock.now = 1_000;
        expect(entry.passLeftMs?.(),).toBe(0,);
        clock.now = 1_500;
        expect(entry.passLeftMs?.(),).toBe(-500,);
      },
    },),
  ],
},);
