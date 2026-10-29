/**
 Atomic JSON file writing helpers for Electron integration tests.
 
 @example
 ```ts
 await writeJsonFileAtomically({ filePath: '/tmp/state.json', value: { count: 1 } });
 ```
 */

import {
  rename,
  writeFile,
} from 'node:fs/promises';

/**
 JSON scalar supported by this infra package.
 
 @example
 ```ts
 const value: JsonScalar = 1;
 ```
 */
export type JsonScalar = boolean | number | string;

/**
 Shallow JSON object used for boundary-test state snapshots.
 
 @example
 ```ts
 const state: JsonObject = { count: 1 };
 ```
 */
export type JsonObject = Record<string, JsonScalar>;

/**
 Writes a JSON file through a same-directory temporary file and rename.
 
 @param filePath - Destination path readers observe.
 
 @param value - JSON object to serialize.
 
 @mutates value - `JSON.stringify` may invoke record accessors or proxy traps.
 
 @example
 ```ts
 await writeJsonFileAtomically({ filePath: '/tmp/state.json', value: { ready: true } });
 ```
 */
export async function writeJsonFileAtomically(
  {
    filePath,
    value,
  }: {
    readonly filePath: string;
    readonly value: JsonObject;
  },
): Promise<void> {
  /**
   Unique temporary path in destination directory.
   */
  const temporaryPath = `${filePath}.${process.pid}.${Date.now()}.tmp`;

  await writeFile(
    temporaryPath,
    `${JSON.stringify(
      value,
      null,
      2,
    )}\n`,
    'utf8',
  );
  await rename(
    temporaryPath,
    filePath,
  );
}
