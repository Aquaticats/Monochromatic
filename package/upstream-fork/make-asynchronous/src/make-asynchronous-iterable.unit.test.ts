/**
 Tests for `makeAsynchronousIterable`: iteration draining, iterator
 normalization, failure propagation, abort handling, and the `withSignal`
 member shape.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type AsyncIterableWrapped,
  type IterableCall,
  type IterableValue,
  makeAsynchronousIterable,
} from '../dist/final/neutral/index.mjs';

/**
 Drains one async iterable into an array.
 
 The iterable is typed `unknown` at the drain boundary because worker
 results cross the structured-clone boundary; each test narrows its own
 values after collecting.
 
 @param iterable - Iterable to drain.
 
 @returns Values in yield order.
 */
async function collect(iterable: AsyncIterable<unknown>,): Promise<unknown[]> {
  /**
   Values in yield order.
   */
  const values: unknown[] = [];
  for await (const value of iterable)
    values.push(value,);
  return collectDone(values,);
}

/**
 Hands drained values back through one more tick so the worker's cleanup
 disposer runs before the caller observes them.
 
 @param values - Drained values.
 
 @returns Same values.
 */
async function collectDone(values: unknown[],): Promise<unknown[]> {
  return values;
}

await describe({
  name: makeAsynchronousIterable.name,
  children: [
    //region Iteration

    it({
      name: 'drains a generator function in yield order',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (count: number,): Generator<number> {
            for (let index = 0; index < count; index += 1)
              yield index;
          },
        },);
        expect(
          await collect(fn({ args: [3], }),),
        ).toEqual([
          0,
          1,
          2,
        ],);
      },
    },),

    it({
      name: 'drains an iterator object returned without a generator',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          fn: function iterate(fixture: readonly number[],): Iterable<number> {
            return fixture;
          },
        },);
        expect(
          await collect(fn({ args: [[1, 2]], }),),
        ).toEqual([
          1,
          2,
        ],);
      },
    },),

    it({
      name: 'drains a non-iterator iterable like an array',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          fn: function identity(fixture: readonly number[],): readonly number[] {
            return fixture;
          },
        },);
        expect(
          await collect(fn({ args: [[1, 2, 3]], }),),
        ).toEqual([
          1,
          2,
          3,
        ],);
      },
    },),

    it({
      name: 'drains an async iterable returned without an iterator',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: async function * (count: number,): AsyncGenerator<number> {
            for (let index = 0; index < count; index += 1)
              yield index;
          },
        },);
        expect(
          await collect(fn({ args: [2], }),),
        ).toEqual([
          0,
          1,
        ],);
      },
    },),

    it({
      name: 'resolves an empty iterable to no values',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (): Generator<never> {
            // Nothing to yield.
          },
        },);
        expect(
          await collect(fn({ args: [], }),),
        ).toEqual([],);
      },
    },),

    it({
      name: 'yields falsy values instead of ending iteration',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line no-restricted-syntax/no-nullish-union, eslint/object-shorthand -- falsy worker values are the behavior under test (reply protocol keys on `error`, not the value); serialized generators must keep `function *` expression syntax
          fn: function * (fixture: readonly (number | null)[],): Generator<number | null> {
            yield * fixture;
          },
        },);
        expect(
          await collect(fn({
            args: [[0, null, 1]],
          },),),
        ).toEqual([
          0,
          null,
          1,
        ],);
      },
    },),

    it({
      name: 'keeps concurrent iterations isolated in their own workers',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (count: number,): Generator<number> {
            for (let index = 0; index < count; index += 1)
              yield index;
          },
        },);
        expect(await Promise.all([
          collect(fn({ args: [2], }),),
          collect(fn({ args: [3], }),),
        ],),).toEqual([
          [
            0,
            1,
          ],
          [
            0,
            1,
            2,
          ],
        ],);
      },
      timeout: 30_000,
    },),

    //endregion Iteration

    //region Failures

    it({
      name: 'propagates a generator failure after earlier values',
      fn: async () => {
        /**
         Failure the generator throws after two values.
         */
        const failure = new Error('Catch me if you can!',);
        /**
         Values that made it through before the failure.
         */
        const values: unknown[] = [];
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (fixture: readonly number[],): Generator<number> {
            for (const value of fixture)
              yield value;
            throw new Error('Catch me if you can!',);
          },
        },);
        let caught: unknown;
        try {
          for await (const value of fn({ args: [[1, 2]], }))
            values.push(value,);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe(failure.message,);
        expect(values,).toEqual([
          1,
          2,
        ],);
      },
    },),

    it({
      name: 'rejects when the function throws before returning',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          fn: function fail(): Generator<never, never, unknown> {
            throw new TypeError('unicorn',);
          },
        },);
        let caught: unknown;
        try {
          for await (const ignored of fn({ args: [], }))
            void ignored;
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
      },
    },),

    it({
      name: 'rejects when the function returns a non-iterable',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          fn: function notIterable(): Generator<number, void, unknown> {
            return undefined as unknown as Generator<number, void, unknown>;
          },
        },);
        let caught: unknown;
        try {
          for await (const ignored of fn({ args: [], }))
            void ignored;
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
      },
      timeout: 30_000,
    },),

    it({
      name: 'rejects when the iterator result is not an object',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          fn: function badIterator(): Generator<number, void, unknown> {
            return { next: function next(): number {
              return 5;
            }, } as unknown as Generator<number, void, unknown>;
          },
        },);
        let caught: unknown;
        try {
          for await (const ignored of fn({ args: [], }))
            void ignored;
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('Iterator result is not an object',);
      },
      timeout: 30_000,
    },),

    it({
      name: 'preserves iterable error properties',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (): Generator<number> {
            yield 1;
            /**
             Failure carrying an extra code property.
             */
            const error = new Error('unicorn',) as Error & { code?: string; };
            error.code = 'E_ITERABLE';
            throw error;
          },
        },);
        /**
         Values drained before the error.
         */
        const values: unknown[] = [];
        let caught: unknown;
        try {
          for await (const value of fn({ args: [], }))
            values.push(value,);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as { readonly code?: string; }).code,).toBe('E_ITERABLE',);
        expect(values,).toEqual([
          1,
        ],);
      },
    },),

    //endregion Failures

    //region Abort

    it({
      name: 'rejects iteration with the reason of a pre-aborted signal',
      fn: async () => {
        /**
         Abort reason the iteration must reject with.
         */
        const failure = new Error('Aborted',);
        /**
         Controller aborted before iteration starts.
         */
        const controller = new AbortController();
        controller.abort(failure,);
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (): Generator<number> {
            for (let index = 0; index < 1_000_000; index += 1)
              yield 1;
          },
        },);
        let caught: unknown;
        try {
          for await (const ignored of fn.withSignal(controller.signal,)({ args: [], }))
            void ignored;
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);
      },
    },),

    it({
      name: 'keeps values yielded before an abort',
      fn: async () => {
        /**
         Abort reason delivered after the first value.
         */
        const failure = new Error('Aborted',);
        /**
         Controller aborted once the consumer sees the first value.
         */
        const controller = new AbortController();
        /**
         Values that made it through before the abortion.
         */
        const values: unknown[] = [];
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (): Generator<number> {
            yield 1;
            yield 2;
            yield 3;
          },
        },);
        let caught: unknown;
        try {
          for await (const value of fn.withSignal(controller.signal,)({ args: [], })) {
            values.push(value,);
            controller.abort(failure,);
          }
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);
        expect(values,).toEqual([
          1,
        ],);
      },
    },),

    //endregion Abort

    //region Members

    it({
      name: 'attaches withSignal as an enumerable, writable, configurable property like upstream',
      fn: async () => {
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (count: number,): Generator<number> {
            yield count;
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
      name: 'types the wrapper as AsyncIterableWrapped and calls as IterableCall',
      fn: async () => {
        /**
         Wrapper whose public type must stay stable for consumers.
         */
        const fn = makeAsynchronousIterable({
          // oxlint-disable-next-line eslint/object-shorthand -- serialized generator functions must keep `function *` expression syntax; method shorthand serializes as `*fn` which is not a valid worker expression
          fn: function * (count: number,): Generator<number> {
            yield count;
          },
        },);
        expectTypeOf(fn,).toEqualTypeOf<AsyncIterableWrapped<(count: number,) => Generator<number>>>();
        /**
         Call request whose args tuple must track the function parameters.
         */
        const call: IterableCall<(count: number,) => Generator<number>> = {
          args: [2],
        };
        expectTypeOf(call.args,).toEqualTypeOf<[count: number]>();
        expectTypeOf(fn({ args: [2], }),).toEqualTypeOf<AsyncIterable<IterableValue<(count: number,) => Generator<number>>>>();
      },
    },),

    //endregion Types
  ],
},);
