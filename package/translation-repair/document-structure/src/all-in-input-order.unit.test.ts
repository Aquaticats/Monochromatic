/**
 Tests for waiting on side-by-side members and reporting by input order.

 WHAT THESE PIN: where every member succeeds the values come back in input
 order whatever order they finished in; where members fail, the failure
 thrown is the very value the first failing member in input order rejected
 with, in every order the members can complete in; a failure at one position
 waits for the members listed before it and for nothing listed after it; an
 abort a member rejected with is reported like any other failure; and an
 empty input resolves to nothing.

 Every member waits at a gate the test opens, so completion order is the
 test's to choose rather than the scheduler's.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { allInInputOrder, } from '../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

//region Fixtures
// Gated members: each member is a promise the test settles when it chooses.

/**
 Builds one gate per member.

 @param count - how many members to gate

 @returns One gate per member, in input order

 @example
 ```ts
 const gates = gatesFor({ count: 3, },);
 ```
 */
function gatesFor({ count, }: { readonly count: number; },): readonly PromiseWithResolvers<string>[] {
  return Array.from(
    { length: count, },
    function openGate(): PromiseWithResolvers<string> {
      return Promise.withResolvers<string>();
    },
  );
}

/**
 The promises of a run of gates, as the helper takes them.

 @param gates - gates to read the members off

 @returns Each gate's promise, in input order

 @example
 ```ts
 const members = membersOf({ gates, },);
 ```
 */
function membersOf(
  { gates, }: { readonly gates: readonly PromiseWithResolvers<string>[]; },
): readonly Promise<string>[] {
  return gates.map(function promiseOf({ promise, },): Promise<string> {
    return promise;
  },);
}

/**
 One gate of a run.

 @param gates - run's gates

 @param position - gate to take

 @returns Gate at that position

 @example
 ```ts
 gateAt({ gates, position: 1, },).reject(new Error('second',),);
 ```
 */
function gateAt(
  {
    gates,
    position,
  }: {
    readonly gates: readonly PromiseWithResolvers<string>[];
    readonly position: number;
  },
): PromiseWithResolvers<string> {
  return nonNullishOrThrow(gates[position],);
}

/**
 Lets every settled continuation run, so a report that could be made has
 been made by the time the test looks.

 @example
 ```ts
 await settle();
 ```
 */
async function settle(): Promise<void> {
  await wait(0,);
}

/**
 Every order three members can complete in.
 */
const COMPLETION_ORDERS: readonly (readonly [number, number, number])[] = [
  [
    0,
    1,
    2,
  ],
  [
    0,
    2,
    1,
  ],
  [
    1,
    0,
    2,
  ],
  [
    1,
    2,
    0,
  ],
  [
    2,
    0,
    1,
  ],
  [
    2,
    1,
    0,
  ],
];

//endregion Fixtures

