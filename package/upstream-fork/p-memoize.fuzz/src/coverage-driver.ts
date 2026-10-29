/**
 Deterministic coverage driver: exercises every exported function and its
 error paths with fixed inputs, so the V8 coverage it produces is
 reproducible. Run under `NODE_V8_COVERAGE` by the `fuzz:coverage` task,
 then summarized by `coverage-report.ts`.
 
 @module
 */

import {
  NonMethodDecorationError,
  NotMemoizedError,
  pMemoize,
  pMemoizeClear,
  pMemoizeDecorator,
  PrivateMethodDecorationError,
  UnclearableCacheError,
} from '@monochromatic-dev/upstream-fork-p-memoize/ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  alwaysHitCache,
  asyncClearCache,
  cacheLog,
  recordingCache,
} from './coverage-caches.ts';

//region Fixtures

/**
 Monotonic counter answered on every wrapped invocation.
 */
const counterState = {
  index: 0,
};

/**
 Initializers registered by the decorator context double.
 */
const initializers: ((this: unknown,) => void)[] = [];

/**
 Wrapped function ignoring its key and counting invocations.
 
 @param _key - Cache key ignored by the fixture.
 
 @returns Next invocation ordinal.
 */
function counter(_key?: unknown,): number {
  return counterState.index++;
}

/**
 Wrapped function throwing before it can return a promise.
 
 @param _key - Cache key ignored by the fixture.
 
 @throws Error on every call.
 */
function syncThrower(_key: string,): Promise<never> {
  throw new Error('sync-boom',);
}

/**
 Wrapped function rejecting every call.
 
 @param _key - Cache key ignored by the fixture.
 */
function rejecting(_key: string,): Promise<never> {
  return Promise.reject(new Error('boom',),);
}

/**
 Wrapped function that must never run for the unknown-function clear path.
 
 @returns Resolved constant.
 */
function neverMemoized(): Promise<number> {
  return Promise.resolve(1,);
}

/**
 Stringifies the first argument for the custom-key fixtures.
 
 @param args - Argument tuple of one call.
 
 @returns String form of the first argument.
 */
function stringifyFirst(args: readonly unknown[],): string {
  /**
   First element of the argument tuple.
   */
  const [firstArgument] = args;
  return String(firstArgument,);
}

/**
 Stringifies the whole argument tuple for the JSON-key fixture.
 
 @param args - Argument tuple of one call.
 
 @returns JSON form of the argument tuple.
 */
function stringifyAll(args: readonly unknown[],): string {
  return JSON.stringify(args,);
}

/**
 Identity key for the WeakMap fixture.
 
 @param args - Argument tuple of one call.
 
 @returns First argument as an object key.
 */
function identityKey(args: readonly unknown[],): object {
  /**
   First element of the argument tuple.
   */
  const [firstArgument] = args;
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the WeakMap fixture keys by object identity, so the first argument must be narrowed from `unknown`
  return firstArgument as object;
}

/**
 Denies every cache write.
 
 @returns `false` on every call.
 */
function deny(): boolean {
  return false;
}

/**
 Throws instead of deciding, exercising the predicate-failure path.
 
 @returns Never; every call throws.
 
 @throws Error on every call.
 */
function explode(): boolean {
  throw new Error('predicate-boom',);
}

/**
 Allows every cache write asynchronously.
 
 @param _value - Fulfilled value ignored by the predicate.
 
 @returns Resolved `true`.
 */
function allow(_value: number,): Promise<boolean> {
  return Promise.resolve(true,);
}

/**
 Synthetic method context the decorator receives.
 */
const methodContext = {
  kind: 'method',
  private: false,
  name: 'counter',
  addInitializer: function addInitializer(initializer: (this: unknown,) => void,): void {
    initializers.push(initializer,);
  },
};

/**
 Decorated method body bumping the receiver's counter.
 
 The `this` parameter is the receiver the decorator initializer installs
 the memoized method onto.
 
 @returns Counter value before the increment.
 */
