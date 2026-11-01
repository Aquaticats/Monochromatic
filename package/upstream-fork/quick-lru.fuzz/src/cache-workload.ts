/**
 Deterministic cache workload model and runner shared by this package's
 property files.
 
 A workload constructs a cache and applies generated operations with a
 controlled clock: storage and reads, bound changes, evictions, clears, and
 clock advances. Every operation's normalized result, the eviction events
 it fired, and a full state snapshot are recorded, so one cache fed one
 workload produces a directly comparable trace.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import type { FakeClock, } from '@monochromatic-dev/upstream-fork-quick-lru/ts/test-support.ts';

//region Types

/**
 One generated cache operation.
 
 `set` relies on the cache's global `maxAge`; `setWithMaxAge` overrides the
 lifetime per item exactly as upstream's `set(key, value, {maxAge})` does.
 */
export type CacheOp =
  | {
    /**
     Store a value under the global lifetime.
     */
    readonly kind: 'set';
    /**
     Key to store under.
     */
    readonly key: string;
    /**
     Value to store.
     */
    readonly value: string;
  }
    | {
    /**
     Store a value under a per-item lifetime.
     */
      readonly kind: 'setWithMaxAge';
    /**
     Key to store under.
     */
      readonly key: string;
    /**
     Value to store.
     */
      readonly value: string;
    /**
     Per-item lifetime in milliseconds, quirks included.
     */
      readonly maxAge: number;
    }
    | {
    /**
     Read a value, marking it most recently used.
     */
      readonly kind: 'get';
    /**
     Key to read.
     */
      readonly key: string;
    }
    | {
    /**
     Read a value without touching recency.
     */
      readonly kind: 'peek';
    /**
     Key to read.
     */
      readonly key: string;
    }
    | {
    /**
     Check presence without touching recency.
     */
      readonly kind: 'has';
    /**
     Key to check.
     */
      readonly key: string;
    }
    | {
    /**
     Remove one item without notifications.
     */
      readonly kind: 'delete';
    /**
     Key to remove.
     */
      readonly key: string;
    }
    | {
    /**
     Read one item's remaining lifetime without removing it.
     */
      readonly kind: 'expiresIn';
    /**
     Key to inspect.
     */
      readonly key: string;
    }
    | {
    /**
     Remove every item without notifications.
     */
      readonly kind: 'clear';
    }
    | {
    /**
     Update the target maximum in place.
     */
      readonly kind: 'resize';
    /**
     New target maximum, quirks included.
     */
      readonly maxSize: number;
    }
    | {
    /**
     Evict the least recently used items.
     */
      readonly kind: 'evict';
    /**
     Eviction count, quirks included.
     */
      readonly count: number;
    }
    | {
    /**
     Move the controlled clock forward.
     */
      readonly kind: 'advanceClock';
    /**
     Milliseconds to advance by.
     */
      readonly milliseconds: number;
    };

/**
 Fully generated workload: constructor bounds and operations.
 */
export type CacheWorkload = {
  /**
   Target maximum number of items.
   */
  readonly maxSize: number;
  /**
   Global lifetime in milliseconds; `Number.POSITIVE_INFINITY` stands in
   for upstream's absent `maxAge`, which resolves to the same bound.
   */
  readonly maxAge: number;
  /**
   Whether the cache carries an `onEviction` callback.
   */
  readonly onEviction: boolean;
  /**
   Operations to apply, in order.
   */
  readonly ops: readonly CacheOp[];
};

/**
 One cache's observable state, projected to strings so both
 implementations compare structurally.
 */
export type CacheSnapshot = {
  /**
   Reported item count.
   */
  readonly size: string;
  /**
   Reported target maximum.
   */
  readonly maxSize: string;
  /**
   Reported global lifetime.
   */
  readonly maxAge: string;
  /**
   `entriesAscending` pairs, oldest first.
   */
  readonly ascending: readonly string[];
  /**
   `entries` pairs; upstream aliases `entriesAscending` here.
   */
  readonly entries: readonly string[];
  /**
   `entriesDescending` pairs, newest first within each map.
   */
  readonly descending: readonly string[];
  /**
   Default-iterator pairs, recent map before old map.
   */
  readonly iterator: readonly string[];
  /**
   Live keys in default-iterator order.
   */
  readonly keys: readonly string[];
  /**
   Live values in default-iterator order.
   */
  readonly values: readonly string[];
  /**
   Old map's raw key and item projections, in map order.
   */
  readonly oldCache: readonly string[];
  /**
   `toString()` output.
   */
  readonly text: string;
};

