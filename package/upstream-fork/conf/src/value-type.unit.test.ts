/**
 Value-type gate tests: JSON-representable values pass, while `undefined`,
 `symbol`, and `function` values are rejected with upstream's exact
 `UnsupportedValueTypeError` message.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  checkValueType,
  UnsupportedValueTypeError,
} from '../dist/final/neutral/index.mjs';

/**
 Store key reported verbatim in the diagnostics under test.
 */
const KEY = 'payload';

/**
 Function-valued fixture the JSON gate must reject.
 
 @returns Sample text proving the fixture is an ordinary callable.
 */
function unsupportedFunction(): string {
  return 'not storable';
}

/**
 Runs `fn` and returns the error it throws,
 so assertions can pin the exact class and message.
 
 @param fn - Call expected to throw.
 
 @returns The caught error instance.
 
 @throws Error when `fn` returns normally or throws a non-Error value.
 
 @example
 ```ts
 const thrown = captureThrown(function attempt(): void {},);
 ```
 */
function captureThrown(fn: () => void,): Error {
  try {
    fn();
  }
  catch (caught) {
    if (Error.isError(caught,))
      return caught;
    throw caught;
  }
  throw new Error('expected the guarded call to throw',);
}

/**
 Builds the exact upstream `UnsupportedValueTypeError` message for one
 rejected `typeof` name.
 
 @param type - `typeof` name of the rejected value.
 
 @returns The upstream message text for `KEY` verbatim.
 
 @example
 ```ts
 unsupportedMessage('undefined',); // full upstream diagnostic
 ```
 */
function unsupportedMessage(type: string,): string {
  return `Setting a value of type \`${type}\` for key \`${KEY}\` is not allowed as it's not supported by JSON`;
}

await describe({
  name: checkValueType.name,
  children: [
    it({
      name: 'accepts JSON-representable primitives without throwing',
      fn: async () => {
        for (const value of [
          'text',
          42,
          true,
          false,
          null,
        ]) {
          checkValueType({
            key: KEY,
            value,
          },);
        }
      },
    },),

    it({
      name: 'accepts JSON-representable objects and arrays without throwing',
      fn: async () => {
        for (const value of [
          {},
          {
            nested: {
              count: 1,
            },
          },
          [],
          [
            1,
            'two',
            null,
          ],
        ]) {
          checkValueType({
            key: KEY,
            value,
          },);
        }
      },
    },),

    it({
      name: 'rejects undefined values with the upstream message',
      fn: async () => {
        /**
         Rejected call whose error is captured below.
         */
        const thrown = captureThrown(function attemptUndefined(): void {
          checkValueType({
            key: KEY,
            value: undefined,
          },);
        },);

        expect(thrown,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(thrown.message,).toBe(unsupportedMessage('undefined',),);
      },
    },),

    it({
      name: 'rejects symbol values with the upstream message',
      fn: async () => {
        /**
         Rejected call whose error is captured below.
         */
        const thrown = captureThrown(function attemptSymbol(): void {
          checkValueType({
            key: KEY,
            value: Symbol('symbol value rejected by the JSON config gate',),
          },);
        },);

        expect(thrown,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(thrown.message,).toBe(unsupportedMessage('symbol',),);
      },
    },),

    it({
      name: 'rejects function values with the upstream message',
      fn: async () => {
        /**
         Rejected call whose error is captured below.
         */
        const thrown = captureThrown(function attemptFunction(): void {
          checkValueType({
            key: KEY,
            value: unsupportedFunction,
          },);
        },);

        expect(thrown,).toBeInstanceOf(UnsupportedValueTypeError,);
        expect(thrown.message,).toBe(unsupportedMessage('function',),);
      },
    },),
  ],
},);