function decoratedMethod(this: {
  index: number;
},): number {
  return this.index++;
}

/**
 Instance shape the decorator initializer installs the memoized method
 onto.
 */
type DecoratedInstance = {
  /**
   Counter incremented through the memoized method.
   */
  index: number;
  /**
   Memoized method slot installed by the decorator initializer.
   */
  counter: (call: {
    args: readonly unknown[];
  },) => Promise<number>;
};

/**
 Prototype holding the decorated method where the decorator looks for it,
 as a class's prototype would.
 */
const decoratedPrototype = {
  counter: decoratedMethod,
};

/**
 Wrapped function for the WeakMap fixture.
 
 @param key - Identity key ignored by the fixture.
 
 @returns Resolved constant.
 */
function keyed(key: object,): Promise<number> {
  void key;
  return Promise.resolve(1,);
}

//endregion Fixtures

//region Driver

/**
 Settles one promise without letting its outcome propagate, for fixtures
 whose failures are the behavior under test.
 
 @param promise - Promise to settle silently.
 */
async function settleSilently(promise: Promise<unknown>,): Promise<void> {
  await Promise.allSettled([
    promise,
  ],);
}

/**
 Runs one throwing fixture and records its failure text, for the error
 paths the driver exercises deliberately.
 
 @param action - Fixture expected to throw.
 */
function captureThrow(action: () => void,): void {
  try {
    action();
  }
  catch (error) {
    cacheLog.push(`threw ${caughtValueText(error,)}`,);
  }
}

/**
 Runs every fixture sequence once; called at module load so the coverage
 report sees a single deterministic pass.
 */