/**
 Observable trace of one operation.
 */
export type StepTrace = {
  /**
   Operation label identifying its kind and target.
   */
  readonly opLabel: string;
  /**
   Normalized result: the operation's value projection or its thrown
   failure.
   */
  readonly result: string;
  /**
   Eviction events fired while the operation ran, in order.
   */
  readonly evictions: readonly string[];
  /**
   Full state snapshot taken right after the operation.
   */
  readonly snapshot: CacheSnapshot;
};

/**
 Observable trace of one workload run.
 */
export type WorkloadTrace = {
  /**
   Per-operation traces, in application order.
   */
  readonly steps: readonly StepTrace[];
};

/**
 One cache's workload surface, normalized so the same workload drives this
 package's fork and upstream `quick-lru`.
 */
export type CacheAdapter = {
  /**
   Stores one value, with an optional per-item lifetime.
   */
  readonly set: (options: {
    /**
     Key to store under.
     */
    readonly key: string;
    /**
     Value to store.
     */
    readonly value: string;
    /**
     Per-item lifetime override.
     */
    readonly maxAge?: number;
  },) => void;
  /**
   Reads one value, `absent` when the key is missing or expired.
   */
  readonly get: (key: string,) => string;
  /**
   Reads one value without touching recency, `absent` when missing or
   expired.
   */
  readonly peek: (key: string,) => string;
  /**
   Checks one key without touching recency.
   */
  readonly has: (key: string,) => boolean;
  /**
   Removes one item without notifications.
   */
  readonly remove: (key: string,) => boolean;
  /**
   Reads one item's remaining lifetime as `absent`, `infinite`, or
   `remaining:<milliseconds>`.
   */
  readonly expiresIn: (key: string,) => string;
  /**
   Removes every item without notifications.
   */
  readonly clear: () => void;
  /**
   Updates the target maximum in place.
   */
  readonly resize: (maxSize: number,) => void;
  /**
   Evicts the least recently used items.
   */
  readonly evict: (count: number,) => void;
  /**
   Projects the full observable state.
   */
  readonly snapshot: () => CacheSnapshot;
  /**
   Eviction events fired so far, in order, collected by the `onEviction`
   callback installed at construction.
   */
  readonly evictions: string[];
};

//endregion Types

//region Projection

/**
 Renders one key and value pair the way every ordering projection does.
 
 @param key - Pair key.
 
 @param value - Pair value.
 
 @returns `key=value` text.
 
 @example
 ```ts
 renderPair('a', 'b'); // => 'a=b'
 ```
 */
export function renderPair({
  key,
  value,
}: {
  /**
   Pair key.
   */
  readonly key: string;
  /**
   Pair value.
   */
  readonly value: string;
},): string {
  return `${key}=${value}`;
}

/**
 Renders one cached item for the old-map projection.
 
 @param key - Item key.
 
 @param value - Item value.
 
 @param expiry - Item expiry stamp as reported by the cache internals.
 
 @returns `key=value@expiry` text.
 
 @example
 ```ts
 renderItem('a', 'b', undefined,); // => 'a=b@none'
 ```
 */
export function renderItem({
  key,
  value,
  expiry,
}: {
  /**
   Item key.
   */
  readonly key: string;
  /**
   Item value.
   */
  readonly value: string;
  /**
   Item expiry stamp as reported by the cache internals.
   */
  readonly expiry: string;
},): string {
  return `${key}=${value}@${(expiry === 'undefined') ? 'none' : expiry}`;
}

/**
 Renders one thrown failure for comparison across implementations.
 
 Failures normalize on message text only: the fork's configuration errors
 name their own classes while keeping upstream's message text verbatim, so
 comparing names would flag the documented deviation instead of a real
 behavior change. Class shape is pinned by the runtime package's unit
 suites.
 
 @param reason - Failure observed by the runner.
 
 @returns Message of the failure, normalized for deep comparison.
 
 @mutates reason - Rendering a non-Error value runs JavaScript string
 conversion, which may invoke getters, proxies, `Symbol.toPrimitive`,
 `toString`, or `valueOf` through `caughtValueText`.
 
 @example
 ```ts
 describeThrown(new TypeError('bad',),);
 // => 'bad'
 ```
 */
export function describeThrown(reason: unknown,): string {
  if (Error.isError(reason,))
    return reason.message;

  return `ThrownValue: ${caughtValueText(reason,)}`;
}

