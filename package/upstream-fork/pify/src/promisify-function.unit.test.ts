/**
 Tests for the promisified wrapper's call semantics: results, failures,
 argument forwarding, callback shapes, `this` unwrapping, and the promise
 module, exercised through the public `pify` entry point.
 
 @module
 */

/* oxlint-disable promise/prefer-await-to-callbacks -- External-boundary mirror: this package wraps node-style callback functions, so its fixtures and wrapped-function declarations implement the callback pattern the tests exercise; see DECISION.callback-capture.md. */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pify, } from '../dist/final/neutral/index.mjs';

/**
 Type alias shortening error-first callback signatures in fixtures.
 */
type Callback = (error: unknown, value: unknown) => void;

await describe({
  name: 'promisify-function',
  children: [
    //region Results and failures

    it({
      name: 'resolves with the callback value',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: Callback): void {
            callback(null, 'unicorn',);
          },
        },);
        expect(await promisified({
          args: [],
        },),).toBe('unicorn',);
      },
    },),

    it({
      name: 'rejects with the callback error',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: Callback): void {
            callback('boom', undefined,);
          },
        },);
        /**
         Rejection reason captured for comparison, whatever its type.
         */
        let rejected: unknown = 'unset';
        try {
          await promisified({
            args: [],
          },);
        }
        catch (error) {
          rejected = error;
        }
        expect(rejected,).toBe('boom',);
      },
    },),

    it({
      name: 'turns a synchronous throw into a rejection',
      fn: async () => {
        const promisified = pify({
          input: function fixture(): never {
            throw new Error('sync-boom',);
          },
        },);
        /**
         Rejection reason captured for comparison.
         */
        let rejected: unknown = 'unset';
        try {
          await promisified({
            args: [],
          },);
        }
        catch (error) {
          rejected = error;
        }
        expect(rejected,).toBeInstanceOf(Error,);
        expect((rejected as Error).message,).toBe('sync-boom',);
      },
    },),

    it({
      name: 'spreads the args tuple into the wrapped function before the callback',
      fn: async () => {
        const promisified = pify({
          input: function fixture(first: string, second: number, callback: Callback): void {
            callback(null, `${first}-${second}`,);
          },
        },);
        expect(await promisified({
          args: [
            'a',
            7,
          ],
        },),).toBe('a-7',);
      },
    },),

    it({
      name: 'never mutates the caller-supplied args tuple',
      fn: async () => {
        /**
         Caller-side argument tuple, checked for identity after the call.
         */
        const args: [string] = ['kept',];
        const promisified = pify({
          input: function fixture(first: string, callback: Callback): void {
            callback(null, first,);
          },
        },);
        expect(await promisified({ args, },),).toBe('kept',);
        expect(args,).toEqual(['kept',],);
      },
    },),

    it({
      name: 'keeps the callback usable when a wrapped function forwards it to another member',
      fn: async () => {
        /**
         Module whose method forwards its received callback to a sibling
         member, upstream pify's internal-callback regression case.
         */
        const module = {
          foo(callback: Callback): void {
            this.bar(4, callback,);
          },
          bar(_value: number, callback: Callback): void {
            callback(null, 42,);
          },
        };
        const pified = pify({
          input: module,
        },);
        expect(await pified.foo({
          args: [],
        },),).toBe(42,);
      },
    },),

    //endregion Results and failures

    //region Callback shapes

    it({
      name: 'multiArgs resolves with every callback argument after the error',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: (error: unknown, ...results: unknown[]) => void): void {
            callback(null, 'unicorn', 'rainbow',);
          },
          options: {
            multiArgs: true,
          },
        },);
        expect(await promisified({
          args: [],
        },),).toEqual([
          'unicorn',
          'rainbow',
        ],);
      },
    },),

    it({
      name: 'multiArgs rejects with the whole callback argument list, error included',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: (error: unknown, ...results: unknown[]) => void): void {
            callback('error', 'unicorn', 'rainbow',);
          },
          options: {
            multiArgs: true,
          },
        },);
        /**
         Rejection reason captured for comparison.
         */
        let rejected: unknown = 'unset';
        try {
          await promisified({
            args: [],
          },);
        }
        catch (error) {
          rejected = error;
        }
        expect(rejected,).toEqual([
          'error',
          'unicorn',
          'rainbow',
        ],);
      },
    },),

    it({
      name: 'multiArgs without errorFirst resolves with the whole callback argument list',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: (error: unknown, ...results: unknown[]) => void): void {
            callback('unicorn', 'rainbow',);
          },
          options: {
            multiArgs: true,
            errorFirst: false,
          },
        },);
        expect(await promisified({
          args: [],
        },),).toEqual([
          'unicorn',
          'rainbow',
        ],);
      },
    },),

    it({
      name: 'without errorFirst the callback value resolves directly',
      fn: async () => {
        const promisified = pify({
          input: function fixture(value: string, callback: (result: unknown) => void): void {
            callback(value,);
          },
          options: {
            errorFirst: false,
          },
        },);
        expect(await promisified({
          args: ['🦄'],
        },),).toBe('🦄',);
      },
    },),

    it({
      name: 'resolves with undefined when the callback reports no result',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: (error: unknown) => void): void {
            callback(null,);
          },
        },);
        expect(await promisified({
          args: [],
        },),).toBe(undefined,);
      },
    },),

    //endregion Callback shapes

    //region Receivers

    it({
      name: 'unwraps the proxy to its target so wrapped methods keep their owner',
      fn: async () => {
        /**
         Module carrying state the method reads through `this`.
         */
        const module = {
          x: 'foo',
          read(callback: Callback): void {
            callback(null, this.x,);
          },
        };
        const pified = pify({
          input: module,
        },);
        expect(await pified.read({
          args: [],
        },),).toBe('foo',);
      },
    },),

    it({
      name: 'uses the caller receiver for detached calls',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: Callback): void {
            /**
             Receiver of the call narrowed to an object so an absent `this`
             reads as an empty receiver.
             */
            const receiver = (this ?? {}) as { readonly tag?: string; };
            callback(null, receiver.tag ?? 'no-this',);
          },
        },);
        expect(await promisified({
          args: [],
        },),).toBe('no-this',);
        expect(
          await promisified.call({
            tag: 'bound',
          }, {
            args: [],
          },),
        ).toBe('bound',);
      },
    },),

    //endregion Receivers

    //region Promise module

    it({
      name: 'builds promises through the configured promiseModule',
      fn: async () => {
        /**
         Constructions counted by the custom promise constructor.
         */
        const state = {
          constructed: 0,
        };
        /**
         Minimal promise constructor stand-in: constructable, delegates to
         the native promise, and counts its constructions.
         
         @param executor - Native promise executor forwarded to the delegate.
         
         @returns Native promise settling with the executor's outcome.
         */
        function TrackingPromise<T>(
          this: unknown,
          executor: (resolve: (value: T | PromiseLike<T>) => void, reject: (reason: unknown) => void) => void,
        ): Promise<T> {
          state.constructed += 1;
          return new Promise(executor,);
        }
        const promisified = pify({
          input: function fixture(callback: Callback): void {
            callback(null, 'pm',);
          },
          options: {
            promiseModule: TrackingPromise as unknown as PromiseConstructor,
          },
        },);
        expect(await promisified({
          args: [],
        },),).toBe('pm',);
        expect(state.constructed,).toBe(1,);
      },
    },),

    //endregion Promise module
  ],
},);