async function runDriver(): Promise<void> {
  /**
   Memoized function reading and writing the recording cache.
   */
  const cached = pMemoize({
    fn: counter,
    options: {
      cache: recordingCache,
    },
  },);
  await cached({
    args: ['k'],
  },);
  await cached({
    args: ['k'],
  },);

  /**
   Memoized function with a custom key and a denying predicate.
   */
  const selective = pMemoize({
    fn: counter,
    options: {
      cache: recordingCache,
      cacheKey: stringifyFirst,
      shouldCache: deny,
    },
  },);
  await selective({
    args: [1],
  },);

  /**
   Memoized function whose predicate throws on every write.
   */
  const poisoned = pMemoize({
    fn: counter,
    options: {
      cache: recordingCache,
      shouldCache: explode,
    },
  },);
  await settleSilently(poisoned({
    args: ['p'],
  },),);

  /**
   Memoized function whose predicate allows writes asynchronously.
   */
  const awaited = pMemoize({
    fn: counter,
    options: {
      cache: recordingCache,
      shouldCache: allow,
    },
  },);
  await awaited({
    args: ['w'],
  },);

  /**
   Memoized function whose cache reports hits but reads `undefined`.
   */
  const alwaysHit = pMemoize({
    fn: counter,
    options: {
      cache: alwaysHitCache,
    },
  },);
  await alwaysHit({
    args: ['h'],
  },);

  /**
   Argument pairs the JSON-keyed fixture's wrapped function observed.
   */
  const recordedArguments: unknown[] = [];
  /**
   Memoized function keying by the whole argument tuple; its wrapped
   function is the memoizer's callback, so its two-parameter shape is
   dictated by the call protocol, and it records into the local list above.
   */
  const jsonKeyed = pMemoize({
    fn: function counterPair(
      _first: unknown,
      _second: unknown,
    ): number {
      recordedArguments.push(
        _first,
        _second,
      );
      return counterState.index++;
    },
    options: {
      cacheKey: stringifyAll,
    },
  },);
  await jsonKeyed({
    args: [
      1,
      2,
    ],
  },);
  await jsonKeyed({
    args: [
      1,
      2,
    ],
  },);

  /**
   Memoized function whose wrapped call rejects every time.
   */
  const flaky = pMemoize({
    fn: rejecting,
  },);
  await settleSilently(flaky({
    args: ['r'],
  },),);
  await settleSilently(flaky({
    args: ['r'],
  },),);

  /**
   Memoized function with caching disabled: concurrent de-duplication.
   */
  const uncached = pMemoize({
    fn: counter,
    options: {
      cache: false,
    },
  },);
  await Promise.all([
    uncached({
      args: ['u'],
    },),
    uncached({
      args: ['u'],
    },),
  ],);

  /**
   Memoized function with caching disabled whose wrapped call throws
   synchronously, exercising the disabled-cache yield before the call.
   */
  const uncachedThrower = pMemoize({
    fn: syncThrower,
    options: {
      cache: false,
    },
  },);
  /**
   First call's promise; the second call must join its in-flight entry.
   */
  const replayed = uncachedThrower({
    args: ['t'],
  },);
  await settleSilently(uncachedThrower({
    args: ['t'],
  },),);
  await settleSilently(replayed,);

  /**
   Memoized function over a WeakMap cache, cleared through `pMemoizeClear`.
   */
  const weak = pMemoize({
    fn: keyed,
    options: {
      cache: new WeakMap<object, number>(),
      cacheKey: identityKey,
    },
  },);
  await weak({
    args: [{}],
  },);
  captureThrow(function clearWeak(): void {
    pMemoizeClear(weak,);
  },);

  // pMemoizeClear paths: success, asynchronous clear, disabled cache, unknown
  // function.
  pMemoizeClear(cached,);
  /**
   Memoized function over a cache whose `clear` returns a promise.
   */
  const asyncCleared = pMemoize({
    fn: counter,
    options: {
      cache: asyncClearCache,
    },
  },);
  await asyncCleared({
    args: ['z'],
  },);
  pMemoizeClear(asyncCleared,);
  captureThrow(function clearDisabled(): void {
    pMemoizeClear(pMemoize({
      fn: counter,
      options: {
        cache: false,
      },
    },),);
  },);
  captureThrow(function clearUnknown(): void {
    pMemoizeClear(neverMemoized,);
  },);

  // Decorator: per-instance initialization.
  /**
   Decorator under test, applied through the synthetic context below.
   */
  const decorate = pMemoizeDecorator();
  /* oxlint-disable typescript/no-unsafe-type-assertion -- the synthetic decorator context is a test double for the runtime-supplied `ClassMethodDecoratorContext` */
  /**
   Synthetic method context asserted into the decorator protocol slot.
   */
  const methodContextDouble = methodContext as never;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  decorate(
    decoratedMethod,
    methodContextDouble,
  );
  /**
   Instance whose counter the memoized method bumps.
   */
  // oxlint-disable-next-line typescript/no-unsafe-assignment -- `Object.create` is typed `any`; the prototype chain is what the decorator inspects
  const instance: DecoratedInstance = Object.assign(
    Object.create(decoratedPrototype,),
    {
      index: 0,
    },
  );
  for (const initialize of initializers)
    initialize.call(instance,);
  await instance.counter({
    args: [],
  },);

  // Decorator validation failures.
  /* oxlint-disable typescript/no-unsafe-type-assertion -- non-method decorator context double for the `NonMethodDecorationError` path */
  /**
   Non-method context double for the `NonMethodDecorationError` path.
   */
  const getterContextDouble = {
    kind: 'getter',
  } as never;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  captureThrow(function decorateGetter(): void {
    decorate(
      decoratedMethod,
      getterContextDouble,
    );
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- private-method decorator context double for the `PrivateMethodDecorationError` path */
  /**
   Private-method context double for the `PrivateMethodDecorationError`
   path.
   */
  const privateContextDouble = {
    kind: 'method',
    private: true,
  } as never;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  captureThrow(function decoratePrivate(): void {
    decorate(
      decoratedMethod,
      privateContextDouble,
    );
  },);

  // Error class construction, one per class.
  void new NotMemoizedError();
  void new PrivateMethodDecorationError();
  void new NonMethodDecorationError();
  void new UnclearableCacheError();
}

await runDriver();

//endregion Driver
