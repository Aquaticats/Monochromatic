/**
 Disposable temp directory and plain-`JSON` projection shared by the
 differential oracle's store runners.

 Each run gets its own directory, removed however the run ends, so the fork and
 the vendored upstream snapshot never write into each other's config file.

 @module
 */

import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import type { JsonValue, } from './store-operations.ts';

//region Fixtures

/**
 Prefix of every disposable store directory this module creates.
 */
const TEMP_DIR_PREFIX = 'conf-fork-fuzz-';

/**
 Marker reported instead of file contents when no config file exists.
 */
export const ABSENT_FILE = 'absent' as const;

//endregion Fixtures

//region Handles

/**
 Disposable temp directory created for one store run.
 */
export type TempStoreDir = {
  /**
   Absolute path of the created directory.
   */
  readonly path: string;
  /**
   Removes the directory however the run ends.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable temp directory for one store run.

 @returns Handle carrying the directory path and its removal hook;
 consume it with `await using` so the directory is removed however the run
 ends.

 @example
 ```ts
 await using dir = await createTempStoreDir();
 ```
 */
export async function createTempStoreDir(): Promise<TempStoreDir> {
  /**
   Absolute path of the created directory.
   */
  const path = await mkdtemp(
    join(
      tmpdir(),
      TEMP_DIR_PREFIX,
    ),
  );
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Handles

//region Values

/**
 Normalizes a `JSON`-safe value to plain `JSON`,
 dropping prototypes and shared references so cross-implementation
 comparison is shape-exact.

 Every value reaching here is JSON-safe by construction, so a structured clone
 is the same projection as a `JSON` round-trip without parsing text.

 @param value - Value read from a store.

 @returns Plain-`JSON` copy of `value`.

 @example
 ```ts
 jsonClone([1, { a: 2 },]); // => [1, { a: 2 }]
 ```
 */
export function jsonClone(value: JsonValue,): JsonValue {
  return structuredClone(value,);
}

//endregion Values
