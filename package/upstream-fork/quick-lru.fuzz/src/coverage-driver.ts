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

import { renderPair, } from './cache-workload.ts';

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
 Eviction events recorded by every notified cache below, as `key=value`.
 */
const evicted: string[] = [];

/**
 Items the shared scenario stores, in insertion order.

 The eviction check compares against the same data the scenario drives,
 so a stored pair and the expectation of it cannot drift apart.
 */
const SCENARIO_ITEMS: readonly {
  /**
   Cache key.
   */
  readonly key: string;
  /**
   Cache value.
   */
  readonly value: string;
  /**
   Per-item lifetime the scenario stores.
   */
  readonly maxAge?: number;
}[] = [
  {
    key: 'a',
    value: 'alpha',
  },
  {
    key: 'b',
    value: 'beta',
  },
  {
    key: 'c',
    value: 'gamma',
  },
  {
    key: 'a',
    value: 'alpha-2',
  },
  {
    key: 'd',
    value: 'delta',
    maxAge: 50,
  },
  {
    key: 'e',
    value: 'epsilon',
    maxAge: Number.NaN,
  },
  {
    key: 'f',
    value: 'zeta',
    maxAge: 0,
  },
];

/**
 Keys the duplicate-key cache stores, each with itself as its value.
 */
const DUPLICATE_KEYS: readonly string[] = [
  'a',
  'b',
  'c',
  'a',
  'd',
];

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
    evicted.push(renderPair({
      key,
      value,
    },),);
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
  for (const item of SCENARIO_ITEMS) {
    options.lru
      .set(item,);
  }
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
 An eviction report the driver's workload cannot account for.
 */
class EvictionReportError extends Error {
  /**
   Names this class in the stack a failing gate prints.

   @param message - What the cache reported and why the workload rules it out.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'EvictionReportError';
  }
}

/**
 Confirms the notified caches reported evictions, and reported only pairs the
 workload stored.

 The driver has no oracle for which keys a cache must drop, so it does not
 assert the eviction order; the differential property test compares the fork's
 notifications against upstream for that. What this gate adds is that the
 callback fires at all and that every report names a pair the scenario
 inserted, which a swap of the callback's two arguments or a dead notification
 fails.

 @throws When no eviction was reported, or a report names a pair the workload
   never inserted.
 */
function assertEvictionsDescribeStoredPairs(): void {
  if (evicted.length === 0)
    throw new EvictionReportError('the notified caches reported no eviction',);
  /**
   Pairs the workload stored, rendered the way the recorder renders them.
   */
  const stored = new Set<string>();
  for (const item of SCENARIO_ITEMS)
    stored.add(renderPair(item,),);
  for (const key of DUPLICATE_KEYS)
    stored.add(renderPair({
      key,
      value: key,
    },),);
  for (const report of evicted) {
    if (!stored.has(report,))
      throw new EvictionReportError(`the cache reported evicting ${report}, which the workload never stored`,);
  }
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
  for (const key of DUPLICATE_KEYS) {
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
  assertEvictionsDescribeStoredPairs();
}

main();

//endregion Driver
