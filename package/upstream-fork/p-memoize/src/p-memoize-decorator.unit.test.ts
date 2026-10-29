/**
 Tests for `pMemoizeDecorator`: per-instance installation through the
 decorator protocol's initializer, and the two target-validation failures.
 
 The tests drive the decorator through its function interface with synthetic
 contexts because Node's type stripping cannot execute decorator syntax;
 the protocol shapes are the standard library's.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type MemoizedFunction,
  NonMethodDecorationError,
  pMemoizeClear,
  pMemoizeDecorator,
  PrivateMethodDecorationError,
} from '../dist/final/neutral/index.mjs';

/**
 Decorated method body shared by the tests: bumps and returns the
 receiver's counter.
 
 @returns Counter value before the increment.
 */
async function counterMethod(this: CounterInstance,): Promise<number> {
  return this.index++;
}

/**
 Instance shape the decorated method is installed onto.
 */
type CounterInstance = {
  /**
   Counter incremented through the memoized method.
   */
  index: number;
  /**
   Memoized method slot installed by the decorator initializer.
   */
  counter: MemoizedFunction<Parameters<typeof counterMethod>, number>;
};

/**
 Synthetic decorator context plus the initializer it captured.
 */
type FakeDecoration = {
  /**
   Context handed to the decorator in place of the runtime's.
   */
  readonly context: ClassMethodDecoratorContext<CounterInstance>;
  /**
   Initializer captured from `addInitializer`, runnable per instance.
   */
  readonly initialize: (instance: CounterInstance,) => void;
};

/**
 Builds one synthetic method-decoration context capturing its initializer.
 
 @param contextFields - `kind` and `private` flags the decorator validates.
 
 @returns Context and a runner that applies the captured initializer to one
 instance.
 
 @example
 ```ts
 const decoration = createFakeDecoration({
   kind: 'method',
   isPrivate: false,
 });
 ```
 */
function createFakeDecoration({
  kind,
  isPrivate,
}: {
  readonly kind: string;
  readonly isPrivate: boolean;
},): FakeDecoration {
  /**
   Initializers registered by the decorator under test.
   */
  const initializers: ((this: CounterInstance,) => void)[] = [];
  /**
   Synthetic context standing in for the runtime-supplied one.
   */
  const context = {
    kind,
    private: isPrivate,
    name: 'counter',
    addInitializer: function addInitializer(initializer: (this: CounterInstance,) => void,): void {
      initializers.push(initializer,);
    },
  } as unknown as ClassMethodDecoratorContext<CounterInstance>;

  return {
    context,
    initialize: function initialize(instance: CounterInstance,): void {
      /**
       Initializer the decorator registered for this context.
       */
      const [initializer] = initializers;

      if (initializer === undefined)
        throw new Error('decorator registered no initializer',);

      initializer.call(instance,);
    },
  };
}

await describe({
  name: pMemoizeDecorator.name,
  children: [
    it({
      name: 'memoizes each instance separately',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         First instance getting its own memoized method.
         */
        const alpha = {
          index: 0,
        } as CounterInstance;
        /**
         Second instance that must not share the first one's cache.
         */
        const beta = {
          index: 100,
        } as CounterInstance;
        decoration.initialize(alpha,);
        decoration.initialize(beta,);

        expect(await alpha.counter({
          args: [],
        },),).toBe(0,);
        expect(await alpha.counter({
          args: [],
        },),).toBe(0,);
        expect(await beta.counter({
          args: [],
        },),).toBe(100,);
        expect(await beta.counter({
          args: [],
        },),).toBe(100,);
      },
    },),

    it({
      name: 'installs the memoized method as an own non-enumerable property',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Instance whose own property shape is inspected.
         */
        const instance = {
          index: 0,
        } as CounterInstance;
        decoration.initialize(instance,);

        /**
         Own descriptor of the installed memoized method.
         */
        const descriptor = Object.getOwnPropertyDescriptor(
          instance,
          'counter',
        ) as PropertyDescriptor;
        expect(descriptor.enumerable,).toBe(false,);
        expect(descriptor.writable,).toBe(true,);
        expect(descriptor.configurable,).toBe(true,);
        expect(typeof descriptor.value,).toBe('function',);
      },
    },),

    it({
      name: 'clears one instance cache through pMemoizeClear',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Instance whose cache the test clears.
         */
        const alpha = {
          index: 0,
        } as CounterInstance;
        /**
         Instance whose cache must stay intact.
         */
        const beta = {
          index: 100,
        } as CounterInstance;
        decoration.initialize(alpha,);
        decoration.initialize(beta,);

        await alpha.counter({
          args: [],
        },);
        await beta.counter({
          args: [],
        },);
        pMemoizeClear(alpha.counter,);

        expect(await alpha.counter({
          args: [],
        },),).toBe(1,);
        expect(await beta.counter({
          args: [],
        },),).toBe(100,);
      },
    },),

    it({
      name: 'throws NonMethodDecorationError for a non-method target',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'getter',
          isPrivate: false,
        },);
        const decorate = pMemoizeDecorator();

        let caught: unknown;
        try {
          decorate(counterMethod, decoration.context,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NonMethodDecorationError,);
        expect((caught as Error).message,).toBe('pMemoizeDecorator can only decorate methods',);
      },
    },),

    it({
      name: 'throws PrivateMethodDecorationError for a private method',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: true,
        },);
        const decorate = pMemoizeDecorator();

        let caught: unknown;
        try {
          decorate(counterMethod, decoration.context,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(PrivateMethodDecorationError,);
        expect((caught as Error).message,).toBe('pMemoizeDecorator cannot decorate private methods',);
      },
    },),

    it({
      name: 'forwards cache options into the memoized method',
      fn: async () => {
        /**
         Predicate call counter proving the decorator's options reach the
         memoizer.
         */
        const state = {
          calls: 0,
        };
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator<Parameters<typeof counterMethod>, number>({
          shouldCache: function countCalls(): boolean {
            state.calls += 1;
            return true;
          },
        })(counterMethod, decoration.context,);
        /**
         Instance whose method runs with the forwarded options.
         */
        const instance = {
          index: 0,
        } as CounterInstance;
        decoration.initialize(instance,);

        await instance.counter({
          args: [],
        },);
        expect(state.calls,).toBe(1,);
      },
    },),
  ],
},);
