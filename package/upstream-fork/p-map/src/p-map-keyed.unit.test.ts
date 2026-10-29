/**
 Tests for `pMapKeyed` and `pMapSettledKeyed`, ported from `p-map` 7.1.1's
 keyed suites: key selection, null-prototype results, skips and stops,
 failures, `Map` inputs, and argument validation.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidKeyedInputError,
  MapperRequiredError,
  pMapKeyed,
  pMapSettledKeyed,
  pMapSkip,
  pMapStop,
} from '../dist/final/neutral/index.mjs';

import { rejectionOf, } from './test-support.ts';

/**
 Rejects after the stop of the keyed run that uses it has landed.

 @returns Promise that rejects after a delay.

 @throws Error after the delay.

 @example
 ```ts
 const late = rejectLater();
 ```
 */
async function rejectLater(): Promise<number> {
  await wait(25,);
  throw new Error('after the stop',);
}

await describe({
  name: pMapKeyed.name,
  children: [
    it({
      name: 'maps each value with its key into a null-prototype object',
      fn: async () => {
        const result = await pMapKeyed({
          input: {
            a: 1,
            b: Promise.resolve(2,),
          },
          mapper: function describeEntry(
            value: number,
            key: string,
          ): string {
            return `${key}${String(value,)}`;
          },
        },);
        expect(Object.getPrototypeOf(result,),).toBe(null,);
        expect(result,).toEqual({
          a: 'a1',
          b: 'b2',
        },);
      },
    },),

    it({
      name: 'uses the own enumerable string and symbol keys, in property order',
      fn: async () => {
        /**
         Symbol key of the input.
         */
        const symbol = Symbol('symbol key of the input',);
        /**
         Input with an inherited key, a hidden key, numeric keys, and a symbol.
         */
        const input = Object.create({ inherited: 'inherited', },) as Record<PropertyKey, string>;
        Reflect.set(
          input,
          'b',
          'b',
        );
        Reflect.set(
          input,
          'a',
          'a',
        );
        input[2] = 'two';
        input[1] = 'one';
        input[symbol] = 'symbol';
        Object.defineProperty(
          input,
          'hidden',
          {
            value: 'hidden',
            enumerable: false,
          },
        );
        /**
         Keys the mapper was called with, in order.
         */
        const mappedKeys: unknown[] = [];
        const result = await pMapKeyed({
          input,
          mapper: function recordKey(
            value: string,
            key: unknown,
          ): string {
            mappedKeys.push(key,);
            return value;
          },
          options: { concurrency: 1, },
        },);
        /**
         Expected keys: integer keys ascending, then strings, then symbols.
         */
        const expectedKeys = [
          '1',
          '2',
          'b',
          'a',
          symbol,
        ];
        expect(Reflect.ownKeys(result,),).toEqual(expectedKeys,);
        expect(mappedKeys,).toEqual(expectedKeys,);
      },
    },),

    it({
      name: 'checks and reads each key in turn, like Promise.allKeyed',
      fn: async () => {
        /**
         Proxy traps hit, in order.
         */
        const traps: string[] = [];
        /**
         Proxy over a two-key object recording its traps.
         */
        const input = new Proxy(
          {
            x: 1,
            y: 2,
          },
          {
            ownKeys: function trapOwnKeys(target: object,): (string | symbol)[] {
              traps.push('ownKeys',);
              return Reflect.ownKeys(target,);
            },
            getOwnPropertyDescriptor: function trapDescriptor(
              target: object,
              key: string | symbol,
            ): ReturnType<typeof Reflect.getOwnPropertyDescriptor> {
              traps.push(`getOwnPropertyDescriptor ${String(key,)}`,);
              return Reflect.getOwnPropertyDescriptor(
                target,
                key,
              );
            },
            get: function trapGet(
              target: object,
              key: string | symbol,
              receiver: unknown,
            ): unknown {
              traps.push(`get ${String(key,)}`,);
              return Reflect.get(
                target,
                key,
                receiver,
              );
            },
          },
        );
        await pMapKeyed({
          input,
          mapper: function identity(value: number,): number {
            return value;
          },
        },);
        expect(traps,).toEqual([
          'ownKeys',
          'getOwnPropertyDescriptor x',
          'get x',
          'getOwnPropertyDescriptor y',
          'get y',
        ],);
      },
    },),

    it({
      name: 'keeps a key like __proto__ as an own property of the result',
      fn: async () => {
        /**
         Input parsed from JSON so `__proto__` is an own key.
         */
        const input = JSON.parse('{"__proto__": 1, "toString": 2}',) as Record<string, number>;
        const result = await pMapKeyed({
          input,
          mapper: function tenfold(value: number,): number {
            return value * 10;
          },
        },);
        expect(Object.keys(result,),).toEqual([
          '__proto__',
          'toString',
        ],);
        expect(Reflect.get(
          result,
          '__proto__',
        ),).toBe(10,);
        expect(Reflect.get(
          result,
          'toString',
        ),).toBe(20,);
      },
    },),

    it({
      name: 'rejects on a mapper error and on a value that rejects, without calling the mapper for the value',
      fn: async () => {
        const mapperError = await rejectionOf(pMapKeyed({
          input: {
            a: 1,
            b: 2,
          },
          mapper: async function failOnB(
            value: number,
            key: string,
          ): Promise<number> {
            await wait(1,);
            if (key === 'b')
              throw new Error('b',);
            return value;
          },
        },),);
        expect((mapperError as Error).message,).toBe('b',);

        /**
         Keys the mapper was called with.
         */
        const mappedKeys: string[] = [];
        const valueError = await rejectionOf(pMapKeyed({
          input: {
            a: 1,
            b: Promise.reject<number>(new Error('b',),),
          },
          mapper: function recordKey(
            value: number,
            key: string,
          ): number {
            mappedKeys.push(key,);
            return value;
          },
        },),);
        expect((valueError as Error).message,).toBe('b',);
        expect(mappedKeys,).toEqual(['a',],);
      },
    },),

    it({
      name: 'collects the errors in key order with stopOnError false',
      fn: async () => {
        const error = await rejectionOf(pMapKeyed({
          input: {
            a: 1,
            b: 2,
          },
          mapper: async function failLater(
            value: number,
            key: string,
          ): Promise<number> {
            await wait((key === 'a')
              ? 15
              : 1,);
            throw new Error(`failed ${String(value,)}`,);
          },
          options: { stopOnError: false, },
        },),);
        expect(error,).toBeInstanceOf(AggregateError,);
        expect((error as AggregateError).errors.map(function messageOf(inner: Error,): string {
          return inner.message;
        },),).toEqual([
          'failed 1',
          'failed 2',
        ],);
      },
    },),

    it({
      name: 'leaves out a skipped key and keeps only the keys before a stop',
      fn: async () => {
        const skipped = await pMapKeyed({
          input: {
            a: 1,
            b: 2,
            c: 3,
          },
          mapper: function skipB(
            value: number,
            key: string,
          ): number | typeof pMapSkip {
            return (key === 'b')
              ? pMapSkip
              : value;
          },
        },);
        expect(skipped,).toEqual({
          a: 1,
          c: 3,
        },);

        const stopped = await pMapKeyed({
          input: {
            a: 1,
            b: 2,
            c: 3,
            d: rejectLater(),
          },
          mapper: async function stopAtC(
            value: number,
            key: string,
          ): Promise<number | typeof pMapStop> {
            await wait(1,);
            return (key === 'c')
              ? pMapStop
              : value;
          },
          options: { concurrency: 4, },
        },);
        expect(stopped,).toEqual({
          a: 1,
          b: 2,
        },);
      },
    },),

    it({
      name: 'treats a mapper that throws pMapSkip or pMapStop as a rejection',
      fn: async () => {
        /**
         Rejection of the run whose mapper throws the skip sentinel.
         */
        const skipRejection = await rejectionOf(pMapKeyed({
          input: { a: 1, },
          mapper: function throwSkip(): number {
            // oxlint-disable-next-line typescript/only-throw-error -- the case under test is a mapper that throws a sentinel
            throw pMapSkip;
          },
        },),);
        expect(skipRejection,).toBe(pMapSkip,);
      },
    },),

    it({
      name: 'rejects when the signal aborts and when reading a value throws',
      fn: async () => {
        const controller = new AbortController();
        const aborted = pMapKeyed({
          input: { a: 1, },
          mapper: async function slow(value: number,): Promise<number> {
            await wait(30,);
            return value;
          },
          options: { signal: controller.signal, },
        },);
        controller.abort(new Error('aborted',),);
        expect(((await rejectionOf(aborted,)) as Error).message,).toBe('aborted',);

        /**
         Failure the getter throws.
         */
        const failure = new Error('getter failed',);
        const readError = await rejectionOf(pMapKeyed({
          input: Object.defineProperty(
            {},
            'a',
            {
              enumerable: true,
              get: function failingGetter(): number {
                throw failure;
              },
            },
          ),
          mapper: function identity(value: unknown,): unknown {
            return value;
          },
        },),);
        expect(readError,).toBe(failure,);
      },
    },),

    it({
      name: 'validates the arguments in order: input, then mapper',
      fn: async () => {
        /**
         Rejections of the invalid runs.
         */
        const nullInput = await rejectionOf(pMapKeyed({
          input: null as never,
          mapper: undefined as never,
        },),);
        const numberInput = await rejectionOf(pMapKeyed({
          input: 5 as never,
          mapper: function identity(value: unknown,): unknown {
            return value;
          },
        },),);
        const missingMapper = await rejectionOf(pMapKeyed({
          input: { a: 1, },
          mapper: undefined as never,
        },),);
        expect(nullInput,).toBeInstanceOf(InvalidKeyedInputError,);
        expect((nullInput as Error).message,).toBe('Expected `input` to be an object, got (null)',);
        expect((numberInput as Error).message,).toBe('Expected `input` to be an object, got (number)',);
        expect(missingMapper,).toBeInstanceOf(MapperRequiredError,);
      },
    },),

    it({
      name: 'accepts any object and an array, like Promise.allKeyed',
      fn: async () => {
        expect(await pMapKeyed({
          input: [
            'x',
            'y',
          ] as never,
          mapper: function identity(value: unknown,): unknown {
            return value;
          },
        },),).toEqual({
          0: 'x',
          1: 'y',
        },);
        expect(await pMapKeyed({
          input: {},
          mapper: function identity(value: unknown,): unknown {
            return value;
          },
        },),).toEqual({},);
      },
    },),

    it({
      name: 'maps a Map into a Map with the same keys in the same order',
      fn: async () => {
        /**
         Object key of the map.
         */
        const objectKey = {};
        const input = new Map<unknown, number | Promise<number>>([
          [
            'a',
            1,
          ],
          [
            objectKey,
            Promise.resolve(2,),
          ],
          [
            3,
            3,
          ],
        ],);
        const result = await pMapKeyed({
          input,
          mapper: function tenfold(value: number,): number {
            return value * 10;
          },
        },);
        expect(result,).toBeInstanceOf(Map,);
        expect([...result.entries(),],).toEqual([
          [
            'a',
            10,
          ],
          [
            objectKey,
            20,
          ],
          [
            3,
            30,
          ],
        ],);
        expect(result,).not.toBe(input,);
      },
    },),

    it({
      name: 'reads a Map up front, so a later change does not affect the result',
      fn: async () => {
        const input = new Map<string, number>([
          [
            'a',
            1,
          ],
          [
            'b',
            2,
          ],
        ],);
        const result = await pMapKeyed({
          input,
          mapper: async function changeInput(value: number,): Promise<number> {
            input.set(
              'c',
              3,
            );
            input.delete('b',);
            await wait(1,);
            return value;
          },
          options: { concurrency: 1, },
        },);
        expect([...result.keys(),],).toEqual([
          'a',
          'b',
        ],);
      },
    },),

    it({
      name: 'keeps only the Map keys before a stop and handles an empty Map',
      fn: async () => {
        const stopped = await pMapKeyed({
          input: new Map<string, number>([
            [
              'a',
              1,
            ],
            [
              'b',
              2,
            ],
            [
              'c',
              3,
            ],
          ],),
          mapper: function stopAtB(
            value: number,
            key: string,
          ): number | typeof pMapStop {
            return (key === 'b')
              ? pMapStop
              : value;
          },
          options: { concurrency: 1, },
        },);
        expect([...stopped.keys(),],).toEqual(['a',],);
        const empty = await pMapKeyed({
          input: new Map<string, number>(),
          mapper: function identity(value: number,): number {
            return value;
          },
        },);
        expect(empty.size,).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: pMapSettledKeyed.name,
  children: [
    it({
      name: 'settles each key into a null-prototype object',
      fn: async () => {
        /**
         Failure the second value's mapper call produces.
         */
        const failure = new Error('b failed',);
        const result = await pMapSettledKeyed({
          input: {
            a: 1,
            b: 2,
            c: Promise.reject<number>(new Error('c rejected',),),
          },
          mapper: function failOnB(
            value: number,
            key: string,
          ): number {
            if (key === 'b')
              throw failure;
            return value;
          },
        },);
        expect(Object.getPrototypeOf(result,),).toBe(null,);
        expect(result.a,).toEqual({
          status: 'fulfilled',
          value: 1,
        },);
        expect(result.b,).toEqual({
          status: 'rejected',
          reason: failure,
        },);
        expect(result.c?.status,).toBe('rejected',);
      },
    },),

    it({
      name: 'settles each key of a Map into a Map',
      fn: async () => {
        const result = await pMapSettledKeyed({
          input: new Map<string, number>([
            [
              'a',
              1,
            ],
            [
              'b',
              2,
            ],
          ],),
          mapper: function failOnB(
            value: number,
            key: string,
          ): number {
            if (key === 'b')
              throw new Error('b failed',);
            return value;
          },
        },);
        expect(result,).toBeInstanceOf(Map,);
        expect([...result.keys(),],).toEqual([
          'a',
          'b',
        ],);
        expect(result.get('a',),).toEqual({
          status: 'fulfilled',
          value: 1,
        },);
        expect(result.get('b',)?.status,).toBe('rejected',);
      },
    },),

    it({
      name: 'keeps an undefined mapper error and wraps a result that looks settled',
      fn: async () => {
        const result = await pMapSettledKeyed({
          input: {
            a: 1,
            b: 2,
          },
          mapper: function mixed(
            value: number,
            key: string,
          ): unknown {
            if (key === 'a')
              // oxlint-disable-next-line no-throw-literal, typescript/only-throw-error -- the case under test is a mapper that throws `undefined`
              throw undefined;
            return {
              status: 'rejected',
              reason: value,
            };
          },
        },);
        expect(result,).toEqual({
          a: {
            status: 'rejected',
            reason: undefined,
          },
          b: {
            status: 'fulfilled',
            value: {
              status: 'rejected',
              reason: 2,
            },
          },
        },);
      },
    },),

    it({
      name: 'leaves out skipped keys and keeps only the keys before a stop',
      fn: async () => {
        const result = await pMapSettledKeyed({
          input: {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
          },
          mapper: function control(
            value: number,
            key: string,
          ): number | typeof pMapSkip | typeof pMapStop {
            if (key === 'b')
              return pMapSkip;
            if (key === 'd')
              return pMapStop;
            return value;
          },
          options: { concurrency: 1, },
        },);
        expect(Object.keys(result,),).toEqual([
          'a',
          'c',
        ],);
      },
    },),
  ],
},);
