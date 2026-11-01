/**
 Store adapters exposing one uniform operation surface over this package's
 fork and upstream `conf`,
 so the same generated operation sequence can drive both.
 
 Each adapter constructs its store in a disposable temp directory,
 runs the sequence,
 and returns one observation record:
 per-operation results
 (read values,
 probes,
 thrown error names and message texts),
 final store contents,
 item count,
 and the config file's parsed `JSON`.
 
 @module
 */

import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import Conf from '../upstream-conf/index.ts';

import {
  createConf,
} from '@monochromatic-dev/upstream-fork-conf/ts';

import {
  type JsonRecord,
  type JsonValue,
  type StoreOperation,
  type StoreOperationSequence,
  STORE_DEFAULTS,
} from './store-operations.ts';

//region Types

/**
 Base error class observed from a thrown value:
 `TypeError`,
 plain `Error`,
 or a thrown non-`Error`.
 */
export type ErrorBase = 'TypeError' | 'Error' | 'ThrownValue';

/**
 Result of one store operation.
 */
export type OperationOutcome =
  | {
    /**
     Operation completed without throwing.
     */
    readonly kind: 'ok';
  }
    | {
    /**
     `get` result:
     `value` is `undefined` exactly when `found` is `false`.
     */
      readonly kind: 'value';
    /**
     Whether the key resolved to a stored value.
     */
      readonly found: boolean;
    /**
     Value read,
     normalized to plain `JSON`.
     */
      readonly value: JsonValue | undefined;
    }
    | {
    /**
     `has` result.
     */
      readonly kind: 'boolean';
    /**
     Probe answer.
     */
      readonly value: boolean;
    }
    | {
    /**
     Operation threw.
     */
      readonly kind: 'threw';
    /**
     Thrown error's `name`.
     */
      readonly errorName: string;
    /**
     Thrown error's base class.
     */
      readonly errorBase: ErrorBase;
    /**
     Thrown error's `message`.
     */
      readonly errorMessage: string;
    };

/**
 Observable record of one operation-sequence run.
 */
export type StoreObservation = {
  /**
   Per-operation results,
   in sequence order.
   */
  readonly outcomes: readonly OperationOutcome[];
  /**
   Whole store after the sequence,
   normalized to plain `JSON`.
   */
  readonly finalStore: JsonRecord;
  /**
   Item count reported by the store after the sequence.
   */
  readonly size: number;
  /**
   Parsed config-file contents after the sequence,
   or `'absent'` when no file was written.
   */
  readonly fileContents: JsonRecord | 'absent';
};

/**
 Minimal store surface the runner drives:
 one implementation adapter per store under comparison.
 */
type StoreSurface = {
  /**
   Path of the config file this store persists to.
   */
  readonly path: string;
  /**
   Reads one key,
   with an optional default for missing keys.
   */
  readonly get: (input: {
    readonly key: string;
    readonly useDefault: boolean;
    readonly defaultValue: JsonValue;
  },) => JsonValue | undefined;
  /**
   Places one keyed value.
   */
  readonly setOne: (input: {
    readonly key: string;
    readonly value: JsonValue;
  },) => void;
  /**
   Places many keyed values at once.
   */
  readonly setMany: (input: {
    readonly entries: readonly (readonly [
      string,
      JsonValue
    ])[];
  },) => void;
  /**
   Probes one key.
   */
  readonly has: (key: string,) => boolean;
  /**
   Removes one key.
   */
  readonly delete: (key: string,) => void;
  /**
   Appends one item to one key's array value.
   */
  readonly appendToArray: (input: {
    readonly key: string;
    readonly value: JsonValue;
  },) => void;
  /**
   Restores listed keys to their defaults.
   */
  readonly reset: (input: {
    readonly keys: readonly string[];
  },) => void;
  /**
   Restores the whole store to its defaults.
   */
  readonly clear: () => void;
  /**
   Replaces the whole store.
   */
  readonly replaceStore: (input: {
    readonly value: JsonRecord;
  },) => void;
  /**
   Reads the whole store.
   */
  readonly readStore: () => JsonRecord;
  /**
   Reads the item count.
   */
  readonly readSize: () => number;
};

//endregion Types

//region Fixtures

/**
 Prefix of every disposable store directory this module creates.
 */
const TEMP_DIR_PREFIX = 'conf-fork-fuzz-';

/**
 Marker reported instead of file contents when no config file exists.
 */
const ABSENT_FILE = 'absent' as const;

//endregion Fixtures

//region Helpers

/**
 Creates a disposable temp directory for one store run.
 
 @returns Handle carrying the directory path and its removal hook;
 consume it with `using` so the directory is removed however the run ends.
 
 @example
 ```ts
 using dir = createTempStoreDir();
 ```
 */
