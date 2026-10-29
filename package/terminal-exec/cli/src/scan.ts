/**
 Scans XDG application directories for `.desktop` files.
 Derives entry IDs from file paths following the Desktop Entry Specification
 (subdirectory separators become dashes in the ID).
 
 @module
 */

import type { Dirent, } from 'node:fs';
import { readdir, } from 'node:fs/promises';
import {
  join,
  relative,
} from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Logger root for terminal-exec after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'terminal-exec', },);

/**
 Tagged logger for this module.
 */
const l = tagged({
  tag: 'scan',
  l: parentLogger,
},);

/**
 Sentinel returned by {@link readdirOrAbsent} when a directory cannot be read.
 A `unique symbol`; the walker narrows with `=== DIR_UNREADABLE` and skips it.
 */
const DIR_UNREADABLE: unique symbol = Symbol('terminal-exec/application directory cannot be read',);

/**
 Lists directory entries with type info, returning {@link DIR_UNREADABLE} when the directory cannot be read.
 Used so the walker skips missing or unreadable directories without leaking a let into walk's body.
 
 @param current - Absolute directory path to read.
 
 @returns Array of Dirent entries, or {@link DIR_UNREADABLE} when readdir throws.
 
 @example
 ```ts
 const entries = await readdirOrAbsent({ current: '/usr/share/applications' })
 // entries === [<Dirent for 'org.gnome.Terminal.desktop'>, ...] when present
 ```
 */
async function readdirOrAbsent(
  { current, }: { readonly current: string; },
): Promise<readonly Dirent[] | typeof DIR_UNREADABLE> {
  try {
    return await readdir(
      current,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return DIR_UNREADABLE;
  }
}

/**
 Registry entry mapping an entry ID to its filesystem path.
 When the same ID appears in multiple directories, the highest-priority
 directory (last in the ascending-priority list) wins.
 */
export type EntryRegistration = {
  /**
   Desktop entry ID, e.g. `com.mitchellh.ghostty.desktop`.
   */
  readonly id: string;
  /**
   Absolute path to the `.desktop` file.
   */
  readonly path: string;
};

/**
 Recursively finds all `.desktop` files in a directory.
 
 @param dir - Absolute path to the applications directory.
 
 @returns Array of absolute file paths.
 */
async function findDesktopFiles({ dir, }: { readonly dir: string; },): Promise<readonly string[]> {
  /**
   Mutable accumulator filled by the inner walk function.
   */
  const results: string[] = [];

  /**
   Recursively collects `.desktop` file paths under `current`.
   
   @param current - Directory to walk.
   */
  async function walk({ current, }: { readonly current: string; },): Promise<void> {
    /**
     DIR_UNREADABLE when readdir throws (e.g. directory missing or unreadable); skip the directory.
     */
    const entries = await readdirOrAbsent({ current, },);
    if (entries === DIR_UNREADABLE)
      return;
    for (const entry of entries) {
      /**
       Absolute candidate for recursion or `.desktop` matching.
       */
      const fullPath = join(
        current,
        entry.name,
      );
      if (entry.isDirectory()) {
        /* oxlint-disable-next-line no-await-in-loop -- recursive directory walk must be sequential */
        await walk({ current: fullPath, },);
      }
      else if (entry.name
        .endsWith('.desktop',)) {
        results.push(fullPath,);
      }
    }
  }
  await walk({ current: dir, },);
  return results;
}

/**
 Scans all application directories for `.desktop` files and builds a registry.
 Directories are provided in ascending priority order; later directories
 override earlier ones for duplicate entry IDs. Each directory's files come
 from {@link findDesktopFiles}.
 
 @param dirs - Application directory paths in ascending priority order.
 
 @returns Map of entry ID to registration, and ordered fallback ID list (highest priority first).
 
 @example
 ```ts
 const { registry, fallbackIds } = await scanEntries({
   dirs: ['/usr/share/applications/', '~/.local/share/applications/']
 })
 ```
 */
export async function scanEntries({ dirs, }: { readonly dirs: readonly string[]; },): Promise<{
  readonly registry: ReadonlyMap<string, EntryRegistration>;
  readonly fallbackIds: readonly string[];
}> {
  /**
   Mutable map; later dirs override earlier for same ID.
   */
  const registry = new Map<string, EntryRegistration>();
  /**
   Tracks insertion order per directory for fallback priority.
   */
  const allIds: string[] = [];

  for (const dir of dirs) {
    /* oxlint-disable no-await-in-loop -- sequential: later dirs override earlier for same ID */
    /**
     Desktop files in one directory; one batch per priority level.
     */
    const files = await findDesktopFiles({ dir, },);
    /* oxlint-enable no-await-in-loop */
    for (const filePath of files) {
      /**
       Subpath used to derive the entry id.
       */
      const rel = relative(
        dir,
        filePath,
      );
      /**
       Entry id per spec: subdir separators become dashes.
       */
      const id = rel.replaceAll(
        '/',
        '-',
      );
      registry.set(
        id,
        {
          id,
          path: filePath,
        },
      );
      /**
       Remove previous occurrence so re-adding puts it at the end (higher priority).
       */
      const prevIdx = allIds.indexOf(id,);
      if (prevIdx !== (-1)) {
        allIds.splice(
          prevIdx,
          1,
        );
      }
      allIds.push(id,);
    }
  }

  /**
   Reverse so highest-priority entries come first in fallback ordering.
   */
  const fallbackIds = allIds.toReversed();

  l.debug(
    `scanned ${String(registry.size,)} entries, ${
      String(fallbackIds.length,)
    } fallback candidates`,
  );
  return {
    registry,
    fallbackIds,
  };
}
