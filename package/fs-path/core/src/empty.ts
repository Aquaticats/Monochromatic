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
   Position of the query separator so the bundler-style `?raw` suffix can be stripped before `posix.parse`.
   */
  const queryIndex = path.indexOf('?',);
  /**
   Path with any query suffix removed; only this form is fed to `posix.parse` so extension detection is not fooled by the query.
   */
  const cleanPath = queryIndex !== (-1)
    ? path.slice(
      0,
      queryIndex,
    )
    : path;
  /**
   Parsed segments used solely to read `ext`, which decides file-vs-directory dispatch.
   */
  const parsed = posix.parse(cleanPath,);

  if (parsed.ext) {
    l.info(`${path} has extension, emptying as file`,);
    return emptyFile(path,);
  }

  return emptyDir(cleanPath,);
}
/* oxlint-enable eslint/require-await */

/**
 Removes all entries inside a directory without removing the directory itself.
 
 @param path - Directory path to empty
 
 @returns Resolved path string
 
 @example
 ```ts
 await emptyDir('dist');
 ```
 */
export async function emptyDir(path: string,): Promise<string> {
  /**
   Snapshot of directory contents taken once so removals don't race with a live iterator.
   */
  const entries = await readdir(path,);

  await Promise.all(entries.map(function removeEntry(entry,): Promise<void> {
    return rm(
      posix.join(
        path,
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
   Position of the query separator so the bundler-style `?raw` suffix can be stripped before the write.
   */
  const queryIndex = path.indexOf('?',);
  /**
   Path with any query suffix removed so `writeFile` targets the actual on-disk file.
   */
  const cleanPath = queryIndex !== (-1)
    ? path.slice(
      0,
      queryIndex,
    )
    : path;

  await writeFile(
    cleanPath,
    '',
  );
  return path;
}

/**
 Removes all empty files (zero bytes after trimming) from a directory.
 Non-file entries and files with content are left untouched.
 
 @param path - Directory path to scan
 
 @returns Resolved path string
 
 @example
 ```ts
 await removeEmptyFilesInDir('src/generated');
 ```
 */
export async function removeEmptyFilesInDir(path: string,): Promise<string> {
  /**
   Snapshot of directory contents taken once so the removal pass doesn't race with a live iterator.
   */
  const entries = await readdir(path,);

  await Promise.all(
    entries.map(async function checkAndRemoveIfEmpty(entry,): Promise<void> {
      /**
       Absolute path to the entry so `stat`, `readFile`, and `rm` all target the same node regardless of `cwd`.
       */
      const fullPath = posix.join(
        path,
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