export function createTempStoreDir(): {
  readonly path: string;
  [Symbol.dispose]: () => void;
} {
  /**
   Absolute path of the created directory.
   */
  const path = mkdtempSync(join(
    tmpdir(),
    TEMP_DIR_PREFIX,
  ),);
  return {
    path,
    [Symbol.dispose]: function removeTempStoreDir(): void {
      rmSync(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Normalizes a `JSON`-safe value to plain `JSON`,
 dropping prototypes and shared references so cross-implementation
 comparison is shape-exact.
 
 @param value - Value read from a store.
 
 @returns Plain-`JSON` copy of `value`.
 
 @example
 ```ts
 jsonClone([1, { a: 2 },]); // => [1, { a: 2 }]
 ```
 */
export function jsonClone(value: JsonValue,): JsonValue {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON round-tripping a JSON-safe value yields the same JSON-safe shape
  return JSON.parse(JSON.stringify(value,),) as JsonValue;
}

/**
 Renders a thrown value for cross-implementation comparison:
 name,
 base class,
 and message text.
 
 @param error - Value the operation threw.
 
 @throws Error when a thrown non-`Error` cannot be stringified.
 
 @returns Name,
 base class,
 and message of the thrown value.
 
 @example
 ```ts
 describeThrown(new TypeError('bad',),);
 // => { errorName: 'TypeError', errorBase: 'TypeError', errorMessage: 'bad' }
 ```
 */
function describeThrown(error: unknown,): {
  readonly errorName: string;
  readonly errorBase: ErrorBase;
  readonly errorMessage: string;
} {
  if (Error.isError(error,))
    return {
      errorName: error.name,
      errorBase: (error instanceof TypeError)
        ? 'TypeError'
        : 'Error',
      errorMessage: error.message,
    };

  return {
    errorName: 'ThrownValue',
    errorBase: 'ThrownValue',
    errorMessage: String(error,),
  };
}

/**
 Converts one operation's entries to the values record both multi-`set`
 call shapes accept.
 
 @param entries - Key-value entries generated for the multi-`set`.
 
 @returns Record built through `Object.fromEntries`,
 keeping generated `__proto__` keys own.
 
 @example
 ```ts
 entriesRecord([['a', 1],]); // => { a: 1 }
 ```
 */
function entriesRecord(entries: readonly (readonly [
  string,
  JsonValue
])[],): Record<string, JsonValue> {
  return Object.fromEntries(entries,);
}

//endregion Helpers

//region Runner

/**
 Runs one operation against one store surface and records its outcome.
 
 @param surface - Store surface under test.
 
 @param operation - Generated operation to run.
 
 @returns Outcome of the operation:
 read value,
 probe answer,
 or thrown error description.
 
 @example
 ```ts
 const outcome = observeOperation({ surface, operation, });
 ```
 */
function observeOperation(
  {
    surface,
    operation,
  }: {
    readonly surface: StoreSurface;
    readonly operation: StoreOperation;
  },
): OperationOutcome {
  try {
    if (operation.kind === 'setOne') {
      surface.setOne({
        key: operation.key,
        value: operation.value,
      },);
      return {
        kind: 'ok',
      };
    }

    if (operation.kind === 'setMany') {
      surface.setMany({
        entries: operation.entries,
      },);
      return {
        kind: 'ok',
      };
    }

    if (operation.kind === 'get') {
      /**
       Value the key resolved to,
       or `undefined` for missing keys.
       */
      const value = surface.get({
        key: operation.key,
        useDefault: operation.useDefault,
        defaultValue: operation.defaultValue,
      },);
      /**
       Whether the key resolved to a stored value.
       */
      const found = value !== undefined;
      return {
        kind: 'value',
        found,
        value: found
          ? jsonClone(value,)
          : undefined,
      };
    }

    if (operation.kind === 'has')
      return {
        kind: 'boolean',
        value: surface.has(operation.key,),
      };

    if (operation.kind === 'delete') {
      surface.delete(operation.key,);
      return {
        kind: 'ok',
      };
    }

    if (operation.kind === 'appendToArray') {
      surface.appendToArray({
        key: operation.key,
        value: operation.value,
      },);
      return {
        kind: 'ok',
      };
    }

    if (operation.kind === 'reset') {
      surface.reset({
        keys: operation.keys,
      },);
      return {
        kind: 'ok',
      };
    }

    if (operation.kind === 'clear') {
      surface.clear();
      return {
        kind: 'ok',
      };
    }

    surface.replaceStore({
      value: operation.value,
    },);
    return {
      kind: 'ok',
    };
  }
  catch (error) {
    return {
      kind: 'threw',
      ...describeThrown(error,),
    };
  }
}

/**
 Runs one operation sequence against one store surface and records the
 observation.
 
 @param surface - Store surface under test.
 
 @param sequence - Generated operations to run in order.
 
 @returns Observation of per-operation outcomes,
 final store,
 item count,
 and file contents.
 
 @example
 ```ts
 const observation = runSequence({ surface, sequence, });
 ```
 */
function runSequence(
  {
    surface,
    sequence,
  }: {
    readonly surface: StoreSurface;
    readonly sequence: StoreOperationSequence;
  },
): StoreObservation {
  /**
   Per-operation outcomes,
   in sequence order.
   */
  const outcomes = sequence.operations
    .map(function observeOne(operation: StoreOperation,): OperationOutcome {
      return observeOperation({
        surface,
        operation,
      },);
    },);
  /**
   Parsed config-file contents after the sequence,
   or the absent marker.
   */
  const fileContents: JsonRecord | 'absent' = existsSync(surface.path,)
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the config file always holds JSON-object contents in these runs
    ? JSON.parse(readFileSync(
      surface.path,
      'utf8',
    ),) as JsonRecord
    : ABSENT_FILE;

  return {
    outcomes,
    finalStore: {
      ...surface.readStore(),
    },
    size: surface.readSize(),
    fileContents,
  };
}

//endregion Runner

//region Adapters

/**
 Runs one operation sequence against this package's fork.
 
 @param sequence - Generated operations to run in order.
 
 @returns Observation of the fork's behavior.
 
 @example
 ```ts
 const observation = runOnFork({ operations: [], });
 ```
 */
export function runOnFork(sequence: StoreOperationSequence,): StoreObservation {
  using dir = createTempStoreDir();
  /**
   Fork store under test.
   */
  const config = createConf<Record<string, JsonValue>>({
    cwd: dir.path,
    defaults: STORE_DEFAULTS,
  });

  return runSequence({
    surface: {
      path: config.path,
      get: function forkGet(input: {
        readonly key: string;
        readonly useDefault: boolean;
        readonly defaultValue: JsonValue;
      },): JsonValue | undefined {
        return input.useDefault
          ? config.get({
            key: input.key,
            defaultValue: input.defaultValue,
          },)
          : config.get(input.key,);
      },
      setOne: function forkSetOne(input: {
        readonly key: string;
        readonly value: JsonValue;
      },): void {
        config.set(input,);
      },
      setMany: function forkSetMany(input: {
        readonly entries: readonly (readonly [
          string,
          JsonValue
        ])[];
      },): void {
        config.set({
          values: entriesRecord(input.entries,),
        },);
      },
      has: function forkHas(key: string,): boolean {
        return config.has(key,);
      },
      delete: function forkDelete(key: string,): void {
        config.delete(key,);
      },
      appendToArray: function forkAppend(input: {
        readonly key: string;
        readonly value: JsonValue;
      },): void {
        config.appendToArray(input,);
      },
      reset: function forkReset(input: {
        readonly keys: readonly string[];
      },): void {
        config.reset(input,);
      },
      clear: function forkClear(): void {
        config.clear();
      },
      replaceStore: function forkReplaceStore(input: {
        readonly value: JsonRecord;
      },): void {
        config.store = input.value;
      },
      readStore: function forkReadStore(): JsonRecord {
        return config.store;
      },
      readSize: function forkReadSize(): number {
        return config.size;
      },
    },
    sequence,
  });
}

/**
 Runs one operation sequence against the vendored upstream `conf` snapshot.
 
 @param sequence - Generated operations to run in order.
 
 @returns Observation of upstream's behavior.
 
 @example
 ```ts
 const observation = runOnUpstream({ operations: [], });
 ```
 */
export function runOnUpstream(sequence: StoreOperationSequence,): StoreObservation {
  using dir = createTempStoreDir();
  /**
   Upstream store under comparison.
   */
  const config = new Conf<Record<string, JsonValue>>({
    cwd: dir.path,
    defaults: STORE_DEFAULTS,
  });

  return runSequence({
    surface: {
      path: config.path,
      get: function upstreamGet(input: {
        readonly key: string;
        readonly useDefault: boolean;
        readonly defaultValue: JsonValue;
      },): JsonValue | undefined {
        return input.useDefault
          ? config.get(
            input.key,
            input.defaultValue,
          )
          : config.get(input.key,);
      },
      setOne: function upstreamSetOne(input: {
        readonly key: string;
        readonly value: JsonValue;
      },): void {
        config.set(
          input.key,
          input.value,
        );
      },
      setMany: function upstreamSetMany(input: {
        readonly entries: readonly (readonly [
          string,
          JsonValue
        ])[];
      },): void {
        config.set(entriesRecord(input.entries,),);
      },
      has: function upstreamHas(key: string,): boolean {
        return config.has(key,);
      },
      delete: function upstreamDelete(key: string,): void {
        config.delete(key,);
      },
      appendToArray: function upstreamAppend(input: {
        readonly key: string;
        readonly value: JsonValue;
      },): void {
        config.appendToArray(
          input.key,
          input.value,
        );
      },
      reset: function upstreamReset(input: {
        readonly keys: readonly string[];
      },): void {
        config.reset(...input.keys,);
      },
      clear: function upstreamClear(): void {
        config.clear();
      },
      replaceStore: function upstreamReplaceStore(input: {
        readonly value: JsonRecord;
      },): void {
        config.store = input.value;
      },
      readStore: function upstreamReadStore(): JsonRecord {
        return config.store;
      },
      readSize: function upstreamReadSize(): number {
        return config.size;
      },
    },
    sequence,
  });
}

//endregion Adapters
