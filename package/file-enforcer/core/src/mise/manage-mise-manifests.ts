/**
 {@link manageMiseManifests}: normalize task-header key quoting in hand-authored
 mise manifests.
 
 `mise.toml` task headers accept several spellings for the same key
 (`[tasks."lint:types"]` and `[tasks.'lint:types']`), and splice-mode TOML edits
 preserve whichever spelling a file already has. This manager rewrites the
 quoting while leaving every other byte alone, so the workspace converges on one
 form without reformatting comments, values, or ordering. See
 `doc/planning/mise-toml-file-enforcer.md`.
 
 @module
 */

import { glob, } from 'node:fs/promises';

import { normalizeTableHeaderKeyQuoting, } from '@monochromatic-dev/toml-edit-core/ts';

import {
  ABSENT_FILE_CONTENT,
  overwrite,
  readExisting,
} from '../io/write.ts';
import { addWatchedPaths, } from '../tracker.ts';

/**
 Normalizes task-header key quoting in every mise manifest matched by
 `manifestGlobs`.
 
 Manifests are registered as watched paths after discovery, so a run in watch
 mode is retriggered by an edit to any of them. Callers own glob breadth: bounded
 patterns keep discovery out of installed `node_modules` trees, and a manifest
 matched by two globs is rewritten once. Resolves after every matched manifest
 is normalized; canonical manifests produce no write.
 
 @param manifestGlobs - bounded discovery globs for hand-authored manifests
 
 @param manifestPath - optional staleness manifest path override forwarded to every write
 
 @example
 ```ts
 await manageMiseManifests({
   manifestGlobs: ['./package/<category>/<name>/mise.toml',],
 });
 ```
 */
export async function manageMiseManifests(
  {
    manifestGlobs,
    manifestPath: stalenessManifestPath,
  }: {
    readonly manifestGlobs: readonly string[];
    readonly manifestPath?: string;
  },
): Promise<void> {
  /**
   Matched manifests gathered concurrently across every discovery glob.
   */
  const manifestPaths = (await Promise.all(manifestGlobs.map(
    async function collectGlob(manifestGlob,): Promise<readonly string[]> {
      return await Array.fromAsync(glob(manifestGlob,),);
    },
  ),)).flat();
  /**
   Unique paths, sorted so overlapping globs stay deterministic and log stable.
   */
  const uniqueManifestPaths = [...new Set(manifestPaths,),].toSorted();
  addWatchedPaths(uniqueManifestPaths,);
  await Promise.all(uniqueManifestPaths.map(async function normalizeOne(manifestPath,): Promise<void> {
    /**
     Current manifest content, or absence when discovery and read disagree.
     */
    const existing = await readExisting(manifestPath,);
    if (existing === ABSENT_FILE_CONTENT) return;
    /**
     Normalized manifest text written back when it differs from the current text.
     */
    const normalized = normalizeTableHeaderKeyQuoting({ source: existing, },);
    await overwrite(stalenessManifestPath === undefined
      ? {
        dest: manifestPath,
        content: normalized,
      }
      : {
        dest: manifestPath,
        content: normalized,
        manifestPath: stalenessManifestPath,
      },);
  },),);
}
