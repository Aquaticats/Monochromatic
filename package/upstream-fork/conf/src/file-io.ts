/**
 Synchronous filesystem primitives for the config store.
 
 Upstream `conf`'s whole API is synchronous: `get`,
 `set`,
 and `store` write
 before returning. Every `*Sync` call therefore lives in this one module so
 the repository's `no-restricted-syntax/no-sync` suppression has a single,
 documented boundary instead of scattering across the store; see
 `DECISION.sync-api.md` in this package.
 
 @module
 */

import path from 'node:path';
import process from 'node:process';
import { randomUUID, } from 'node:crypto';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region File reads

/* oxlint-disable no-restricted-syntax/no-sync -- Structurally synchronous boundary: upstream conf's get/set/store contract writes before returning, so the async API cannot serve it; every call site is inside this module per package/upstream-fork/conf/DECISION.sync-api.md. */

/**
 Reads a file as raw bytes.
 
 @param path - Absolute file path.
 
 @returns File contents as bytes.
 
 @throws Node's file error when the file is missing or unreadable;
 callers match `error.code` against `ENOENT`.
 
 @example
 ```ts
 const bytes = readFileBytes({ path: '/tmp/app/config.json', });
 ```
 */
export function readFileBytes({
  path: filePath,
}: {
  readonly path: string;
},): Uint8Array {
  return readFileSync(filePath,);
}

/**
 Reads a file as UTF-8 text.
 
 @param path - Absolute file path.
 
 @returns File contents as text.
 
 @throws Node's file error when the file is missing or unreadable;
 callers match `error.code` against `ENOENT`.
 
 @example
 ```ts
 const text = readFileText({ path: '/tmp/app/config.json', });
 ```
 */
export function readFileText({
  path: filePath,
}: {
  readonly path: string;
},): string {
  return readFileSync(
    filePath,
    'utf8',
  );
}

/**
 Reports whether a path exists on disk.
 
 @param path - Absolute path to probe.
 
 @returns `true` when the path resolves to any filesystem entry.
 
 @example
 ```ts
 pathExists({ path: '/tmp/app/config.json', }); // => false
 ```
 */
export function pathExists({
  path: filePath,
}: {
  readonly path: string;
},): boolean {
  return existsSync(filePath,);
}

//endregion File reads

//region File writes

/**
 Removes a file when present,
 logging and swallowing any failure so cleanup never masks the error being
 reported.
 
 @param path - Absolute file path to remove.
 
 @param logger - Logger for cleanup failures.
 
 @example
 ```ts
 removeFileIfExists({ path: temporaryPath, logger, });
 ```
 */
export function removeFileIfExists({
  path: filePath,
  logger,
}: {
  readonly path: string;
  readonly logger?: Logger;
},): void {
  /**
   Logger wrapped with this function's name so cleanup diagnostics name
   their origin.
   */
  const log = logger ?? tagged({
    tag: removeFileIfExists.name,
  },);
  try {
    unlinkSync(filePath,);
  }
  catch (error) {
    log.warn(`cleanup of ${filePath} failed: ${caughtValueText(error,)}`,);
  }
}

/**
 Writes bytes to a temporary sibling,
 then renames it over the target so readers see the old or new content but
 never a partial write.
 
 The temporary file lands in the target's own directory, which keeps the
 rename on one filesystem and makes the cross-device fallback upstream
 `conf` carries unnecessary. Under Snap, upstream `conf` writes directly
 (see sindresorhus/conf#82),
 which this mirrors.
 
 @param path - Absolute target file path.
 
 @param data - Bytes or text to write.
 
 @param mode - File mode for the created file, before the process umask
 reduces it.
 
 @example
 ```ts
 writeFileAtomic({
   path: '/tmp/app/config.json',
   data: '{"theme":"dark"}',
   mode: 0o666,
 });
 ```
 */
export function writeFileAtomic({
  path: filePath,
  data,
  mode,
}: {
  readonly path: string;
  readonly data: string | Uint8Array;
  readonly mode: number;
},): void {
  if (process.env
    .SNAP
    !== undefined) {
    writeFileSync(
      filePath,
      data,
      {
        mode,
      },
    );
    return;
  }
  /**
   Randomly-named sibling the content is staged in before the atomic
   rename.
   */
  const temporaryPath = path.join(
    path.dirname(filePath,),
    `.${path.basename(filePath,)}.${randomUUID()}.tmp`,
  );
  try {
    writeFileSync(
      temporaryPath,
      data,
      {
        mode,
      },
    );
    renameSync(
      temporaryPath,
      filePath,
    );
  }
  catch (error) {
    removeFileIfExists({
      path: temporaryPath,
    },);
    throw error;
  }
}

/**
 Creates a directory and any missing parents.
 
 @param path - Absolute directory path to materialize.
 
 @example
 ```ts
 ensureDirectory({ path: '/tmp/app/config', });
 ```
 */
export function ensureDirectory({
  path: directoryPath,
}: {
  readonly path: string;
},): void {
  mkdirSync(
    directoryPath,
    {
      recursive: true,
    },
  );
}

/* oxlint-enable no-restricted-syntax/no-sync */

//endregion File writes
