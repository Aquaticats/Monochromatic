/**
 Deterministic coverage driver: exercises every cache member, error path,
 and iteration ordering with fixed inputs, so the V8 coverage it produces
 is reproducible. Run under `NODE_V8_COVERAGE` by the `fuzz:coverage` task,
 then summarized by `coverage-report.ts`.
 
 @module
 */

import {
  createQuickLru,
} from '@monochromatic-dev/upstream-fork-quick-lru/ts';
import { installFakeClock, } from '@monochromatic-dev/upstream-fork-quick-lru/ts/test-support.ts';

//region Fixtures

/**
 Instant the controlled clock starts from.
 */
const CLOCK_START = 1_700_000_000_000;

/**
 Fractional eviction count exercising upstream's `Math.trunc` rounding.
 */
const FRACTIONAL_EVICT_COUNT = 2.5;

/**
 Eviction events recorded by every notified cache below.
 */
const evicted: string[] = [];

/**
 Builds one eviction callback appending to {@link evicted}.
 
 @returns Named callback matching upstream's `onEviction` argument list.
 
 @example
 ```ts
 const lru = createQuickLru({ onEviction: createEvictionRecorder(), },);
 ```
 */
function createEvictionRecorder(): (
  key: string,
  value: string,
) => void {
  return function recordEviction(
    key: string,
    value: string,
  ): void {
    evicted.push(`${key}=${value}`,);
  };
}

/**
 Reads every symbol-keyed member once, so the runtime-only members are
 exercised even though types cannot name them.
 
 @param lru - Cache under inspection.
 */
function readSymbolMembers(lru: object,): void {
  /**
   Cache carrying the runtime-only members as indexable slots.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- `object` widens to `Record<PropertyKey, unknown>` so the symbol-keyed members can be read at all; the type cannot name them because `Symbol.for` keys are not expressible in type literals
  const inspectable = lru as Record<PropertyKey, unknown>;
  void inspectable[Symbol.toStringTag];
  /**
   Node custom inspection output, produced exactly as `node:util`
   inspection produces it.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the runtime-only inspection member is invoked the same way `node:util` invokes it, and its signature cannot be typed because `Symbol.for` keys are not expressible in type literals
  const inspected = (inspectable[Symbol.for('nodejs.util.inspect.custom',)] as () => string)();
  void inspected;
}

//endregion Fixtures

//region Helpers

/**
 Runs a thunk that is expected to throw, swallowing `Error` outcomes so the
 driver keeps exercising remaining paths. Re-throws anything that is not an
 `Error`.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad(): void {
   createQuickLru({ maxSize: 0, },);
 },);
 ```
 */
function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    if (!(Error.isError(error,)))
      throw error;
  }
}

//endregion Helpers

//region Driver

/**
 Drives one fixed scenario over a cache: storage, recency, expiry,
 deletion, resize, evict, and every iteration ordering.
 
 @param options - Cache under exercise and the clock it reads.
 */
function exerciseScenario(options: {
  /**
   Cache under exercise.
   */
  readonly lru: ReturnType<typeof createQuickLru<string, string>>;
  /**
   Controlled clock the scenario moves.
   */
  readonly clock: ReturnType<typeof installFakeClock>;
},): void {
  options.lru
    .set({
      key: 'a',
      value: 'alpha',
    },);
  options.lru
    .set({
      key: 'b',
      value: 'beta',
    },);
  options.lru
    .set({
      key: 'c',
      value: 'gamma',
    },);
  options.lru
    .set({
      key: 'a',
      value: 'alpha-2',
    },);
  options.lru
    .set({
      key: 'd',
      value: 'delta',
      maxAge: 50,
    },);
  options.lru
    .set({
      key: 'e',
      value: 'epsilon',
      maxAge: Number.NaN,
    },);
  options.lru
    .set({
      key: 'f',
      value: 'zeta',
      maxAge: 0,
    },);
  void options.lru
    .get('d',);
  void options.lru
    .get('b',);
  void options.lru
    .peek('c',);
  void options.lru
    .has('d',);
  void options.lru
    .expiresIn('d',);
  void options.lru
    .expiresIn('missing',);
  options.clock
    .advance(100,);
  void options.lru
    .get('f',);
  void options.lru
    .peek('f',);
  void options.lru
    .has('f',);
  void [...options.lru];
  void [...options.lru
    .entriesAscending()];
  void [...options.lru
    .entriesDescending()];
  void [...options.lru
    .entries()];
  void [...options.lru
    .keys()];
  void [...options.lru
    .values()];
  options.lru
    .forEach({
      callback: function visit(
        value: string,
        key: string,
      ): void {
        void `${key}=${value}:${String(options.lru
          .has(key,),)}`;
      },
    },);
  void options.lru
    .size;
  void options.lru
    .maxSize;
  void options.lru
    .maxAge;
  void options.lru
    .__oldCache;
  void options.lru
    .toString();
  readSymbolMembers(options.lru,);
  void options.lru
    .delete('a',);
  options.lru
    .resize(10,);
  options.lru
    .resize(2,);
  options.lru
    .resize(2,);
  options.lru
    .evict(1,);
  options.lru
    .evict(0,);
  options.lru
    .evict(FRACTIONAL_EVICT_COUNT,);
  options.lru
    .clear();
}

/**
 Drives the fixed scenarios over caches with and without `onEviction`, over
 the counter branches, and over every configuration failure.
 */
function main(): void {
  /**
   Clock controlling time for every scenario below.
   */
  const clock = installFakeClock({
    startMilliseconds: CLOCK_START,
  },);
  /**
   Notified cache whose evictions land in {@link evicted}.
   */
  const notified = createQuickLru<string, string>({
    maxSize: 2,
    maxAge: 500,
    onEviction: createEvictionRecorder(),
  },);
  exerciseScenario({
    lru: notified,
    clock,
  },);
  /**
   Unnotified cache exercising the absent-callback gates.
   */
  const unnotified = createQuickLru<string, string>({
    maxSize: 2,
    maxAge: 500,
  },);
  exerciseScenario({
    lru: unnotified,
    clock,
  },);
  /**
   Duplicate-key cache exercising the size tally branch and the stale
   duplicate eviction notification.
   */
  const duplicate = createQuickLru<string, string>({
    maxSize: 3,
    onEviction: createEvictionRecorder(),
  },);
  for (const key of [
    'a',
    'b',
    'c',
    'a',
    'd',
  ]) {
    duplicate.set({
      key,
      value: key,
    },);
  }
  void duplicate.size;
  void duplicate.__oldCache;
  /**
   NaN-lifetime cache exercising upstream's falsy-stamp rewrite.
   */
  const nanLifetime = createQuickLru<string, string>({
    maxSize: 2,
    maxAge: Number.NaN,
  },);
  nanLifetime.set({
    key: 'a',
    value: 'alpha',
    maxAge: -5,
  },);
  void nanLifetime.get('a',);
  nanLifetime.evict(100,);
  swallow(function rejectZeroMaxSize(): void {
    createQuickLru<string, string>({
      maxSize: 0,
    },);
  },);
  swallow(function rejectZeroMaxAge(): void {
    createQuickLru<string, string>({
      maxSize: 1,
      maxAge: 0,
    },);
  },);
  swallow(function rejectZeroResize(): void {
    notified.resize(0,);
  },);
  clock.restore();
  void evicted;
}

main();

//endregion Driver
