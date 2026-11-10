/**
 Records one generated operation sequence against one store surface:
 per-operation outcomes,
 final store contents,
 item count,
 and the config file's parsed `JSON`.

 Outcomes carry error names and message texts verbatim, so the differential
 oracle compares implementations rather than this package's diagnostics.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  ABSENT_FILE,
  jsonClone,
} from './store-fixture.ts';
import type {
  StoreSurface,
} from './store-surface.ts';
import type {
  JsonRecord,
  JsonValue,
  StoreOperation,
  StoreOperationSequence,
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
     `get` result.
     */
      readonly kind: 'value';
    /**
     Whether the key resolved to a stored value.
     */
      readonly found: boolean;
    /**
     Value read,
     normalized to plain `JSON`;
     absent exactly when `found` is `false`.
     */
      readonly value?: JsonValue;
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

//endregion Types

//region Helpers

/**
 Whether a thrown value is the `ENOENT` error an absent file raises.

 @param error - Value a file read threw.

 @returns `true` when the file was missing.

 @example
 ```ts
 isMissingFileError(new Error('boom',)); // => false
 ```
 */
function isMissingFileError(error: unknown,): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && (String(error.code,) === 'ENOENT');
}

/**
 Reads the config file's parsed contents,
 or the absent marker when no file exists.

 The read is attempted directly and `ENOENT` is treated as absent, so no
 separate existence check can race the read.

 @param path - Config file path.

 @throws Error when the file exists and cannot be read or parsed.

 @returns Parsed contents,
 or `'absent'`.

 @example
 ```ts
 await readFileContents('/tmp/store/config.json');
 ```
 */
async function readFileContents(path: string,): Promise<JsonRecord | 'absent'> {
  try {
    /**
     Raw file contents.
     */
    const text = await readFile(
      path,
      'utf8',
    );
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the config file always holds JSON-object contents in these runs
    return JSON.parse(text,) as JsonRecord;
  }
  catch (error: unknown) {
    if (isMissingFileError(error,))
      return ABSENT_FILE;
    throw error;
  }
}

/**
 Renders a thrown value for cross-implementation comparison:
 name,
 base class,
 and message text.

 @param error - Value the operation threw.

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
       Answer the store gave for the requested key.
       */
      const lookup = surface.get({
        key: operation.key,
        useDefault: operation.useDefault,
        defaultValue: operation.defaultValue,
      },);
      return (lookup.kind === 'found')
        ? {
          kind: 'value',
          found: true,
          value: jsonClone(lookup.value,),
        }
        : {
          kind: 'value',
          found: false,
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
 const observation = await runSequence({ surface, sequence, });
 ```
 */
export async function runSequence(
  {
    surface,
    sequence,
  }: {
    readonly surface: StoreSurface;
    readonly sequence: StoreOperationSequence;
  },
): Promise<StoreObservation> {
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
  const fileContents = await readFileContents(surface.path,);

  return {
    outcomes,
    finalStore: {
      ...surface.readStore(),
    },
    size: surface.readSize(),
    fileContents,
  };
}

//endregion Helpers
