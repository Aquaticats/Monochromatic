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
 
 @param contextFields - `kind` and `private` flags the decorator validates,
 plus the optional method name and static flag.
 
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
  name = 'counter',
  isStatic = false,
}: {
  readonly kind: string;
  readonly isPrivate: boolean;
  readonly name?: string | symbol;
  readonly isStatic?: boolean;
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
    static: isStatic,
    name,
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

/**
 Prototype the instances inherit the decorated method from, as a class's
 prototype would hold it.
 */
const counterPrototype = {
  counter: counterMethod,
};

/**
 Builds one instance whose prototype defines the decorated method, which is
 where the decorator looks for it.

 @param index - Starting counter value of the instance.

 @returns Instance inheriting `counterPrototype`.

 @example
 ```ts
 const instance = createInstance(0,);
 ```
 */
function createInstance(index: number,): CounterInstance {
  return Object.assign(
    Object.create(counterPrototype,) as CounterInstance,
    {
      index,
    },
  );
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
        const alpha = createInstance(0,);
        /**
         Second instance that must not share the first one's cache.
         */
        const beta = createInstance(100,);
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
        const instance = createInstance(0,);
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
        const alpha = createInstance(0,);
        /**
         Instance whose cache must stay intact.
         */
        const beta = createInstance(100,);
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
        const instance = createInstance(0,);
        decoration.initialize(instance,);

        await instance.counter({
          args: [],
        },);
        expect(state.calls,).toBe(1,);
      },
    },),

    it({
      name: 'stands down when a subclass overrides the method',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Subclass prototype whose override replaces the decorated method.
         */
        const subclassPrototype = Object.create(counterPrototype,) as object;
        Object.defineProperty(subclassPrototype, 'counter', {
          value: async function override(): Promise<number> {
            return -1;
          },
        },);
        /**
         Instance of the overriding subclass.
         */
        const instance = Object.create(subclassPrototype,) as CounterInstance;
        decoration.initialize(instance,);

        expect(Object.hasOwn(instance, 'counter',),).toBe(false,);
        expect(await (instance.counter as unknown as () => Promise<number>)(),).toBe(-1,);
      },
    },),

    it({
      name: 'stands down when the method is deleted from the prototype',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Instance whose chain no longer defines the method.
         */
        const instance = Object.create(Object.create(null,) as object,) as CounterInstance;
        decoration.initialize(instance,);

        expect(Object.hasOwn(instance, 'counter',),).toBe(false,);
      },
    },),

    it({
      name: 'does not invoke a subclass accessor while checking for an override',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Counter proving whether the accessor ran.
         */
        const state = {
          reads: 0,
        };
        /**
         Subclass prototype that replaced the method with a throwing getter.
         */
        const subclassPrototype = Object.create(counterPrototype,) as object;
        Object.defineProperty(subclassPrototype, 'counter', {
          get: function readCounter(): never {
            state.reads += 1;
            throw new Error('getter exploded',);
          },
        },);
        /**
         Instance of the subclass with the accessor.
         */
        const instance = Object.create(subclassPrototype,) as CounterInstance;
        decoration.initialize(instance,);

        expect(state.reads,).toBe(0,);
        expect(Object.hasOwn(instance, 'counter',),).toBe(false,);
      },
    },),

    it({
      name: 'memoizes a method that a subclass inherits without overriding',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Instance three levels down a chain that never overrides the method.
         */
        const grandparent = Object.create(counterPrototype,) as object;
        const parent = Object.create(grandparent,) as object;
        const instance = Object.assign(
          Object.create(parent,) as CounterInstance,
          {
            index: 0,
          },
        );
        decoration.initialize(instance,);

        expect(Object.hasOwn(instance, 'counter',),).toBe(true,);
        expect(await instance.counter({
          args: [],
        },),).toBe(0,);
        expect(await instance.counter({
          args: [],
        },),).toBe(0,);
      },
    },),

    it({
      name: 'stands down three levels down a chain that overrides the method',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Middle prototype holding the override.
         */
        const middle = Object.create(counterPrototype,) as object;
        Object.defineProperty(middle, 'counter', {
          value: async function override(): Promise<number> {
            return -1;
          },
        },);
        /**
         Instance below the overriding link.
         */
        const instance = Object.create(Object.create(middle,) as object,) as CounterInstance;
        decoration.initialize(instance,);

        expect(Object.hasOwn(instance, 'counter',),).toBe(false,);
      },
    },),

    it({
      name: 'memoizes a static method on the class itself and stands down for a static override',
      fn: async () => {
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
          isStatic: true,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Class-like function holding the decorated method as a static.
         */
        const base = Object.assign(function Base(): void {}, {
          index: 0,
          counter: counterMethod,
        },) as unknown as CounterInstance;
        decoration.initialize(base,);

        expect(Object.hasOwn(base, 'counter',),).toBe(true,);
        expect(await base.counter({
          args: [],
        },),).toBe(0,);
        expect(await base.counter({
          args: [],
        },),).toBe(0,);

        /**
         Subclass-like function that overrides the static method.
         */
        const derived = Object.assign(function Derived(): void {}, {
          index: 0,
          counter: async function override(): Promise<number> {
            return -1;
          },
        },) as unknown as CounterInstance;
        decoration.initialize(derived,);

        expect(derived.counter,).not.toBe(base.counter,);
        expect(await (derived.counter as unknown as () => Promise<number>)(),).toBe(-1,);
      },
    },),

    it({
      name: 'memoizes a method named by a symbol',
      fn: async () => {
        /**
         Symbol naming the decorated method.
         */
        const name = Symbol('decorated counter method name',);
        const decoration = createFakeDecoration({
          kind: 'method',
          isPrivate: false,
          name,
        },);
        pMemoizeDecorator()(counterMethod, decoration.context,);
        /**
         Instance whose prototype holds the method under the symbol.
         */
        const instance = Object.assign(
          Object.create({
            [name]: counterMethod,
          },) as CounterInstance,
          {
            index: 0,
          },
        );
        decoration.initialize(instance,);

        expect(Object.hasOwn(instance, name,),).toBe(true,);
      },
    },),

    ...[
      null,
      undefined,
      42,
      'text',
      true,
      Symbol('primitive receiver of initializer',),
    ].map(function mapReceiver(receiver,) {
      return it({
        name: `stands down when the initializer receiver is the primitive ${String(receiver,)}`,
        fn: async () => {
          const decoration = createFakeDecoration({
            kind: 'method',
            isPrivate: false,
          },);
          pMemoizeDecorator()(counterMethod, decoration.context,);

          expect(function initializeWithPrimitive(): void {
            decoration.initialize(receiver as unknown as CounterInstance,);
          },).not.toThrow();
        },
      },);
    },),
  ],
},);
