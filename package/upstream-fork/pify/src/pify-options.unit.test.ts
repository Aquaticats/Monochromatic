/**
 Tests for option resolution semantics: upstream `pify`'s defaults, the
 spread-merge quirks where an explicitly `undefined` option overrides its
 default, and per-call `promiseModule` reads, exercised through the public
 `pify` entry point.
 
 @module
 */

/* oxlint-disable promise/prefer-await-to-callbacks -- External-boundary mirror: this package wraps node-style callback functions, so its fixtures and wrapped-function declarations implement the callback pattern the tests exercise; see DECISION.callback-capture.md. */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pify, } from '../dist/final/neutral/index.mjs';

await describe({
  name: 'pify-options',
  children: [
    //region Defaults

    it({
      name: 'errorFirst defaults to true',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'ok',);
          },
        },);
        expect(await promisified({
          args: [],
        },),).toBe('ok',);
      },
    },),

    it({
      name: 'promiseModule defaults to the native Promise',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'ok',);
          },
        },);
        expect(promisified({
          args: [],
        },),).toBeInstanceOf(Promise,);
      },
    },),

    it({
      name: 'the default exclude pattern is suffix-anchored and needs a prefix',
      fn: async () => {
        /**
         Module mixing names the default pattern must and must not catch:
         `readSync` ends with the suffix behind a prefix, `Sync` has no
         prefix character, `xxStreamyy` does not end with the suffix, and
         `read` carries no suffix at all.
         */
        const module = {
          readSync(): string {
            return 'raw';
          },
          Sync(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'p',);
          },
          xxStreamyy(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'p',);
          },
          read(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'p',);
          },
        };
        const pified = pify({
          input: module,
        },);
        /**
         Member classification per name: only `readSync` may pass through
         raw.
         */
        const shapes: Record<string, string> = {};
        for (const key of [
          'readSync',
          'Sync',
          'xxStreamyy',
          'read',
        ]) {
          try {
            /**
             Member call through the fork's call shape.
             */
            const memberCall = (pified as unknown as Record<string, (call: {
              readonly args: readonly unknown[];
            }) => unknown>)[key] as (call: {
              readonly args: readonly unknown[];
            }) => unknown;
            const result = memberCall({
              args: [],
            },);
            shapes[key] = (typeof (result as { then?: unknown; }).then) === 'function'
              ? 'promisified'
              : 'raw';
          }
          catch (error) {
            shapes[key] = `throws ${(error as Error).name}`;
          }
        }
        expect(shapes,).toEqual({
          readSync: 'raw',
          Sync: 'promisified',
          xxStreamyy: 'promisified',
          read: 'promisified',
        },);
      },
    },),

    //endregion Defaults

    //region Explicit undefined overrides

    it({
      name: 'quirk: explicit undefined errorFirst beats the default and resolves the error argument',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: (error: unknown, value: unknown) => void): void {
            callback('boom', 'ignored',);
          },
          options: {
            // Upstream `pify` accepts explicitly-undefined option values at runtime and the spread-merge quirk (they override defaults) is exactly what is under test here.
            errorFirst: undefined as unknown as boolean,
          },
        },);
        expect(await promisified({
          args: [],
        },),).toBe('boom',);
      },
    },),

    it({
      name: 'quirk: explicit undefined exclude beats the default and crashes on member selection',
      fn: async () => {
        /**
         Module whose member call triggers the crashed selection.
         */
        const module = {
          m(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'ok',);
          },
        };
        const pified = pify({
          input: module,
          options: {
            // Upstream `pify` accepts explicitly-undefined option values at runtime and the spread-merge quirk (they override defaults) is exactly what is under test here.
            exclude: undefined as unknown as readonly (keyof typeof module)[],
          },
        },);
        /**
         Failure captured at first member access, exactly like upstream
         `pify` reaching `undefined.some`.
         */
        let thrown: unknown = 'unset';
        try {
          (pified as unknown as {
            m: (call: { readonly args: readonly unknown[]; }) => unknown;
          }).m({
            args: [],
          },);
        }
        catch (error) {
          thrown = error;
        }
        expect(thrown,).toBeInstanceOf(TypeError,);
        expect((thrown as Error).message,).toContain('some',);
      },
    },),

    //endregion Explicit undefined overrides

    //region Option pass-through

    it({
      name: 'multiArgs false keeps the single-result callback shape',
      fn: async () => {
        const promisified = pify({
          input: function fixture(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'only',);
          },
          options: {
            multiArgs: false,
          },
        },);
        expect(await promisified({
          args: [],
        },),).toBe('only',);
      },
    },),

    it({
      name: 'promiseModule is read at call time from the resolved options',
      fn: async () => {
        /**
         Constructions counted by the custom promise constructor.
         */
        const state = {
          constructed: 0,
        };
        /**
         Minimal promise constructor stand-in counting its constructions.
         
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
          input: function fixture(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'tracked',);
          },
          options: {
            promiseModule: TrackingPromise as unknown as PromiseConstructor,
          },
        },);
        await promisified({
          args: [],
        },);
        await promisified({
          args: [],
        },);
        expect(state.constructed,).toBe(2,);
      },
    },),

    //endregion Option pass-through
  ],
},);