await describe({
  name: allInInputOrder.name,
  children: [
    it({
      name: 'RETURNS every value at its own position when the members finish in the reverse order',
      fn: async () => {
        const gates = gatesFor({ count: 3, },);
        const all = allInInputOrder({ members: membersOf({ gates, },), },);
        gateAt({
          gates,
          position: 2,
        },).resolve('third',);
        gateAt({
          gates,
          position: 1,
        },).resolve('second',);
        gateAt({
          gates,
          position: 0,
        },).resolve('first',);
        expect(await all,).toEqual([
          'first',
          'second',
          'third',
        ],);
      },
    },),
    it({
      name: 'KEEPS each member\'s own type at its position, so a pair destructures as the two it holds',
      fn: async () => {
        const [
          count,
          label,
        ] = await allInInputOrder({
          members: [
            Promise.resolve(3,),
            Promise.resolve('cats',),
          ],
        },);
        expect(`${String(count + 1,)} ${label.toUpperCase()}`,).toBe('4 CATS',);
      },
    },),
    it({
      name: 'RESOLVES to nothing for an empty input',
      fn: async () => {
        expect(await allInInputOrder({ members: [], },),).toEqual([],);
      },
    },),
    it({
      name: 'THROWS the very value the one failing member rejected with, after the others fulfilled',
      fn: async () => {
        const gates = gatesFor({ count: 3, },);
        const refusal = new RangeError('second member refused',);
        const reported = rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
        gateAt({
          gates,
          position: 0,
        },).resolve('first',);
        gateAt({
          gates,
          position: 1,
        },).reject(refusal,);
        gateAt({
          gates,
          position: 2,
        },).resolve('third',);
        expect(await reported,).toBe(refusal,);
      },
    },),
    ...COMPLETION_ORDERS.map(function caseForOrder(order,) {
      return it({
        name: `THROWS the first member's failure when all three members reject, finishing in the order ${
          order.join(', ',)
        }`,
        fn: async () => {
          const gates = gatesFor({ count: 3, },);
          const refusals = [
            new Error('member 0 refused',),
            new Error('member 1 refused',),
            new Error('member 2 refused',),
          ];
          const reported = rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
          for (const position of order) {
            gateAt({
              gates,
              position,
            },).reject(nonNullishOrThrow(refusals[position],),);
          }
          expect(await reported,).toBe(nonNullishOrThrow(refusals[0],),);
        },
      },);
    },),
    it({
      name: 'THROWS the first failure in input order when the earlier member fulfilled and two later ones reject, '
        + 'the last of them first',
      fn: async () => {
        const gates = gatesFor({ count: 3, },);
        const middle = new Error('member 1 refused',);
        const last = new Error('member 2 refused',);
        const reported = rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
        gateAt({
          gates,
          position: 2,
        },).reject(last,);
        gateAt({
          gates,
          position: 0,
        },).resolve('first',);
        gateAt({
          gates,
          position: 1,
        },).reject(middle,);
        expect(await reported,).toBe(middle,);
      },
    },),
    it({
      name: 'HOLDS a later failure until the earlier member settles, and reports the earlier one when it fails too',
      fn: async () => {
        const gates = gatesFor({ count: 2, },);
        const early = new Error('member 0 refused',);
        const late = new Error('member 1 refused',);
        /**
         Whether the call has reported yet.
         */
        const state = { reported: false, };
        const reported = (async function noteReport(): Promise<unknown> {
          /**
           What the call reported.
           */
          const failure = await rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
          state.reported = true;
          return failure;
        })();
        gateAt({
          gates,
          position: 1,
        },).reject(late,);
        await settle();
        expect(state.reported,).toBe(false,);
        gateAt({
          gates,
          position: 0,
        },).reject(early,);
        expect(await reported,).toBe(early,);
      },
    },),
    it({
      name: 'HOLDS a later failure until the earlier member settles, and reports the later one when the earlier '
        + 'fulfils',
      fn: async () => {
        const gates = gatesFor({ count: 2, },);
        const late = new Error('member 1 refused',);
        const reported = rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
        gateAt({
          gates,
          position: 1,
        },).reject(late,);
        await settle();
        gateAt({
          gates,
          position: 0,
        },).resolve('first',);
        expect(await reported,).toBe(late,);
      },
    },),
    it({
      name: 'THROWS the earlier failure without waiting for a later member that never settles',
      fn: async () => {
        const gates = gatesFor({ count: 3, },);
        const refusal = new Error('member 0 refused',);
        const reported = rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
        gateAt({
          gates,
          position: 0,
        },).reject(refusal,);
        expect(await reported,).toBe(refusal,);
      },
    },),
    it({
      name: 'THROWS a middle failure without waiting for a later member that never settles',
      fn: async () => {
        const gates = gatesFor({ count: 3, },);
        const refusal = new Error('member 1 refused',);
        const reported = rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
        gateAt({
          gates,
          position: 0,
        },).resolve('first',);
        gateAt({
          gates,
          position: 1,
        },).reject(refusal,);
        expect(await reported,).toBe(refusal,);
      },
    },),
    it({
      name: 'REPORTS an abort a member rejected with as that member\'s failure, unchanged',
      fn: async () => {
        const gates = gatesFor({ count: 2, },);
        const stop = new AbortController();
        stop.abort();
        const reported = rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
        gateAt({
          gates,
          position: 0,
        },).reject(stop.signal.reason,);
        gateAt({
          gates,
          position: 1,
        },).reject(new Error('member 1 refused',),);
        expect(await reported,).toBe(stop.signal.reason,);
      },
    },),
    it({
      name: 'REPORTS an earlier member\'s plain failure ahead of a later member\'s abort, since an abort takes '
        + 'no precedence here',
      fn: async () => {
        const gates = gatesFor({ count: 2, },);
        const stop = new AbortController();
        stop.abort();
        const refusal = new Error('member 0 refused',);
        const reported = rejectionOf({ promise: allInInputOrder({ members: membersOf({ gates, },), },), },);
        gateAt({
          gates,
          position: 1,
        },).reject(stop.signal.reason,);
        gateAt({
          gates,
          position: 0,
        },).reject(refusal,);
        expect(await reported,).toBe(refusal,);
      },
    },),
  ],
},);
