/**
 Deterministic operation-sequence model shared by this package's property
 files.
 
 A sequence lists store operations over one vocabulary
 (single and multi `set`,
 `get`,
 `has`,
 `delete`,
 `appendToArray`,
 `reset`,
 `clear`,
 and whole-store replacement)
 carrying JSON-safe values,
 so the same sequence can drive the vendored upstream `conf` snapshot and the fork through
 identical call shapes and produce directly comparable observations.
 
 @module
 */

import {
  type Arbitrary,
  array,
  boolean,
  constant,
  constantFrom,
  integer,
  oneof,
  string,
  tuple,
} from 'fast-check';

//region Values

/**
 JSON-safe value a store operation may carry:
 exactly the values `JSON` round-trips without loss.
 */
export type JsonValue =
  | null
    | boolean
    | number
    | string
    | readonly JsonValue[]
    | JsonRecord;

/**
 JSON-safe object with string keys.
 */
export type JsonRecord = {
  readonly [key: string]: JsonValue;
};

//endregion Values

//region Operations

/**
 One store operation over the shared vocabulary.
 */
export type StoreOperation =
  | {
    /**
     Single-item `set`.
     */
    readonly kind: 'setOne';
    /**
     Key or dotted path to place the value under.
     */
    readonly key: string;
    /**
     JSON-safe value to place.
     */
    readonly value: JsonValue;
  }
    | {
    /**
     Multi-item `set` over a values record.
     */
      readonly kind: 'setMany';
    /**
     Key-value entries to place at once.
     */
      readonly entries: readonly (readonly [
        string,
        JsonValue
      ])[];
    }
    | {
    /**
     `get`, optionally with a caller-supplied default.
     */
      readonly kind: 'get';
    /**
     Key or dotted path to read.
     */
      readonly key: string;
    /**
     Whether the call supplies a default value.
     */
      readonly useDefault: boolean;
    /**
     Default value supplied when `useDefault` is set.
     */
      readonly defaultValue: JsonValue;
    }
    | {
    /**
     `has`.
     */
      readonly kind: 'has';
    /**
     Key or dotted path to probe.
     */
      readonly key: string;
    }
    | {
    /**
     `delete`.
     */
      readonly kind: 'delete';
    /**
     Key or dotted path to remove.
     */
      readonly key: string;
    }
    | {
    /**
     `appendToArray`.
     */
      readonly kind: 'appendToArray';
    /**
     Key or dotted path of the array to grow.
     */
      readonly key: string;
    /**
     Item to append.
     */
      readonly value: JsonValue;
    }
    | {
    /**
     `reset` over a key list.
     */
      readonly kind: 'reset';
    /**
     Keys or dotted paths to restore to their defaults.
     */
      readonly keys: readonly string[];
    }
    | {
    /**
     `clear`.
     */
      readonly kind: 'clear';
    }
    | {
    /**
     Whole-store replacement through the `store` setter.
     */
      readonly kind: 'replaceStore';
    /**
     Replacement store contents.
     */
      readonly value: JsonRecord;
    };

/**
 Fully generated workload:
 one operation sequence applied to a freshly constructed store.
 */
export type StoreOperationSequence = {
  /**
   Operations to run in order.
   */
  readonly operations: readonly StoreOperation[];
};

//endregion Operations

//region Fixtures

/**
 Smallest generated integer value.
 */
const INTEGER_MIN = -1_000;

/**
 Largest generated integer value.
 */
const INTEGER_MAX = 1_000;

/**
 Longest generated free-form string.
 */
const STRING_MAX_LENGTH = 6;

/**
 Most entries generated for one values record or array value.
 */
const CONTAINER_MAX_ENTRIES = 3;

/**
 Most operations generated for one sequence.
 */
const SEQUENCE_MAX_OPERATIONS = 8;

/**
 Defaults every store under comparison is constructed with:
 plain top-level keys only,
 so `reset` and `clear` restore exact values in both implementations.
 */
export const STORE_DEFAULTS: JsonRecord = {
  theme: 'light',
  retries: 3,
  items: [],
  nested: {
    flag: true,
  },
};

/**
 Key pool covering plain keys,
 dotted paths,
 missing keys,
 reserved-key attempts,
 prototype-shaped names,
 empty and malformed paths,
 and keys needing `JSON` escaping.
 */
const KEY_POOL: readonly string[] = [
  'theme',
  'retries',
  'items',
  'nested',
  'nested.flag',
  'a',
  'b',
  'a.b',
  'a.b.c',
  'missing',
  '',
  '__proto__',
  'constructor',
  'prototype',
  '__internal__',
  '__internal__.x',
  'x.__internal__',
  '.x',
  'x.',
  'x..y',
  'key with space',
  'ü',
  'quote"d',
  String.raw`back\slash`,
  'line\nbreak',
];

/**
 Scalar-value pool covering values needing `JSON` escaping and edge strings.
 */
const SCALAR_STRING_POOL: readonly string[] = [
  '',
  'a',
  'theme',
  'items',
  'null',
  '0',
  '-1',
  '__proto__',
  '__internal__',
  'ü',
  'quote"',
  'line\nbreak',
  ' tab\t',
];

