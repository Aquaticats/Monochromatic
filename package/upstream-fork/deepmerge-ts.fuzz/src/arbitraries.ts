/**
 fast-check generators for merge inputs.

 Trees only: every container is freshly built and appears once, so no input
 contains a cycle or a shared container. Shared containers reach a known
 upstream defect (false cycle detection) that `src/known-defect.unit.test.ts`
 pins separately; excluding them here keeps the model comparison meaningful.

 A small key pool makes inputs overlap, so merges actually combine values
 instead of mostly unioning disjoint keys. The pool includes prototype-named
 keys (`__proto__`, `constructor`, `prototype`) and symbols, and records can
 carry null prototypes, accessors, non-enumerable keys, and a frozen state;
 `exotic: false` turns those off for `deepmergeInto` targets, which upstream
 writes into and which therefore must be writable plain data.

 @module
 */

import {
  array,
  bigInt,
  boolean,
  constant,
  constantFrom,
  date,
  double,
  integer,
  letrec,
  oneof,
  record,
  string,
  tuple,
  uniqueArray,
  type Arbitrary,
} from 'fast-check';

//region Leaves

/**
 Leaf object shaped like a class instance; merges must treat it as opaque.
 */
export type Point = {
  /**
   Coordinate; merge results compare points by identity, not by this.
   */
  readonly x: number;
};

/**
 Constructor-shaped marker giving points a class-like prototype: its
 `prototype` owns `constructor` but not `isPrototypeOf`, which is exactly
 what deepmerge-ts's `isRecord` checks to reject class instances.
 */
function PointMarker(): void {
  // Never called: only its automatically created `prototype` object is used.
}

/**
 Shared prototype of every point.
 */
const POINT_PROTOTYPE: unknown = Reflect.get(
  PointMarker,
  'prototype',
);

/**
 Build a point: an object with a class-like prototype, the stand-in for a
 class instance since this repo bans classes outside errors.

 @param x - Coordinate stored on the point.

 @returns Fresh point.

 @throws When the marker's prototype is missing, which no engine does.

 @example
 ```ts
 point(1); // PointMarker { x: 1 }
 ```
 */
export function point(x: number,): Point {
  if (((typeof POINT_PROTOTYPE) !== 'object') || (POINT_PROTOTYPE === null))
    throw new Error('point: PointMarker has no prototype object',);
  /**
   Point data, given the marker prototype below.
   */
  const created: Point = { x, };
  Reflect.setPrototypeOf(
    created,
    POINT_PROTOTYPE,
  );
  return created;
}

/**
 Symbol keys and values; `Symbol.for` exercises registered symbols too.
 */
export const SYMBOL_POOL: readonly [
  symbol,
  symbol,
] = [
  Symbol('first pooled test symbol',),
  Symbol.for('deepmerge-ts.fuzz.beta',),
];

/**
 Object used only as a Map key, never as a value, so it cannot form a shared
 container.
 */
const OBJECT_MAP_KEY = Object.freeze({ mapKey: true, },);

/**
 Record keys: overlapping names, integer-like names, prototype-named keys
 other than `__proto__`, and symbols.
 */
const SAFE_RECORD_KEYS: readonly PropertyKey[] = [
  'a',
  'b',
  'c',
  '0',
  '1',
  'constructor',
  'prototype',
  'toString',
  ...SYMBOL_POOL,
];

/**
 Record key generator.

 @param protoKey - Whether `__proto__` joins the pool; off only for the
   `FastUnsafe` variants, which document that they do not guard it.

 @returns Key generator over the pool.

 @example
 ```ts
 const keys = recordKeyArbitrary({ protoKey: true, });
 ```
 */
function recordKeyArbitrary({ protoKey, }: { readonly protoKey: boolean; },): Arbitrary<PropertyKey> {
  return constantFrom<PropertyKey>(
    ...SAFE_RECORD_KEYS,
    ...(protoKey ? ['__proto__',] : []),
  );
}

/**
 Map keys: primitives that collide under SameValueZero and one object key.
 */
const mapKeyArbitrary: Arbitrary<unknown> = constantFrom<unknown>(
  'a',
  'b',
  1,
  2,
  Number.NaN,
  OBJECT_MAP_KEY,
);

/**
 Leaf generator.

 @param objectLeaves - Whether dates and class instances join the leaves; off
   only where a known defect (`src/known-defect.unit.test.ts`) makes them diverge.
 
 @param undefinedLeaves - Whether `undefined` joins the leaves; off only for
   `deepmergeInto` targets, for the same reason.

 @returns Generator of every non-container value the merge must treat as a leaf.

 @example
 ```ts
 const leaves = leafArbitrary({ objectLeaves: true, undefinedLeaves: true, });
 ```
 */
export function leafArbitrary(
  {
    objectLeaves,
    undefinedLeaves,
  }: {
    readonly objectLeaves: boolean;
    readonly undefinedLeaves: boolean
  },
): Arbitrary<unknown> {
  return oneof(
    ...(undefinedLeaves ? [constant(undefined,),] : []),
    constant(null,),
    boolean(),
    integer({
      min: -3,
      max: 3,
    },),
    double(),
    string({ maxLength: 3, },),
    bigInt({
      min: -5n,
      max: 5n,
    },),
    constantFrom(...SYMBOL_POOL,),
    ...(objectLeaves ? [
      date(),
      integer()
        .map(function toPoint(x,) {
          return point(x,);
        },),
    ] : []),
  );
}

//endregion Leaves

//region Records

/**
 How one generated record property is defined.
 */
type EntryStyle = 'data' | 'getter' | 'hidden';

/**
 Record shape before it is materialized.
 */
