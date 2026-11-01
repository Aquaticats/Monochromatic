/**
 Tests for the test-only helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createDeferred,
  createGate,
  createSpyCache,
  promiseState,
  rejectionOf,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: 'test support helpers',
  children: [
    describe({
      name: createDeferred.name,
      children: [
        it({
          name: 'keeps the promise pending until settled',
          fn: async () => {
            const deferred = createDeferred<number>();
            expect(await promiseState(deferred.promise,),).toBe('pending',);
          },
        },),

        it({
          name: 'resolves with the resolved value',
          fn: async () => {
            const deferred = createDeferred<number>();
            deferred.resolve(7,);
            expect(await deferred.promise,).toBe(7,);
          },
        },),

        it({
          name: 'rejects with the rejection reason',
          fn: async () => {
            const deferred = createDeferred<number>();
            deferred.reject(new Error('nope',),);

            let caught: unknown;
            try {
              await deferred.promise;
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toBe('nope',);
          },
        },),
      ],
    },),

    describe({
      name: createGate.name,
      children: [
        it({
          name: 'keeps the gate promise pending until released',
          fn: async () => {
            const gate = createGate();
            expect(await promiseState(gate.open,),).toBe('pending',);
          },
        },),

        it({
          name: 'settles the gate promise once released',
          fn: async () => {
            const gate = createGate();
            gate.release();
            expect(await gate.open,).toBeUndefined();
            expect(await promiseState(gate.open,),).toBe('fulfilled',);
          },
        },),
      ],
    },),

    describe({
      name: yieldTurn.name,
      children: [
        it({
          name: 'resolves to undefined after a full turn',
          fn: async () => {
            expect(await yieldTurn(),).toBeUndefined();
          },
        },),

        it({
          name: 'spans a macrotask turn, letting earlier timers fire first',
          fn: async () => {
            /**
             Order in which timer and post-turn steps ran.
             */
            const order: string[] = [];
            setTimeout(function recordTimer(): void {
              order.push('timer',);
            }, 0,);
            await yieldTurn();
            order.push('after',);
            expect(order,).toEqual([
              'timer',
              'after',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: promiseState.name,
      children: [
        it({
          name: 'reports pending for an unsettled promise',
          fn: async () => {
            const deferred = createDeferred<number>();
            expect(await promiseState(deferred.promise,),).toBe('pending',);
          },
        },),

        it({
          name: 'reports fulfilled after resolution',
          fn: async () => {
            const deferred = createDeferred<number>();
            deferred.resolve(1,);
            expect(await promiseState(deferred.promise,),).toBe('fulfilled',);
          },
        },),

        it({
          name: 'reports rejected after rejection without rethrowing',
          fn: async () => {
            const deferred = createDeferred<number>();
            deferred.reject(new Error('boom',),);
            expect(await promiseState(deferred.promise,),).toBe('rejected',);
          },
        },),
      ],
    },),

    describe({
      name: createSpyCache.name,
      children: [
        it({
          name: 'records every storage call in invocation order',
          fn: async () => {
            const spy = createSpyCache<string, number>();
            await spy.cache.set(
              'k',
              1,
            );
            expect(await spy.cache.has('k',),).toBe(true,);
            expect(await spy.cache.get('k',),).toBe(1,);
            spy.cache.delete('k',);
            await spy.cache.set(
              'k',
              2,
            );
            spy.cache.clear();
            expect(spy.calls.map(function callMethod(call,): string {
              return call.method;
            },),).toEqual([
              'set',
              'has',
              'get',
              'delete',
              'set',
              'clear',
            ],);
          },
        },),

        it({
          name: 'keeps its backing entries in sync with set and clear',
          fn: async () => {
            const spy = createSpyCache<string, number>();
            await spy.cache.set(
              'k',
              3,
            );
            expect(spy.entries.get('k',),).toBe(3,);
            spy.cache.clear();
            expect(spy.entries.size,).toBe(0,);
          },
        },),

        it({
          name: 'reports cache misses as has-false and get-undefined',
          fn: async () => {
            const spy = createSpyCache<string, number>();
            expect(await spy.cache.has('missing',),).toBe(false,);
            expect(await spy.cache.get('missing',),).toBeUndefined();
          },
        },),
      ],
    },),

    describe({
      name: rejectionOf.name,
      children: [
        it({
          name: 'hands back the rejection reason',
          fn: async () => {
            /**
             Reason the promise rejects with.
             */
            const reason = new Error('boom',);
            /**
             Rejecting promise handed to the helper.
             */
            const rejecting = Promise.reject(reason,);
            expect(await rejectionOf(rejecting,),).toBe(reason,);
          },
        },),

        it({
          name: 'throws when the promise fulfills',
          fn: async () => {
            /**
             Failure raised for a fulfilled promise.
             */
            let caught: unknown;
            try {
              await rejectionOf(Promise.resolve(1,),);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('expected the promise to reject, but it fulfilled',);
          },
        },),
      ],
    },),
  ],
},);