//endregion Fixtures

//region Arbitraries

/**
 Generated key or dotted path:
 pool entries plus short free-form strings.
 */
export const keyArb: Arbitrary<string> = oneof(
  constantFrom(...KEY_POOL,),
  string({
    maxLength: STRING_MAX_LENGTH,
  },),
);

/**
 Generated scalar JSON value:
 null,
 booleans,
 bounded integers,
 pool strings,
 and short free-form strings.
 */
const scalarValueArb: Arbitrary<JsonValue> = oneof(
  constant(null,),
  boolean(),
  integer({
    min: INTEGER_MIN,
    max: INTEGER_MAX,
  },),
  constantFrom(...SCALAR_STRING_POOL,),
  string({
    maxLength: STRING_MAX_LENGTH,
  },),
);

/**
 Builds an object arbitrary whose keys come from the shared key pool.
 
 Values are built through `Object.fromEntries`,
 so generated `__proto__` keys stay own data properties instead of reaching
 the prototype setter.
 
 @param valueArb - Arbitrary generating each entry's value.
 
 @returns Arbitrary generating JSON-safe objects.
 
 @example
 ```ts
 const arb = recordOf(scalarValueArb);
 ```
 */
function recordOf(valueArb: Arbitrary<JsonValue>,): Arbitrary<JsonRecord> {
  return array(
    tuple(
      keyArb,
      valueArb,
    ),
    {
      maxLength: CONTAINER_MAX_ENTRIES,
    },
  )
    .map(function toRecord(entries: readonly (readonly [
      string,
      JsonValue
    ])[],): JsonRecord {
      return Object.fromEntries(entries,);
    },);
}

/**
 Generated value nested one level below the top:
 scalars,
 flat arrays,
 and flat objects.
 */
const nestedValueArb: Arbitrary<JsonValue> = oneof(
  scalarValueArb,
  array(
    scalarValueArb,
    {
      maxLength: CONTAINER_MAX_ENTRIES,
    },
  ),
  recordOf(scalarValueArb,),
);

/**
 Generated JSON-safe value:
 nested up to two container levels.
 */
export const jsonValueArb: Arbitrary<JsonValue> = oneof(
  scalarValueArb,
  array(
    nestedValueArb,
    {
      maxLength: CONTAINER_MAX_ENTRIES,
    },
  ),
  recordOf(nestedValueArb,),
);

/**
 Generated JSON-safe object,
 used for whole-store replacements and round-trip fixtures.
 */
export const jsonRecordArb: Arbitrary<JsonRecord> = recordOf(jsonValueArb,);

//endregion Arbitraries

//region Sequence

/**
 Generated store operation:
 one uniform pick over the whole operation vocabulary.
 */
export const operationArb: Arbitrary<StoreOperation> = oneof(
  tuple(
    keyArb,
    jsonValueArb,
  )
    .map(function toSetOne([key, value,]: readonly [
      string,
      JsonValue
    ],): StoreOperation {
      return {
        kind: 'setOne',
        key,
        value,
      };
    },),
  array(
    tuple(
      keyArb,
      jsonValueArb,
    ),
    {
      maxLength: CONTAINER_MAX_ENTRIES,
    },
  )
    .map(function toSetMany(entries: readonly (readonly [
      string,
      JsonValue
    ])[],): StoreOperation {
      return {
        kind: 'setMany',
        entries,
      };
    },),
  tuple(
    keyArb,
    boolean(),
    jsonValueArb,
  )
    .map(function toGet([key, useDefault, defaultValue,]: readonly [
      string,
      boolean,
      JsonValue
    ],): StoreOperation {
      return {
        kind: 'get',
        key,
        useDefault,
        defaultValue,
      };
    },),
  keyArb.map(function toHas(key: string,): StoreOperation {
    return {
      kind: 'has',
      key,
    };
  },),
  keyArb.map(function toDelete(key: string,): StoreOperation {
    return {
      kind: 'delete',
      key,
    };
  },),
  tuple(
    keyArb,
    jsonValueArb,
  )
    .map(function toAppend([key, value,]: readonly [
      string,
      JsonValue
    ],): StoreOperation {
      return {
        kind: 'appendToArray',
        key,
        value,
      };
    },),
  array(
    keyArb,
    {
      maxLength: CONTAINER_MAX_ENTRIES,
    },
  )
    .map(function toReset(keys: readonly string[],): StoreOperation {
      return {
        kind: 'reset',
        keys,
      };
    },),
  constant<StoreOperation>({
    kind: 'clear',
  },),
  jsonRecordArb.map(function toReplaceStore(value: JsonRecord,): StoreOperation {
    return {
      kind: 'replaceStore',
      value,
    };
  },),
);

/**
 Generated operation sequence:
 one to {@link SEQUENCE_MAX_OPERATIONS} operations over the shared vocabulary.
 */
export const sequenceArb: Arbitrary<StoreOperationSequence> = array(
  operationArb,
  {
    minLength: 1,
    maxLength: SEQUENCE_MAX_OPERATIONS,
  },
)
  .map(function toSequence(operations: readonly StoreOperation[],): StoreOperationSequence {
    return {
      operations,
    };
  },);

//endregion Sequence
