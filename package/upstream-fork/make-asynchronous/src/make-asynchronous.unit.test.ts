/**
 Tests for `makeAsynchronous`: single-call worker wrapping, result and
 failure restoration, abort handling, and the `withSignal` member shape.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type AsyncCall,
  type AsyncForm,
  type AsyncWrapped,
  makeAsynchronous,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: makeAsynchronous.name,
  children: [
    //region Results

    it({
      name: 'resolves with a synchronous function result',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function answer(): number {
            return 42;
          },
        },);
        expect(await fn({ args: [], }),).toBe(42,);
      },
    },),

    it({
      name: 'resolves with an async function result',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: async function delayedAnswer(): Promise<string> {
            return 'done';
          },
        },);
        expect(await fn({ args: [], }),).toBe('done',);
      },
    },),

    it({
      name: 'spreads the args tuple into the function',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function add(left: number, right: number,): number {
            return left + right;
          },
        },);
        expect(await fn({
          args: [
            2,
            3,
          ],
        },),).toBe(5,);
      },
    },),

    it({
      name: 'runs the function in a worker without blocking the main thread',
      fn: async () => {
        /**
         Whether the main thread observed an event-loop turn while the
         worker spun.
         */
        const state = { turned: false, };
        setImmediate(function markTurn(): void {
          state.turned = true;
        },);
        const fn = makeAsynchronous({
          fn: function spin(): string {
            /**
             Busy-loop deadline inside the worker.
             */
            const end = Date.now() + 300;
            while (Date.now() < end) {
              // Block the worker, not the main thread.
            }
            return 'spun';
          },
        },);
        expect(await fn({ args: [], }),).toBe('spun',);
        expect(state.turned,).toBe(true,);
      },
      timeout: 30_000,
    },),

    it({
      name: 'keeps concurrent calls isolated in their own workers',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function double(value: number,): number {
            return value * 2;
          },
        },);
        expect(await Promise.all([
          fn({ args: [0], },),
          fn({ args: [1], },),
          fn({ args: [2], },),
          fn({ args: [3], },),
          fn({ args: [4], },),
          fn({ args: [5], },),
          fn({ args: [6], },),
          fn({ args: [7], },),
          fn({ args: [8], },),
          fn({ args: [9], },),
        ],),).toEqual([
          0,
          2,
          4,
          6,
          8,
          10,
          12,
          14,
          16,
          18,
        ],);
      },
      timeout: 30_000,
    },),

    it({
      name: 'resolves rich structured-clone values',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function echo(value: { readonly nested: { readonly array: readonly number[]; }; },): {
            readonly nested: { readonly array: readonly number[]; };
          } {
            return value;
          },
        },);
        expect(await fn({
          args: [
            { nested: { array: [1, [2, [3]] as unknown as number], }, },
          ],
        },),).toEqual({ nested: { array: [1, [2, [3]]], }, },);
      },
    },),

    //endregion Results

    //region Failures

    it({
      name: 'rejects with the failure of a throwing function',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function failingCall(): never {
            throw new TypeError('unicorn',);
          },
        },);
        let caught: unknown;
        try {
          await fn({ args: [], },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
        expect((caught as TypeError).message,).toBe('unicorn',);
      },
    },),

    it({
      name: 'rejects with a non-error thrown value verbatim',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function throwString(value: string,): never {
            // oxlint-disable-next-line typescript/only-throw-error -- non-error worker rejections are the behavior under test; the reply protocol keys on `error`, not the value
            throw value;
          },
        },);
        let caught: unknown;
        try {
          await fn({ args: ['unicorn'], },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe('unicorn',);
      },
    },),

    it({
      name: 'rejects with a falsy thrown value instead of resolving',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function throwZero(value: number,): never {
            // oxlint-disable-next-line typescript/only-throw-error -- falsy worker rejections are the behavior under test; the reply protocol keys on `error`, not the value
            throw value;
          },
        },);
        let caught: unknown;
        try {
          await fn({ args: [0], },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(0,);
      },
    },),

    it({
      name: 'resolves with a falsy returned value instead of rejecting',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function identity(value: number,): number {
            return value;
          },
        },);
        expect(await fn({ args: [0], }),).toBe(0,);
      },
    },),

    it({
      name: 'rejects when the wrapped function closes over outer scope',
      fn: async () => {
        const fn = makeAsynchronous({
          // oxlint-disable-next-line unicorn/new-for-builtins, eslint/no-new-func, typescript/no-implied-eval -- serializing a closed-over binding is the behavior under test; `Function` builds the closure without scope access
          fn: Function('return outerMissingValue + 1;',) as () => number,
        },);
        let caught: unknown;
        try {
          await fn({ args: [], },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ReferenceError,);
      },
    },),

    it({
      name: 'rejects when the arguments cannot be cloned',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function identity(value: unknown,): unknown {
            return value;
          },
        },);
        let caught: unknown;
        try {
          await fn({
            args: [function unclonable(): void {},],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toContain('could not be cloned',);
      },
    },),

    //endregion Failures

    //region Abort

    it({
      name: 'rejects with the reason of a pre-aborted signal',
      fn: async () => {
        /**
         Abort reason the call must reject with.
         */
        const failure = new Error('Aborted',);
        /**
         Controller aborted before the call starts.
         */
        const controller = new AbortController();
        controller.abort(failure,);
        const fn = makeAsynchronous({
          fn: function neverSettles(): Promise<never> {
            return new Promise(function pending(): void {},);
          },
        },);
        let caught: unknown;
        try {
          await fn.withSignal(controller.signal,)({ args: [], },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);
      },
    },),

    it({
      name: 'rejects with the reason of an interrupting abort',
      fn: async () => {
        /**
         Abort reason delivered mid-flight.
         */
        const failure = new Error('Aborted',);
        /**
         Controller aborted once the worker is busy.
         */
        const controller = new AbortController();
        const fn = makeAsynchronous({
          fn: function neverSettles(): Promise<never> {
            return new Promise(function pending(): void {},);
          },
        },);
        /**
         Call rejected by the mid-flight abort.
         */
        const pending = fn.withSignal(controller.signal,)({ args: [], },);
        controller.abort(failure,);
        let caught: unknown;
        try {
          await pending;
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);
      },
    },),

    it({
      name: 'removes the abort listener once the call settles',
      fn: async () => {
        const { getEventListeners, } = await import('node:events',);
        /**
         Long-lived signal reused across calls.
         */
        const controller = new AbortController();
        const fn = makeAsynchronous({
          fn: function identity(value: number,): number {
            return value;
          },
        },);
        expect(await fn.withSignal(controller.signal,)({ args: [1], },),).toBe(1,);
        expect(getEventListeners(
          controller.signal,
          'abort',
        ).length,).toBe(0,);
      },
    },),

    it({
      name: 'rejects with a falsy abort reason verbatim',
      fn: async () => {
        /**
         Controller aborted with a falsy reason.
         */
        const controller = new AbortController();
        controller.abort(0,);
        const fn = makeAsynchronous({
          fn: function neverSettles(): Promise<never> {
            return new Promise(function pending(): void {},);
          },
        },);
        let caught: unknown;
        try {
          await fn.withSignal(controller.signal,)({ args: [], },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(0,);
      },
    },),

    //endregion Abort

    //region Members

    it({
      name: 'attaches withSignal as an enumerable, writable, configurable property like upstream',
      fn: async () => {
        const fn = makeAsynchronous({
          fn: function identity(value: string,): string {
            return value;
          },
        },);
        /**
         Own descriptor of the attached `withSignal` member.
         */
        const descriptor = Object.getOwnPropertyDescriptor(
          fn,
          'withSignal',
        );
        expect(descriptor,).toBeDefined();
        expect(descriptor?.enumerable,).toBe(true,);
        expect(descriptor?.writable,).toBe(true,);
        expect(descriptor?.configurable,).toBe(true,);
        expect(typeof descriptor?.value,).toBe('function',);
        expect(Object.keys(fn,),).toEqual([
          'withSignal',
        ],);
      },
    },),

    //endregion Members

    //region Types

    it({
      name: 'types the wrapper as AsyncWrapped and calls as AsyncCall',
      fn: async () => {
        /**
         Wrapper whose public type must stay stable for consumers.
         */
        const fn = makeAsynchronous({
          fn: function add(left: number, right: number,): number {
            return left + right;
          },
        },);
        expectTypeOf(fn,).toEqualTypeOf<AsyncWrapped<(left: number, right: number,) => number>>();
        /**
         Call request whose args tuple must track the function parameters.
         */
        const call: AsyncCall<(left: number, right: number,) => number> = {
          args: [
            1,
            2,
          ],
        };
        expectTypeOf(call.args,).toEqualTypeOf<[left: number, right: number]>();
        expectTypeOf(fn.withSignal(new AbortController().signal,),).toEqualTypeOf<AsyncForm<(left: number, right: number,) => number>>();
      },
    },),

    //endregion Types
  ],
},);