type RecordPlan = {
  readonly entries: readonly (readonly [
    PropertyKey,
    unknown,
    EntryStyle,
  ])[];
  readonly frozen: boolean;
  readonly nullPrototype: boolean;
};

/**
 Materialize a record plan with `defineProperty`, so `__proto__` becomes an
 own key instead of invoking the prototype setter.

 @param plan - Entries, prototype choice, and frozen flag.

 @returns Fresh record matching the plan.

 @example
 ```ts
 buildRecord({ entries: [['a', 1, 'data',],], frozen: false, nullPrototype: false, });
 ```
 */
function buildRecord(plan: RecordPlan,): object {
  /**
   Fresh record with the planned prototype.
   */
  const built: object = {};
  if (plan.nullPrototype) {
    Reflect.setPrototypeOf(
      built,
      null,
    );
  }
  for (const [key, value, style,] of plan.entries) {
    if (style === 'getter') {
      Reflect.defineProperty(
        built,
        key,
        {
          configurable: true,
          enumerable: true,
          get: function readPlanned() {
            return value;
          },
        },
      );
    } else {
      Reflect.defineProperty(
        built,
        key,
        {
          configurable: true,
          enumerable: style === 'data',
          value,
          writable: true,
        },
      );
    }
  }
  return plan.frozen ? Object.freeze(built,) : built;
}

//endregion Records

//region Trees

/**
 Uniqueness selector for generated entries: the key, their first element.

 @param entry - Generated key-first tuple.

 @returns Key that must be unique within one container.

 @example
 ```ts
 keyOfEntry(['a', 1,]); // 'a'
 ```
 */
export function keyOfEntry<const TKey,>(entry: readonly [
  TKey,
  ...unknown[],
],): TKey {
  return entry[0];
}

/**
 Maximum container nesting of one generated tree; deep enough for the
 recursive paths, far below every depth limit.
 */
const TREE_MAX_DEPTH = 4;

/**
 Maximum entries per generated container.
 */
export const CONTAINER_MAX_LENGTH = 4;

/**
 Switches shaping generated trees; each defaults to on except `exotic`,
 which every caller chooses explicitly.
 */
export type TreeOptions = {
  /**
   Whether records may carry accessors, non-enumerable keys, null prototypes,
   and a frozen state.
   */
  readonly exotic: boolean;
  /**
   Whether non-container objects (dates, class instances) appear as leaves.
   */
  readonly objectLeaves?: boolean;
  /**
   Whether records may use the `__proto__` key.
   */
  readonly protoKey?: boolean;
  /**
   Whether `undefined` appears as a leaf.
   */
  readonly undefinedLeaves?: boolean;
};

/**
 Recursive generators sharing one `letrec` scope.
 */
export type TreeArbitraries = {
  readonly record: Arbitrary<object>;
  readonly tree: Arbitrary<unknown>;
};

/**
 Build the recursive tree and record generators.

 @param exotic - See {@link TreeOptions}.
 
 @param objectLeaves - See {@link TreeOptions}.
 
 @param protoKey - See {@link TreeOptions}.
 
 @param undefinedLeaves - See {@link TreeOptions}.

 @returns Tree generator plus a record-only generator over the same scope.

 @example
 ```ts
 const { tree, record } = treeArbitraries({ exotic: true, });
 ```
 */
export function treeArbitraries({
  exotic,
  objectLeaves = true,
  protoKey = true,
  undefinedLeaves = true,
}: TreeOptions,): TreeArbitraries {
  /**
   Entry style generator; plain data only when `exotic` is off.
   */
  const style: Arbitrary<EntryStyle> = exotic
    ? oneof(
      {
        weight: 6,
        arbitrary: constant<EntryStyle>('data',),
      },
      {
        weight: 1,
        arbitrary: constant<EntryStyle>('getter',),
      },
      {
        weight: 1,
        arbitrary: constant<EntryStyle>('hidden',),
      },
    )
    : constant<EntryStyle>('data',);
  /**
   Flag generator that stays false when `exotic` is off.
   */
  const flag = exotic ? boolean() : constant(false,);
  /**
   Mutually recursive generators; `tree` bounds the depth.
   */
  const scope = letrec<{
    tree: unknown;
    container: unknown;
    record: object
  }>(function scopeOf(tie,) {
    return {
      tree: oneof(
        {
          maxDepth: TREE_MAX_DEPTH,
          depthIdentifier: 'tree',
        },
        leafArbitrary({
          objectLeaves,
          undefinedLeaves,
        },),
        tie('container',),
      ),
      container: oneof(
        { depthIdentifier: 'tree', },
        tie('record',),
        array(
          tie('tree',),
          { maxLength: CONTAINER_MAX_LENGTH, },
        ),
        array(
          tie('tree',),
          { maxLength: CONTAINER_MAX_LENGTH, },
        )
          .map(function toSet(items,) {
            return new Set(items,);
          },),
        uniqueArray(
          tuple(
            mapKeyArbitrary,
            tie('tree',),
          ),
          {
            comparator: 'SameValueZero',
            maxLength: CONTAINER_MAX_LENGTH,
            selector: keyOfEntry,
          },
        )
          .map(function toMap(entries,) {
            return new Map(entries,);
          },),
      ),
      record: record({
        entries: uniqueArray(
          tuple(
            recordKeyArbitrary({ protoKey, },),
            tie('tree',),
            style,
          ),
          {
            maxLength: CONTAINER_MAX_LENGTH,
            selector: keyOfEntry,
          },
        ),
        frozen: flag,
        nullPrototype: flag,
      },)
        .map(function materialize(plan,) {
          return buildRecord(plan,);
        },),
    };
  },);
  return {
    record: scope.record,
    tree: scope.tree,
  };
}

//endregion Trees
