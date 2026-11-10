/**
 Filesystem path-emptying utilities for Node.js / Bun.
 
 Each function clears the contents of a file or directory
 without removing the path itself.
 */

import {
  readdir,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { posix, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { stripQuerySuffix, } from './query-suffix.ts';

/**
 Module-scoped tagged logger.
 */
const l = tagged({ tag: 'path/empty', },);

/* oxlint-disable eslint/require-await -- delegates to emptyFile/emptyDir which are async */
/**
 Empties a path, choosing file or directory behavior based on
 whether the path has a file extension, dispatching to {@link emptyFile}
 or {@link emptyDir}.
 Query parameters in paths (e.g. `?raw`) are stripped before inspection.
 
 @param path - Filesystem path to empty
 
 @returns Original path string (including any query suffix)
 
 @example
 ```ts
 await emptyPath('dist/output');       // removes all contents of directory
 await emptyPath('dist/bundle.js');    // truncates file to empty
 ```
 */
export async function emptyPath(path: string,): Promise<string> {
  /**
   Path the filesystem operations use: only this form reaches `posix.parse`, so the query suffix cannot be read as an extension.
   */
  const cleanPath = stripQuerySuffix(path,);
  /**
   Parsed segments used solely to read `ext`, which decides file-vs-directory dispatch.
   */
  const parsed = posix.parse(cleanPath,);

  if (parsed.ext) {
    l.info(`${path} has extension, emptying as file`,);
    return emptyFile(path,);
  }

  return emptyDir(path,);
}
/* oxlint-enable eslint/require-await */

/**
 Removes all entries inside a directory without removing the directory itself.
 Query parameters in paths (e.g. `?raw`) are stripped.
 
 @param path - Directory path to empty
 
 @returns Original path string (including any query suffix)
 
 @example
 ```ts
 await emptyDir('dist');
 ```
 */
export async function emptyDir(path: string,): Promise<string> {
  /**
   Path the filesystem operations use: everything from its first `?` on names no entry.
   */
  const cleanPath = stripQuerySuffix(path,);
  /**
   Snapshot of directory contents taken once so removals don't race with a live iterator.
   */
  const entries = await readdir(cleanPath,);

  await Promise.all(entries.map(function removeEntry(entry,): Promise<void> {
    return rm(
      posix.join(
        cleanPath,
        entry,
      ),
      {
        recursive: true,
        force: true,
      },
    );
  },),);

  return path;
}

/**
 Truncates a file to zero bytes.
 Query parameters in paths (e.g. `?raw`) are stripped.
 
 @param path - File path to empty
 
 @returns Original path string (including any query suffix)
 
 @example
 ```ts
 await emptyFile('dist/bundle.js');
 ```
 */
export async function emptyFile(path: string,): Promise<string> {
  /**
   Path the write targets: everything from its first `?` on names no entry.
   */
  const cleanPath = stripQuerySuffix(path,);

  await writeFile(
    cleanPath,
    '',
  );
  return path;
}

/**
 Removes all empty files (zero bytes after trimming) from a directory.
 Non-file entries and files with content are left untouched.
 Query parameters in paths (e.g. `?raw`) are stripped.
 
 @param path - Directory path to scan
 
 @returns Original path string (including any query suffix)
 
 @example
 ```ts
 await removeEmptyFilesInDir('src/generated');
 ```
 */
export async function removeEmptyFilesInDir(path: string,): Promise<string> {
  /**
   Path the scan reads: everything from its first `?` on names no entry.
   */
  const cleanPath = stripQuerySuffix(path,);
  /**
   Snapshot of directory contents taken once so the removal pass doesn't race with a live iterator.
   */
  const entries = await readdir(cleanPath,);

  await Promise.all(
    entries.map(async function checkAndRemoveIfEmpty(entry,): Promise<void> {
      /**
       Absolute path to the entry so `stat`, `readFile`, and `rm` all target the same node regardless of `cwd`.
       */
      const fullPath = posix.join(
        cleanPath,
        entry,
      );
      /**
       File metadata used to skip directories and other non-regular entries before reading.
       */
      const stats = await stat(fullPath,);

      if (stats.isFile()) {
        /**
         File body read in full because emptiness is decided after trimming whitespace, not by size alone.
         */
        const content = await readFile(
          fullPath,
          'utf8',
        );
        if (content.trim()
          === '') {
          l.debug(`Removing empty file ${fullPath}`,);
          await rm(
            fullPath,
            { force: true, },
          );
        }
      }
    },),
  );

  return path;
}