/**
 Labels one operation for trace comparison.
 
 @param op - Operation under label.
 
 @returns Stable label naming the operation's kind and target.
 
 @example
 ```ts
 labelFor({ kind: 'get', key: 'a', },); // => 'get a'
 ```
 */
export function labelFor(op: CacheOp,): string {
  if ((op.kind === 'set') || (op.kind === 'setWithMaxAge'))
    return `set ${op.key}`;
  if ((op.kind === 'get') || (op.kind === 'peek')
    || (op.kind === 'has')
    || (op.kind === 'delete')
    || (op.kind === 'expiresIn'))
    return `${op.kind} ${op.key}`;
  if (op.kind === 'resize')
    return `resize ${String(op.maxSize,)}`;
  if (op.kind === 'evict')
    return `evict ${String(op.count,)}`;
  if (op.kind === 'advanceClock')
    return `advance ${String(op.milliseconds,)}`;
  return op.kind;
}

//endregion Projection

//region Runner

/**
 Applies one operation and projects its normalized result.
 
 @param adapter - Cache under test.
 
 @param op - Operation to apply.
 
 @param clock - Controlled clock the `advanceClock` operation moves.
 
 @returns Normalized result or thrown-failure text.
 
 @example
 ```ts
 applyOp(adapter, { kind: 'clear', }, clock,); // => 'void'
 ```
 */
export function applyOp({
  adapter,
  op,
  clock,
}: {
  /**
   Cache under test.
   */
  readonly adapter: CacheAdapter;
  /**
   Operation to apply.
   */
  readonly op: CacheOp;
  /**
   Controlled clock the `advanceClock` operation moves.
   */
  readonly clock: FakeClock;
},): string {
  try {
    if (op.kind === 'set') {
      adapter.set({
        key: op.key,
        value: op.value,
      },);
      return 'void';
    }
    if (op.kind === 'setWithMaxAge') {
      adapter.set({
        key: op.key,
        value: op.value,
        maxAge: op.maxAge,
      },);
      return 'void';
    }
    if (op.kind === 'get')
      return adapter.get(op.key,);
    if (op.kind === 'peek')
      return adapter.peek(op.key,);
    if (op.kind === 'has')
      return String(adapter.has(op.key,),);
    if (op.kind === 'delete')
      return String(adapter.remove(op.key,),);
    if (op.kind === 'expiresIn')
      return adapter.expiresIn(op.key,);
    if (op.kind === 'clear') {
      adapter.clear();
      return 'void';
    }
    if (op.kind === 'resize') {
      adapter.resize(op.maxSize,);
      return 'void';
    }
    if (op.kind === 'evict') {
      adapter.evict(op.count,);
      return 'void';
    }
    clock.advance(op.milliseconds,);
    return 'void';
  }
  catch (error) {
    return `threw ${describeThrown(error,)}`;
  }
}

/**
 Runs one workload against one cache adapter and records its trace.
 
 The clock is rewound to the workload's start instant first, so two
 adapters fed one workload observe identical time.
 
 @param options - Adapter, workload, clock, and starting instant.
 
 @returns Per-operation traces in application order.
 
 @example
 ```ts
 const trace = runWorkload({
   adapter,
   workload,
   clock,
   startMilliseconds: 1_000,
 },);
 ```
 */
export function runWorkload(options: {
  /**
   Cache under test.
   */
  readonly adapter: CacheAdapter;
  /**
   Generated constructor bounds and operations.
   */
  readonly workload: CacheWorkload;
  /**
   Controlled clock shared by both adapters.
   */
  readonly clock: FakeClock;
  /**
   Instant every workload run starts from.
   */
  readonly startMilliseconds: number;
},): WorkloadTrace {
  options.clock
    .advance(options.startMilliseconds
      - options.clock
        .now,);
  /**
   Per-operation traces collected below.
   */
  const steps: StepTrace[] = [];
  /**
   How many eviction events the previous steps already recorded.
   */
  const seen = {
    count: 0,
  };
  for (const op of options.workload
    .ops) {
    /**
     Result of the operation, captured before the snapshot is taken.
     */
    const result = applyOp({
      adapter: options.adapter,
      op,
      clock: options.clock,
    },);
    /**
     Eviction events fired during this operation, in order.
     */
    const evictions = options.adapter
      .evictions
      .slice(seen.count,);
    seen.count = options.adapter
      .evictions
      .length;
    steps.push({
      opLabel: labelFor(op,),
      result,
      evictions,
      snapshot: options.adapter
        .snapshot(),
    },);
  }

  return {
    steps,
  };
}

//endregion Runner
