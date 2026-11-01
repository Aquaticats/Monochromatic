/**
 Tests for the proxy view `pify` returns: member preservation, function
 modules, `excludeMain`, mutation visibility, prototype chains, and the
 overload type surface.
 
 @module
 */

/* oxlint-disable promise/prefer-await-to-callbacks -- External-boundary mirror: this package wraps node-style callback functions, so its fixtures and wrapped-function declarations implement the callback pattern the tests exercise; see DECISION.callback-capture.md. */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type EmptyTuple,
  pify,
} from '../dist/final/neutral/index.mjs';

/**
 Callback-style function with a trailing error-first callback, mirroring
 upstream `pify`'s type-test fixture of the same name.
 
 @param x - Value forwarded to the callback.
 
 @param function_ - Trailing error-first callback.
 */
function function1(x: number, function_: (error: Error, value: number) => void): void {
  function_(null as unknown as Error, x,);
}

/**
 Callback-style function whose callback reports only a value, mirroring
 upstream `pify`'s type-test fixture of the same name.
 
 @param function_ - Trailing value-only callback.
 */
function function0(function_: (value: number) => void): void {
  function_(1,);
}

/**
 Fixture module mixing method shapes for the mapped type, mirroring upstream
 `pify`'s type-test fixture of the same name.
 */
const fixtureModule = {
  method1(argument: string, callback: (error: Error, value: string) => void): void {
    callback(null as unknown as Error, argument,);
  },
  methodSync(argument: 'sync'): 'sync' {
    return argument;
  },
  property: 3,
};

await describe({
  name: pify.name,
  children: [
    //region Member preservation

    it({
      name: 'preserves member enumeration and non-function members',
      fn: async () => {
        const module = {
          method(): void {},
          nonMethod: 3,
        };
        const pified = pify({
          input: module,
        },);
        expect(
          Object.keys(pified as unknown as object,),
        ).toEqual(Object.keys(module,),);
        expect(
          (pified as unknown as typeof module).nonMethod,
        ).toBe(3,);
      },
    },),

    it({
      name: 'reflects method mutation after wrapping',
      fn: async () => {
        /**
         Mutable module whose method is replaced after wrapping.
         */
        const mutable: {
          foo: (callback: (error: unknown, value: unknown) => void) => void;
        } = {
          foo(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'original',);
          },
        };
        const pified = pify({
          input: mutable,
        },);
        mutable.foo = function replaced(callback: (error: unknown, value: unknown) => void): void {
          callback(null, 'new',);
        };
        expect(
          await (pified as unknown as typeof mutable).foo({
            args: [],
          } as never,),
        ).toBe('new',);
      },
    },),

    //endregion Member preservation

    //region Function modules

    it({
      name: 'promisifies the own call and members of a function module',
      fn: async () => {
        /**
         Function module carrying a method, upstream pify's fixture shape.
         */
        function moduleFunction(callback: (error: unknown, value: unknown) => void): void {
          callback(null, 'main',);
        }
        moduleFunction.meow = function meow(callback: (error: unknown, value: unknown) => void): void {
          callback(null, 'meow',);
        };
        const pified = pify({
          input: moduleFunction,
        },);
        expect(
          await (pified as unknown as (call: { readonly args: readonly unknown[]; }) => Promise<unknown>)({
            args: [],
          },),
        ).toBe('main',);
        expect(
          await (pified as unknown as { meow: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>; }).meow({
            args: [],
          },),
        ).toBe('meow',);
      },
    },),

    it({
      name: 'excludeMain keeps the function module own call raw',
      fn: async () => {
        /**
         Function module whose raw own call proves it was not promisified.
         */
        function moduleFunction(callback: (error: unknown, value: unknown) => void): void {
          callback(null, 'main',);
        }
        moduleFunction.meow = function meow(callback: (error: unknown, value: unknown) => void): void {
          callback(null, 'meow',);
        };
        const pified = pify({
          input: moduleFunction,
          options: {
            excludeMain: true,
          },
        },);
        /**
         Raw own call through the proxy's apply trap: with no callback
         supplied the raw function throws exactly like calling it raw.
         */
        let thrown: unknown = 'unset';
        try {
          (pified as unknown as (call: { readonly args: readonly unknown[]; }) => unknown)({
            args: [],
          },);
        }
        catch (error) {
          thrown = error;
        }
        expect(thrown,).toBeInstanceOf(TypeError,);
        expect(
          await (pified as unknown as { meow: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>; }).meow({
            args: [],
          },),
        ).toBe('meow',);
      },
    },),

    //endregion Function modules

    //region Prototype chains

    it({
      name: 'promisifies inherited methods and preserves the prototype chain',
      fn: async () => {
        /**
         Grandparent fixture with one own method and one overridden later.
         */
        const grandparent = {
          grandparentMethod1(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'grandparent',);
          },
          overriddenMethod1(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'grandparent',);
          },
        };
        /**
         Parent fixture overriding one inherited method and value.
         */
        const parent = Object.assign(Object.create(grandparent,), {
          parentMethod1(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'parent',);
          },
          overriddenMethod1(value: string, callback: (error: unknown, value: unknown) => void): void {
            callback(null, value,);
          },
          overriddenValue1: 2,
        },) as {
          parentMethod1: (callback: (error: unknown, value: unknown) => void) => void;
          overriddenMethod1: (value: string, callback: (error: unknown, value: unknown) => void) => void;
          overriddenValue1: number;
        };
        /**
         Instance fixture adding own members over the chain.
         */
        const instance = Object.assign(Object.create(parent,), {
          instanceMethod1(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'instance',);
          },
          value1: 'neo',
        },) as {
          instanceMethod1: (callback: (error: unknown, value: unknown) => void) => void;
          value1: string;
        };
        const pified = pify({
          input: instance as unknown as Record<string, unknown>,
        },) as unknown as {
          grandparentMethod1: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>;
          parentMethod1: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>;
          instanceMethod1: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>;
          overriddenMethod1: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>;
          overriddenValue1: number;
          value1: string;
        };
        expect(Object.getPrototypeOf(pified as unknown as object,),).toBe(parent,);
        expect(await pified.grandparentMethod1({
          args: [],
        },),).toBe('grandparent',);
        expect(await pified.parentMethod1({
          args: [],
        },),).toBe('parent',);
        expect(await pified.instanceMethod1({
          args: [],
        },),).toBe('instance',);
        expect(await pified.overriddenMethod1({
          args: ['rainbow',],
        },),).toBe('rainbow',);
        expect(pified.overriddenValue1,).toBe(2,);
        expect(pified.value1,).toBe('neo',);
      },
    },),

    it({
      name: 'never promisifies Function.prototype members like bind',
      fn: async () => {
        /**
         Wrapped function reporting its receiver through the callback.
         */
        function fn(this: unknown, callback: (error: unknown, value: unknown) => void): void {
          callback(null, this,);
        }
        const pified = pify({
          input: fn,
        },);
        /**
         Bind target the wrapper must see as its receiver.
         */
        const target = {};
        expect(
          await (pified as unknown as (call: { readonly args: readonly unknown[]; }) => Promise<unknown>).bind(target,)({
            args: [],
          },),
        ).toBe(target,);
      },
    },),

    it({
      name: 'keeps working when a wrapped function references itself as a member',
      fn: async () => {
        /**
         Function module whose member points at the function itself.
         */
        function fn(first: number, callback: (error: unknown, value: unknown) => void): void {
          callback(null, first,);
        }
        fn.self = fn;
        const pified = pify({
          input: fn,
        },);
        expect(
          await (pified as unknown as {
            self: (call: { readonly args: readonly unknown[]; }) => Promise<unknown>;
          }).self({
            args: [0],
          },),
        ).toBe(0,);
      },
    },),

    //endregion Prototype chains

    it({
      name: 'rejects every non-function non-object input with the upstream message',
      fn: async () => {
        /**
         Inputs upstream `pify` refuses, paired with their message suffixes.
         */
        const invalidCases: readonly { readonly input: unknown; readonly shown: string; }[] = [
          {
            input: null,
            shown: 'null',
          },
          {
            input: undefined,
            shown: 'undefined',
          },
          {
            input: 42,
            shown: 'number',
          },
          {
            input: 'nope',
            shown: 'string',
          },
          {
            input: true,
            shown: 'boolean',
          },
        ];
        for (const { input, shown, } of invalidCases) {
          /**
           Wrap failure captured for its class and message.
           */
          let thrown: unknown = 'unset';
          try {
            pify({
              input: input as object,
            },);
          }
          catch (error) {
            thrown = error;
          }
          expect(thrown,).toBeInstanceOf(TypeError,);
          expect(
            (thrown as Error).message,
          ).toBe(`Expected \`input\` to be a \`Function\` or \`Object\`, got \`${shown}\``,);
        }
      },
    },),

    it({
      name: 'caches one promisified wrapper per member function',
      fn: async () => {
        const module = {
          read(callback: (error: unknown, value: unknown) => void): void {
            callback(null, 'p',);
          },
        };
        const pified = pify({
          input: module,
        },);
        expect(pified.read,).toBe(pified.read,);
      },
    },),

    //region Type surface

    it({
      name: 'overloads mirror upstream pify result inference',
      fn: async () => {
        expectTypeOf(
          await pify({
            input: function1,
          },)({
            args: [1],
          },),
        ).toEqualTypeOf<number>();
        expectTypeOf(
          pify({
            input: function0,
          },),
        ).toEqualTypeOf<(call: { readonly args: EmptyTuple; }) => Promise<unknown>>();
        expectTypeOf(
          pify({
            input: fixtureModule,
          },).property,
        ).toEqualTypeOf<number>();
        expectTypeOf(
          pify({
            input: fixtureModule,
          },).methodSync,
        ).toEqualTypeOf<(argument: 'sync') => 'sync'>();

        /**
         Smoke call keeping the assertions above on executed code paths.
         */
        const pified = pify({
          input: fixtureModule,
        },);
        expect(pified.property,).toBe(fixtureModule.property,);
      },
    },),

    //endregion Type surface
  ],
},);
